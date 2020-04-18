use crate::{
    color::*,
    mesh::{self, Mesh, Vertex},
    sprite::*,
    window::*,
    Point2f, Vector2f,
};
use backend;
use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    buffer,
    command::{self, BufferImageCopy, CommandBuffer},
    device::Device,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{
        self as img, Access, Extent, Filter, Layout, Offset, SubresourceLayers, SubresourceRange,
        ViewCapabilities, WrapMode,
    },
    memory::{Barrier, Dependencies, Properties, Segment},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, Subpass, SubpassDesc},
    pool::{self, CommandPool},
    pso::{
        self, AttributeDesc, BufferDescriptorFormat, BufferDescriptorType, Descriptor,
        DescriptorPool, DescriptorRangeDesc, DescriptorSetLayoutBinding, DescriptorSetWrite,
        DescriptorType, Element, EntryPoint, GraphicsPipelineDesc, GraphicsShaderSet,
        ImageDescriptorType, PipelineStage, ShaderStageFlags, Specialization, VertexBufferDesc,
        Primitive,
    },
    queue::{family::QueueGroup, CommandQueue, QueueFamily, Submission},
    window::{self, Extent2D, PresentationSurface, Surface},
    Backend, IndexType, Instance, MemoryTypeId,
};
use glm;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    rc::Rc,
};

pub(crate) type GfxInstance = ::backend::Instance;
pub(crate) type GfxBuffer = <::backend::Backend as Backend>::Buffer;
pub(crate) type GfxMemory = <::backend::Backend as Backend>::Memory;
pub(crate) type GfxImage = <::backend::Backend as Backend>::Image;
pub(crate) type GfxImageView = <::backend::Backend as Backend>::ImageView;
pub(crate) type GfxRenderPass = <::backend::Backend as Backend>::RenderPass;
pub(crate) type GfxSemaphore = <::backend::Backend as Backend>::Semaphore;
pub(crate) type GfxFence = <::backend::Backend as Backend>::Fence;
pub(crate) type GfxDescriptorSetLayout = <::backend::Backend as Backend>::DescriptorSetLayout;
pub(crate) type GfxDescriptorPool = <::backend::Backend as Backend>::DescriptorPool;
pub(crate) type GfxDescriptorSet = <::backend::Backend as Backend>::DescriptorSet;
pub(crate) type GfxPipelineLayout = <::backend::Backend as Backend>::PipelineLayout;
pub(crate) type GfxShaderModule = <::backend::Backend as Backend>::ShaderModule;
pub(crate) type GfxSampler = <::backend::Backend as Backend>::Sampler;
pub(crate) type GfxGraphicsPipeline = <::backend::Backend as Backend>::GraphicsPipeline;
pub(crate) type GfxFramebuffer = <::backend::Backend as Backend>::Framebuffer;
pub(crate) type GfxSwapchain = <::backend::Backend as Backend>::Swapchain;
pub(crate) type GfxSurface = <::backend::Backend as Backend>::Surface;
pub(crate) type GfxCommandPool = <::backend::Backend as Backend>::CommandPool;
pub(crate) type GfxCommandBuffer = <::backend::Backend as Backend>::CommandBuffer;
pub(crate) type GfxDevice = <::backend::Backend as Backend>::Device;
pub(crate) type GfxAdapter = Adapter<::backend::Backend>;
pub(crate) type GfxQueueGroup = QueueGroup<backend::Backend>;

pub(crate) type GfxDeviceHandle = Rc<RefCell<GfxDevice>>;
pub(crate) type GpuTextureId = u16;

const MAX_SPRITES: u64 = 4096;
const MAX_BATCH_VERTICES: u64 = MAX_SPRITES * 4;
const MAX_BATCH_INDICES: u64 = MAX_SPRITES * 6;
const MAX_DESCRIPTOR_SETS: usize = 512;

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

pub type RenderKey = u64;
pub type ShaderProgramId = u16;
pub type TextureId = u16;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Transparency {
    Opaque = 0,
    Transparent = 1,
}

impl Default for Transparency {
    fn default() -> Self {
        Transparency::Opaque
    }
}

#[derive(Clone)]
pub enum Renderable {
    Quad {
        bl: (f32, f32),
        br: (f32, f32),
        tl: (f32, f32),
        tr: (f32, f32),
        color: Color,
    },
    Sprite {
        x: f32,
        y: f32,
        pivot: Point2f,
        scale: Vector2f,
        color: Color,
        region: SpriteRegion,
    },
}

#[derive(Clone)]
pub struct ShaderDescriptorBinding {
    pub ty: DescriptorType,
    pub stage_flags: ShaderStageFlags,
}

#[derive(Clone)]
pub struct RenderCommand {
    pub transparency: Transparency,
    pub shader_program_id: ShaderProgramId,
    pub tex_id: TextureId,
    pub layer: u8,
    pub data: Renderable,
}

impl RenderCommand {
    pub fn key(&self) -> RenderKey {
        RenderBatch::gen_key(
            self.transparency,
            self.layer,
            self.shader_program_id,
            self.tex_id,
        )
    }
}

pub struct RenderBatch {
    device: GfxDeviceHandle,
    transparency: Transparency,
    layer: u8,
    shader_program_id: ShaderProgramId,

    // texture id, width, height
    tex_info: (GpuTextureId, u32, u32),
    descriptor_set: GfxDescriptorSet,

    // Buffers
    vertex_buffer: (Option<GfxBuffer>, Option<GfxMemory>, usize),
    index_buffer: (Option<GfxBuffer>, Option<GfxMemory>, usize),
    batch_mesh: Option<Mesh>,
}

impl RenderBatch {
    pub fn new(
        device: GfxDeviceHandle,
        transparency: Transparency,
        layer: u8,
        shader_program_id: ShaderProgramId,
        tex_info: (u16, u32, u32),
        descriptor_set: GfxDescriptorSet,
        vertex_buffer: (Option<GfxBuffer>, Option<GfxMemory>, usize),
        index_buffer: (Option<GfxBuffer>, Option<GfxMemory>, usize),
    ) -> Self {
        let batch_mesh = Some(Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        });

        RenderBatch {
            device,
            transparency,
            layer,
            shader_program_id,
            tex_info,
            descriptor_set,
            vertex_buffer,
            index_buffer,
            batch_mesh,
        }
    }

    pub fn key(&self) -> RenderKey {
        let tex_id = self.tex_info.0;
        RenderBatch::gen_key(
            self.transparency,
            self.layer,
            self.shader_program_id,
            tex_id,
        )
    }

    pub fn tex_id(&self) -> u16 {
        self.tex_info.0
    }

    pub fn descriptor_set_ref(&self) -> &GfxDescriptorSet {
        &self.descriptor_set
    }

    pub fn vertex_buffer_ref(&self) -> &GfxBuffer {
        self.vertex_buffer.0.as_ref().unwrap()
    }

    pub fn vertex_buffer_mem_ref(&self) -> &GfxMemory {
        &self.vertex_buffer.1.as_ref().unwrap()
    }

    pub fn index_buffer_ref(&self) -> &GfxBuffer {
        &self.index_buffer.0.as_ref().unwrap()
    }

    pub fn index_buffer_mem_ref(&self) -> &GfxMemory {
        &self.index_buffer.1.as_ref().unwrap()
    }

    pub fn take_mesh(&mut self) -> Mesh {
        let mesh = self.batch_mesh.take().unwrap();
        self.batch_mesh = Some(Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        });
        mesh
    }

    pub fn process_command(&mut self, command: RenderCommand) {
        match command.data {
            Renderable::Quad {
                bl,
                br,
                tl,
                tr,
                color,
            } => {
                mesh::add_quad(self.batch_mesh.as_mut().unwrap(), bl, br, tl, tr, color);
            }
            Renderable::Sprite {
                x,
                y,
                pivot,
                scale,
                color,
                region,
            } => {
                mesh::add_sprite(
                    self.batch_mesh.as_mut().unwrap(),
                    x,
                    y,
                    pivot,
                    scale,
                    color,
                    region,
                    self.tex_info.1,
                    self.tex_info.2,
                );
            }
        }
    }

    pub fn clear(&mut self) {
        if let Some(batch_mesh) = self.batch_mesh.as_mut() {
            batch_mesh.clear();
        }
    }

    fn gen_key(
        transparency: Transparency,
        layer: u8,
        shader_program_id: ShaderProgramId,
        tex_id: TextureId,
    ) -> RenderKey {
        ((transparency as RenderKey) << 56)
            + ((layer as RenderKey) << 48)
            + ((shader_program_id as RenderKey) << 32)
            + ((tex_id as RenderKey) << 16)
    }
}

impl Drop for RenderBatch {
    fn drop(&mut self) {
        println!("Cleaning up RenderBatch {}", self.key());

        let device = self.device.borrow();
        unsafe {
            device.destroy_buffer(self.vertex_buffer.0.take().unwrap());
            device.free_memory(self.vertex_buffer.1.take().unwrap());

            device.destroy_buffer(self.index_buffer.0.take().unwrap());
            device.free_memory(self.index_buffer.1.take().unwrap());
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct UniformBufferObject {
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

pub struct GpuTexture {
    id: GpuTextureId,
    device: GfxDeviceHandle,
    image: Option<GfxImage>,
    memory: Option<GfxMemory>,
    image_view: Option<GfxImageView>,
    sampler: Option<GfxSampler>,
    w: u32,
    h: u32,
}

impl Drop for GpuTexture {
    fn drop(&mut self) {
        println!("Cleaning up GpuTexture {}", self.id);

        let device = self.device.borrow_mut();
        unsafe {
            // TODO (declan, 3/22/2020)
            // Wait for an image transfer fence first?

            device.destroy_image(self.image.take().unwrap());
            device.destroy_image_view(self.image_view.take().unwrap());
            device.destroy_sampler(self.sampler.take().unwrap());
            device.free_memory(self.memory.take().unwrap());
        }
    }
}

struct RenderProgram {
    device: GfxDeviceHandle,
    vert_shader: Option<GfxShaderModule>,
    frag_shader: Option<GfxShaderModule>,
    pipeline: Option<GfxGraphicsPipeline>,
    pipeline_layout: Option<GfxPipelineLayout>,
    descriptor_pool: Option<GfxDescriptorPool>,
    descriptor_set_layout: Option<GfxDescriptorSetLayout>,
    shader_descriptor_bindings: Vec<ShaderDescriptorBinding>,
}

impl Drop for RenderProgram {
    fn drop(&mut self) {
        println!("Cleaning up RenderProgram");

        let device = self.device.borrow();
        unsafe {
            device.destroy_shader_module(self.vert_shader.take().unwrap());
            device.destroy_shader_module(self.frag_shader.take().unwrap());
            device.destroy_graphics_pipeline(self.pipeline.take().unwrap());
            device.destroy_pipeline_layout(self.pipeline_layout.take().unwrap());
            device.destroy_descriptor_set_layout(self.descriptor_set_layout.take().unwrap());

            self.descriptor_pool.as_mut().unwrap().reset();
            device.destroy_descriptor_pool(self.descriptor_pool.take().unwrap());
        }
    }
}

pub struct Renderer {
    instance: GfxInstance,
    surface: Option<GfxSurface>,
    adapter: GfxAdapter,
    device: GfxDeviceHandle,
    queue_group: GfxQueueGroup,
    command_pools: Option<Vec<GfxCommandPool>>,
    command_buffers: Vec<GfxCommandBuffer>,
    surface_color_format: Format,
    depth_format: Format,
    dimensions: Extent2D,
    viewport: pso::Viewport,
    render_scale: f32,

    frame_semaphores: Option<Vec<GfxSemaphore>>,
    frame_fences: Option<Vec<GfxFence>>,

    render_pass: Option<GfxRenderPass>,
    shader_programs: HashMap<ShaderProgramId, RenderProgram>,

    uniform_buffer: Option<GfxBuffer>,
    uniform_buffer_memory: Option<GfxMemory>,
    uniform_buffer_frame_size: usize,

    textures: HashMap<TextureId, GpuTexture>,
    batches: HashMap<RenderKey, RenderBatch>,

    frames_in_flight: usize,
    current_frame: usize,
}

impl Renderer {
    pub fn new(window: &WinitWindow, render_scale: f32) -> Renderer {
        // Create an instance, which is the entry point to the graphics API.
        let instance =
            GfxInstance::create("gfx-rs", 1).expect("Failed to create backend instance!");

        // Create a surface, which is an abstraction over the OS's native window.
        let surface = unsafe {
            instance
                .create_surface(window)
                .expect("Failed to create window surface!")
        };

        // Grab the first available adapter.
        // An adapter represents a physical device, like a GPU.
        // TODO do we actually need to iterate and grab a proper adapter
        let adapter = instance.enumerate_adapters().remove(0);

        let family = adapter
            .queue_families
            .iter()
            .find(|family| {
                surface.supports_queue_family(family) && family.queue_type().supports_graphics()
            })
            .unwrap();

        let mut gpu = unsafe {
            adapter
                .physical_device
                .open(&[(family, &[1.0])], gfx_hal::Features::empty())
                .unwrap()
        };

        // The device is a logical device that allows us to perform GPU operations.
        // The queue group contains a set of command queues which we can submit drawing commands to.
        let queue_group = gpu.queue_groups.pop().unwrap();
        let device = gpu.device;

        let frames_in_flight = 2;

        // The number of the rest of the resources is based on the frames in flight.
        let mut frame_semaphores: Vec<GfxSemaphore> = Vec::with_capacity(frames_in_flight);
        let mut frame_fences: Vec<GfxFence> = Vec::with_capacity(frames_in_flight);
        let mut command_pools: Vec<GfxCommandPool> = Vec::with_capacity(frames_in_flight);
        let mut command_buffers: Vec<GfxCommandBuffer> = Vec::with_capacity(frames_in_flight);

        // A command pool is used to acquire command buffers, which are used to
        // send drawing instructions to the GPU.
        for _ in 0..frames_in_flight {
            unsafe {
                command_pools.push(
                    device
                        .create_command_pool(
                            queue_group.family,
                            pool::CommandPoolCreateFlags::empty(),
                        )
                        .expect("Failed to create create command pool for frame!"),
                );
            }
        }

        for i in 0..frames_in_flight {
            frame_semaphores.push(
                device
                    .create_semaphore()
                    .expect("Failed to create frame semaphore!"),
            );

            frame_fences.push(
                device
                    .create_fence(true)
                    .expect("Failed to create frame fence!"),
            );

            command_buffers.push(unsafe { command_pools[i].allocate_one(command::Level::Primary) });
        }

        // Grab the supported image formats for our surface, then decide on a surface color format
        let formats = surface.supported_formats(&adapter.physical_device);
        println!("Surface supported formats: {:?}", formats);
        let surface_color_format = formats.map_or(Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        // TODO (Declan, 10/16/2018)
        // Need to do some stuff to actually find a supported depth format
        let depth_format = Format::D32SfloatS8Uint;

        // Wrapping the device in a reference counted ref cell, because it will need to be shared with various resources
        let device: GfxDeviceHandle = Rc::new(RefCell::new(device));

        let render_pass = create_render_pass(device.clone(), surface_color_format, depth_format);
        let shader_programs = {
            let mut shader_programs: HashMap<u16, RenderProgram> = HashMap::new();

            shader_programs.insert(
                0,
                create_render_program(
                    device.clone(),
                    &render_pass,
                    "gfx-lib/res/shaders/bin/untextured.glslv.spv",
                    "gfx-lib/res/shaders/bin/untextured.glslf.spv",
                    vec![ShaderDescriptorBinding {
                        ty: DescriptorType::Buffer {
                            ty: BufferDescriptorType::Uniform,
                            format: BufferDescriptorFormat::Structured {
                                dynamic_offset: false,
                            },
                        },
                        stage_flags: ShaderStageFlags::VERTEX,
                    }],
                    Primitive::TriangleList
                ),
            );

            shader_programs.insert(
                1,
                create_render_program(
                    device.clone(),
                    &render_pass,
                    "gfx-lib/res/shaders/bin/textured.glslv.spv",
                    "gfx-lib/res/shaders/bin/textured.glslf.spv",
                    vec![
                        ShaderDescriptorBinding {
                            ty: DescriptorType::Buffer {
                                ty: BufferDescriptorType::Uniform,
                                format: BufferDescriptorFormat::Structured {
                                    dynamic_offset: false,
                                },
                            },
                            stage_flags: ShaderStageFlags::VERTEX,
                        },
                        ShaderDescriptorBinding {
                            ty: DescriptorType::Image {
                                ty: ImageDescriptorType::Sampled {
                                    with_sampler: false,
                                },
                            },
                            stage_flags: ShaderStageFlags::FRAGMENT,
                        },
                        ShaderDescriptorBinding {
                            ty: DescriptorType::Sampler,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                        },
                    ],
                    Primitive::TriangleList,
                ),
            );

            shader_programs.insert(
                2,
                create_render_program(
                    device.clone(),
                    &render_pass,
                    "gfx-lib/res/shaders/bin/untextured.glslv.spv",
                    "gfx-lib/res/shaders/bin/untextured.glslf.spv",
                    vec![ShaderDescriptorBinding {
                        ty: DescriptorType::Buffer {
                            ty: BufferDescriptorType::Uniform,
                            format: BufferDescriptorFormat::Structured {
                                dynamic_offset: false,
                            },
                        },
                        stage_flags: ShaderStageFlags::VERTEX,
                    }],
                    Primitive::LineStrip
                ),
            );

            shader_programs
        };

        // Create the uniform buffer
        let (uniform_buffer, uniform_buffer_memory, uniform_buffer_frame_size) =
            create_uniform_buffer(
                device.clone(),
                &adapter.physical_device,
                UniformBufferObject {
                    view: glm::Mat4::identity().into(),
                    model: glm::Mat4::identity().into(),
                    projection: glm::Mat4::identity().into(),
                },
                frames_in_flight,
            );

        let window_inner_size = window.inner_size();
        let dimensions = Extent2D {
            width: window_inner_size.width,
            height: window_inner_size.height,
        };

        let viewport = pso::Viewport {
            rect: pso::Rect {
                x: 0,
                y: 0,
                w: dimensions.width as _,
                h: dimensions.height as _,
            },
            depth: 0.0..1.0,
        };

        Renderer {
            instance,
            surface: Some(surface),
            adapter,
            device,
            queue_group,
            command_pools: Some(command_pools),
            command_buffers,
            surface_color_format,
            depth_format,
            dimensions,
            viewport,
            render_scale,
            frame_semaphores: Some(frame_semaphores),
            frame_fences: Some(frame_fences),
            render_pass: Some(render_pass),
            shader_programs,
            uniform_buffer: Some(uniform_buffer),
            uniform_buffer_memory: Some(uniform_buffer_memory),
            uniform_buffer_frame_size,
            textures: HashMap::new(),
            batches: HashMap::new(),
            frames_in_flight,
            current_frame: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.dimensions = Extent2D { width, height };

        self.rebuild_swapchain();
    }

    pub fn create_render_batch(
        &mut self,
        transparency: Transparency,
        layer: u8,
        shader_program_id: ShaderProgramId,
        tex_id: u16,
    ) -> Result<RenderKey, gfx_hal::pso::AllocationError> {
        // If we already have a batch with this key, get it
        let key = RenderBatch::gen_key(transparency, layer, shader_program_id, tex_id);
        if let Some(batch) = self.batches.get_mut(&key) {
            batch.clear();
            return Ok(key);
        }

        let (descriptor_set, shader_descriptor_bindings) = {
            let shader_program = match self.shader_programs.get_mut(&shader_program_id) {
                Some(s) => s,
                None => panic!(
                    "Failed to create render batch: Referenced shader program did not exist!"
                ),
            };

            // Grab the descriptor pool and layout from the shader program
            let pool: &mut GfxDescriptorPool = shader_program.descriptor_pool.as_mut().unwrap();
            let layout: &GfxDescriptorSetLayout =
                shader_program.descriptor_set_layout.as_ref().unwrap();

            // Allocate a descriptor set from the pool, with the provided layout
            let descriptor_set = match unsafe { pool.allocate_set(layout) } {
                Ok(set) => set,
                Err(e) => {
                    eprintln!("Failed to create batch! {:?}", e);
                    panic!();
                }
            };

            (
                descriptor_set,
                shader_program.shader_descriptor_bindings.clone(),
            )
        };

        // Create vertex buffer
        let (vertex_buffer, vertex_buffer_memory, vertex_buffer_frame_size) = create_vertex_buffer(
            self.device.clone(),
            &self.adapter.physical_device,
            &[],
            self.frames_in_flight,
        );

        // Create index buffer
        let (index_buffer, index_buffer_memory, index_buffer_frame_size) = create_index_buffer(
            self.device.clone(),
            &self.adapter.physical_device,
            &[],
            self.frames_in_flight,
        );

        let tex_info = if let Some(tex) = self.textures.get(&tex_id) {
            (tex_id, tex.w, tex.h)
        } else {
            (tex_id, 0, 0)
        };

        let batch = RenderBatch::new(
            self.device.clone(),
            transparency,
            layer,
            shader_program_id,
            tex_info,
            descriptor_set,
            (
                Some(vertex_buffer),
                Some(vertex_buffer_memory),
                vertex_buffer_frame_size,
            ),
            (
                Some(index_buffer),
                Some(index_buffer_memory),
                index_buffer_frame_size,
            ),
        );

        self.write_descriptor_sets(&batch, shader_descriptor_bindings);

        // Cache batch
        let key = batch.key();
        self.batches.insert(key, batch);

        println!("[GFX] Created render batch with key {}", key);
        Ok(key)
    }

    fn write_descriptor_sets(
        &mut self,
        batch: &RenderBatch,
        shader_descriptor_bindings: Vec<ShaderDescriptorBinding>,
    ) {
        let set: &GfxDescriptorSet = batch.descriptor_set_ref();
        let (mut image_descriptor, mut sampler_descriptor) =
            if let Some(tex) = self.textures.get(&batch.tex_id()) {
                (
                    Some(Descriptor::Image(
                        tex.image_view.as_ref().unwrap(),
                        Layout::Undefined,
                    )),
                    Some(Descriptor::Sampler(tex.sampler.as_ref().unwrap())),
                )
            } else {
                (None, None)
            };

        let writes = {
            let mut writes = Vec::new();

            for (i, shader_desc_binding) in shader_descriptor_bindings.iter().enumerate() {
                match shader_desc_binding.ty {
                    DescriptorType::Buffer { ty: BufferDescriptorType::Uniform { .. }, ..} => {
                        writes.push(DescriptorSetWrite {
                            set,
                            binding: i as u32,
                            array_offset: 0,
                            descriptors: Some(Descriptor::Buffer(self.uniform_buffer.as_ref().unwrap(), buffer::SubRange { offset: 0, size: None })),
                        });
                    }
                    DescriptorType::Image { ty: ImageDescriptorType::Sampled { .. }, .. } => {
                        if image_descriptor.is_none() {
                            eprintln!("Failed to write to Sampled Image descriptor binding! Image descriptor was already in use or didn't exist!");
                            continue;
                        }

                        writes.push(DescriptorSetWrite {
                            set,
                            binding: i as u32,
                            array_offset: 0,
                            descriptors: image_descriptor.take(),
                        });
                    }
                    DescriptorType::Sampler => {
                        if sampler_descriptor.is_none() {
                            eprintln!("Failed to write to Sampler descriptor binding! Image descriptor was already in use or didn't exist!");
                            continue;
                        }

                        writes.push(DescriptorSetWrite {
                            set,
                            binding: i as u32,
                            array_offset: 0,
                            descriptors: sampler_descriptor.take(),
                        });
                    }
                    _ => panic!("Failed to write descriptor sets! Unhandled DescriptorType in ShaderDescriptorBinding")
                }
            }

            writes
        };

        unsafe {
            self.device.borrow().write_descriptor_sets(writes);
        }
    }

    /// Process some `RenderCommand`s, sorting them and producing batches that can be rendered.
    pub fn process_commands(&mut self, mut commands: Vec<RenderCommand>) -> Vec<RenderKey> {
        commands.sort_by(|a, b| a.key().cmp(&b.key()));

        // Process commands into batches
        let mut batch_keys: Vec<RenderKey> = Vec::new();
        let mut batch: Option<&mut RenderBatch> = None;

        for command in commands {
            let cmd_transparency = command.transparency;
            let cmd_layer = command.layer;
            let cmd_tex_id = command.tex_id;
            let cmd_shader_program_id = command.shader_program_id;

            // Flush the current batch if we are encountering new data
            if batch.is_some() {
                let (
                    batch_transparency,
                    batch_layer,
                    batch_shader_program_id,
                    batch_tex_id,
                    batch_key,
                ) = {
                    let b = batch.as_ref().unwrap();
                    (
                        b.transparency,
                        b.layer,
                        b.shader_program_id,
                        b.tex_id(),
                        b.key(),
                    )
                };

                if (batch_transparency != cmd_transparency)
                    || (batch_layer != cmd_layer)
                    || (batch_shader_program_id != cmd_shader_program_id)
                    || (batch_tex_id != cmd_tex_id)
                {
                    batch_keys.push(batch_key);
                    batch = None;
                }
            }

            // Begin a new batch if needed
            if batch.is_none() {
                let key = self
                    .create_render_batch(
                        cmd_transparency,
                        cmd_layer,
                        cmd_shader_program_id,
                        cmd_tex_id,
                    )
                    .unwrap();
                batch = Some(self.batches.get_mut(&key).unwrap());
            }

            if let Some(batch) = batch.as_mut() {
                batch.process_command(command);
            }
        }

        // Flush the current batch since we are done with all commands
        if batch.is_some() {
            batch_keys.push(batch.unwrap().key());
        }

        batch_keys
    }

    pub fn render(&mut self, scale_factor: f32, batch_keys: Vec<RenderKey>) {
        if self.surface.is_none() {
            panic!("Failed to render: Renderer surface was None!");
        }

        let surface_image = unsafe {
            match self.surface.as_mut().unwrap().acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.rebuild_swapchain();
                    return;
                }
            }
        };

        let framebuffer = unsafe {
            use std::borrow::Borrow;
            RefCell::borrow(&self.device)
                .create_framebuffer(
                    self.render_pass.as_ref().unwrap(),
                    std::iter::once(surface_image.borrow()),
                    Extent {
                        width: self.dimensions.width,
                        height: self.dimensions.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };

        let frame_idx = self.current_frame % self.frames_in_flight;

        unsafe {
            let fence = &self.frame_fences.as_ref().unwrap()[frame_idx];
            self.device
                .borrow()
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for frame fence!");
            self.device
                .borrow()
                .reset_fence(fence)
                .expect("Failed to reset frame fence!");
            self.command_pools.as_mut().unwrap()[frame_idx].reset(false);
        }

        let projection = glm::ortho(
            0.0,
            (self.dimensions.width as f32 / scale_factor) / self.render_scale,
            0.0,
            (self.dimensions.height as f32 / scale_factor) / self.render_scale,
            -1.0,
            100.0,
        );

        let ubo = UniformBufferObject {
            view: glm::Mat4::identity().into(),
            model: glm::Mat4::identity().into(),
            projection: projection.into(),
        };

        update_buffer(
            self.uniform_buffer_memory.as_ref().unwrap(),
            frame_idx,
            self.uniform_buffer_frame_size,
            self.device.clone(),
            &[ubo],
        );

        let final_command_buffer = unsafe {
            let command_buffer = &mut self.command_buffers[frame_idx];

            command_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);
            command_buffer.set_viewports(0, &[self.viewport.clone()]);
            command_buffer.set_scissors(0, &[self.viewport.rect]);

            command_buffer.begin_render_pass(
                self.render_pass.as_ref().unwrap(),
                &framebuffer,
                self.viewport.rect,
                &[command::ClearValue {
                    color: command::ClearColor {
                        float32: CLEAR_COLOR,
                    },
                }],
                command::SubpassContents::Inline,
            );

            // Record rendering of batches into command buffer
            for batch_key in batch_keys {
                self.render_batch(batch_key, frame_idx);
            }

            let command_buffer = &mut self.command_buffers[frame_idx];
            command_buffer.end_render_pass();
            command_buffer.finish();
            command_buffer
        };

        let submission = Submission {
            command_buffers: std::iter::once(&final_command_buffer),
            wait_semaphores: None,
            signal_semaphores: std::iter::once(&self.frame_semaphores.as_ref().unwrap()[frame_idx]),
        };

        unsafe {
            self.queue_group.queues[0].submit(
                submission,
                Some(&mut self.frame_fences.as_mut().unwrap()[frame_idx]),
            );
        }

        let result = unsafe {
            self.queue_group.queues[0].present_surface(
                self.surface.as_mut().unwrap(),
                surface_image,
                Some(&self.frame_semaphores.as_ref().unwrap()[frame_idx]),
            )
        };

        unsafe {
            self.device.borrow().destroy_framebuffer(framebuffer);
        }

        if result.is_err() {
            self.rebuild_swapchain();
        }

        self.current_frame += 1;
    }

    fn render_batch(&mut self, batch_key: RenderKey, frame_idx: usize) {
        let command_buffer = &mut self.command_buffers[frame_idx];

        let batch = self.batches.get_mut(&batch_key).unwrap();
        let mesh = batch.take_mesh();
        let indices_len = mesh.indices.len() as u32;

        update_buffer(
            batch.vertex_buffer_mem_ref(),
            frame_idx,
            batch.vertex_buffer.2,
            self.device.clone(),
            &mesh.vertices,
        );
        update_buffer(
            batch.index_buffer_mem_ref(),
            frame_idx,
            batch.index_buffer.2,
            self.device.clone(),
            &mesh.indices,
        );

        unsafe {
            let shader_program = match self.shader_programs.get(&batch.shader_program_id) {
                Some(s) => s,
                None => panic!("Failed to render batch: Referenced shader program did not exist!"),
            };

            command_buffer.bind_graphics_pipeline(shader_program.pipeline.as_ref().unwrap());

            // Bind buffers
            let vertex_buffer_offset = (frame_idx * batch.vertex_buffer.2) as u64;
            command_buffer.bind_vertex_buffers(
                0,
                Some((
                    batch.vertex_buffer_ref(),
                    buffer::SubRange {
                        offset: vertex_buffer_offset,
                        size: Some(batch.vertex_buffer.2 as u64),
                    },
                )),
            );

            let index_buffer_offset = (frame_idx * batch.index_buffer.2) as u64;
            command_buffer.bind_index_buffer(buffer::IndexBufferView {
                buffer: batch.index_buffer_ref(),
                range: buffer::SubRange {
                    offset: index_buffer_offset,
                    size: Some(batch.index_buffer.2 as u64),
                },
                index_type: IndexType::U32,
            });

            command_buffer.bind_graphics_descriptor_sets(
                shader_program.pipeline_layout.as_ref().unwrap(),
                0,
                vec![batch.descriptor_set_ref()],
                &[],
            );

            command_buffer.draw_indexed(0..indices_len, 0, 0..1);
        }
    }

    pub fn rebuild_swapchain(&mut self) {
        if self.surface.is_none() {
            panic!("Failed to rebuild swapchain: Renderer surface was None!");
        }
        let surface = self.surface.as_mut().unwrap();

        println!("Rebuilding swapchain.");

        let capabilities = surface.capabilities(&self.adapter.physical_device);
        let swap_config = window::SwapchainConfig::from_caps(
            &capabilities,
            self.surface_color_format,
            self.dimensions,
        );
        println!("swap_config: {:?}", swap_config);
        let extent = swap_config.extent.to_extent();

        unsafe {
            surface
                .configure_swapchain(&self.device.borrow(), swap_config)
                .expect("Can't create swapchain");
        }

        self.viewport.rect.w = extent.width as _;
        self.viewport.rect.h = extent.height as _;
    }

    pub fn create_gpu_texture(&mut self, id: GpuTextureId, w: u32, h: u32, pixels: &Vec<u8>) {
        let (texture_image, texture_memory, texture_view) = create_image(
            self.device.clone(),
            &self.adapter.physical_device,
            w,
            h,
            Format::Rgba8Srgb,
            img::Usage::TRANSFER_DST | img::Usage::SAMPLED,
            Aspects::COLOR,
        );

        let texture_sampler = unsafe {
            self.device
                .borrow()
                .create_sampler(&img::SamplerDesc::new(Filter::Nearest, WrapMode::Tile))
        }
        .expect("Failed to create sampler!");

        // Write data into texture
        {
            let row_alignment_mask = self
                .adapter
                .physical_device
                .limits()
                .optimal_buffer_copy_pitch_alignment as u32
                - 1;
            let image_stride: usize = 4;
            let row_pitch = (w * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
            let upload_size: u64 = (h * row_pitch).into();

            let (image_upload_buffer, image_upload_memory) = create_buffer(
                self.device.clone(),
                &self.adapter.physical_device,
                buffer::Usage::TRANSFER_SRC,
                Properties::CPU_VISIBLE,
                upload_size as usize,
            );

            unsafe {
                let mapping = self
                    .device
                    .borrow()
                    .map_memory(&image_upload_memory, Segment::ALL)
                    .unwrap();
                for y in 0..h as usize {
                    let row = &pixels
                        [y * (w as usize) * image_stride..(y + 1) * (w as usize) * image_stride];
                    std::ptr::copy_nonoverlapping(
                        row.as_ptr(),
                        mapping.offset(y as isize * row_pitch as isize),
                        w as usize * image_stride,
                    );
                }
                self.device
                    .borrow()
                    .flush_mapped_memory_ranges(std::iter::once((
                        &image_upload_memory,
                        Segment::ALL,
                    )))
                    .unwrap();
                self.device.borrow().unmap_memory(&image_upload_memory);
            };

            // Submit commands to transfer data
            let mut copy_fence = self
                .device
                .borrow()
                .create_fence(false)
                .expect("Failed to create texture copy fence!");
            unsafe {
                let mut cmd_buffer =
                    self.command_pools.as_mut().unwrap()[0].allocate_one(command::Level::Primary);
                cmd_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);

                let color_range = SubresourceRange {
                    aspects: Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                };

                let image_barrier = Barrier::Image {
                    states: (Access::empty(), Layout::Undefined)
                        ..(Access::TRANSFER_WRITE, Layout::TransferDstOptimal),
                    target: &texture_image,
                    families: None,
                    range: color_range.clone(),
                };

                cmd_buffer.pipeline_barrier(
                    PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
                    Dependencies::empty(),
                    &[image_barrier],
                );

                cmd_buffer.copy_buffer_to_image(
                    &image_upload_buffer,
                    &texture_image,
                    Layout::TransferDstOptimal,
                    &[BufferImageCopy {
                        buffer_offset: 0,
                        buffer_width: row_pitch / (image_stride as u32),
                        buffer_height: h,
                        image_layers: SubresourceLayers {
                            aspects: Aspects::COLOR,
                            level: 0,
                            layers: 0..1,
                        },
                        image_offset: Offset { x: 0, y: 0, z: 0 },
                        image_extent: Extent {
                            width: w,
                            height: h,
                            depth: 1,
                        },
                    }],
                );

                let image_barrier = Barrier::Image {
                    states: (Access::TRANSFER_WRITE, Layout::TransferDstOptimal)
                        ..(Access::SHADER_READ, Layout::ShaderReadOnlyOptimal),
                    target: &texture_image,
                    families: None,
                    range: color_range.clone(),
                };

                cmd_buffer.pipeline_barrier(
                    PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                    Dependencies::empty(),
                    &[image_barrier],
                );

                cmd_buffer.finish();

                self.queue_group.queues[0]
                    .submit_without_semaphores(Some(&cmd_buffer), Some(&mut copy_fence));

                self.device
                    .borrow()
                    .wait_for_fence(&copy_fence, !0)
                    .expect("Failed to wait for copy fence!");

                self.device.borrow().destroy_fence(copy_fence);
                self.device.borrow().destroy_buffer(image_upload_buffer);
                self.device.borrow().free_memory(image_upload_memory);
            }
        }

        let tex = GpuTexture {
            id,
            device: self.device.clone(),
            image: Some(texture_image),
            memory: Some(texture_memory),
            image_view: Some(texture_view),
            sampler: Some(texture_sampler),
            w,
            h,
        };

        self.textures.insert(id, tex);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        println!("Cleaning up Renderer");

        self.textures.clear();
        self.shader_programs.clear();
        self.batches.clear();

        let device = self.device.borrow();
        unsafe {
            self.instance.destroy_surface(self.surface.take().unwrap());

            device.destroy_render_pass(self.render_pass.take().unwrap());

            device.destroy_buffer(self.uniform_buffer.take().unwrap());
            device.free_memory(self.uniform_buffer_memory.take().unwrap());

            for semaphore in self.frame_semaphores.take().unwrap() {
                device.destroy_semaphore(semaphore);
            }

            for fence in self.frame_fences.take().unwrap() {
                device.destroy_fence(fence);
            }

            device.wait_idle().unwrap();
            for mut command_pool in self.command_pools.take().unwrap() {
                command_pool.reset(true);
                device.destroy_command_pool(command_pool);
            }
        }
    }
}

fn create_buffer(
    device: GfxDeviceHandle,
    physical_device: &dyn PhysicalDevice<backend::Backend>,
    usage: buffer::Usage,
    properties: Properties,
    buffer_len: usize,
) -> (GfxBuffer, GfxMemory) {
    assert_ne!(buffer_len, 0);

    // Get a list of available memory types
    let memory_types = physical_device.memory_properties().memory_types;

    // First create a buffer
    let mut buffer = unsafe {
        device
            .borrow()
            .create_buffer(buffer_len as u64, usage)
            .unwrap()
    };

    // Get the memory requirements for this buffer
    let mem_requirements = unsafe { device.borrow().get_buffer_requirements(&buffer) };

    // Filter through memory types to find one that is appropriate.
    let upload_type = memory_types
        .iter()
        .enumerate()
        .find(|(id, ty)| {
            let type_supported = mem_requirements.type_mask & (1_u64 << id) != 0;
            type_supported && ty.properties.contains(properties)
        })
        .map(|(id, _ty)| MemoryTypeId(id))
        .expect("Could not find appropriate buffer memory type!");

    // Now allocate the memory and bind our buffer to it.
    let buffer_memory = unsafe {
        device
            .borrow()
            .allocate_memory(upload_type, mem_requirements.size)
    }
    .unwrap();

    unsafe {
        device
            .borrow()
            .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
    }
    .unwrap();

    (buffer, buffer_memory)
}

fn create_vertex_buffer(
    device: GfxDeviceHandle,
    physical_device: &dyn PhysicalDevice<backend::Backend>,
    mesh: &[Vertex],
    frames_in_flight: usize,
) -> (GfxBuffer, GfxMemory, usize) {
    let stride = std::mem::size_of::<Vertex>();
    let buffer_frame_len = (MAX_BATCH_VERTICES as usize) * stride;

    let (buffer, buffer_memory) = create_buffer(
        device.clone(),
        physical_device,
        buffer::Usage::VERTEX | buffer::Usage::TRANSFER_DST,
        Properties::CPU_VISIBLE,
        buffer_frame_len * frames_in_flight,
    );

    update_buffer(&buffer_memory, 0, buffer_frame_len, device.clone(), mesh);

    (buffer, buffer_memory, buffer_frame_len)
}

fn create_index_buffer(
    device: GfxDeviceHandle,
    physical_device: &dyn PhysicalDevice<backend::Backend>,
    indices: &[u32],
    frames_in_flight: usize,
) -> (GfxBuffer, GfxMemory, usize) {
    let stride = std::mem::size_of::<u32>();
    let buffer_frame_len = (MAX_BATCH_INDICES as usize) * stride;

    let (index_buffer, index_buffer_memory) = create_buffer(
        device.clone(),
        physical_device,
        buffer::Usage::INDEX | buffer::Usage::TRANSFER_DST,
        Properties::CPU_VISIBLE,
        buffer_frame_len * frames_in_flight,
    );

    update_buffer(
        &index_buffer_memory,
        0,
        buffer_frame_len,
        device.clone(),
        indices,
    );

    (index_buffer, index_buffer_memory, buffer_frame_len)
}

fn create_uniform_buffer(
    device: GfxDeviceHandle,
    physical_device: &dyn PhysicalDevice<backend::Backend>,
    ubo: UniformBufferObject,
    frames_in_flight: usize,
) -> (GfxBuffer, GfxMemory, usize) {
    let buffer_frame_len = std::mem::size_of::<UniformBufferObject>();

    let (buffer, buffer_memory) = create_buffer(
        device.clone(),
        physical_device,
        buffer::Usage::UNIFORM | buffer::Usage::TRANSFER_DST,
        Properties::CPU_VISIBLE,
        buffer_frame_len * frames_in_flight,
    );

    update_buffer(&buffer_memory, 0, buffer_frame_len, device.clone(), &[ubo]);

    (buffer, buffer_memory, buffer_frame_len)
}

fn update_buffer<T: Copy>(
    buffer_memory: &GfxMemory,
    frame_idx: usize,
    buffer_frame_size: usize,
    device: GfxDeviceHandle,
    data: &[T],
) {
    let data_len = data.len() as u64 * std::mem::size_of::<T>() as u64;
    let buffer_offset = (frame_idx * buffer_frame_size) as u64;

    let device = device.borrow();
    unsafe {
        let segment = Segment {
            offset: buffer_offset,
            size: Some(data_len),
        };
        let mapping = device.map_memory(buffer_memory, segment.clone()).unwrap();
        std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, mapping, data_len as usize);
        device
            .flush_mapped_memory_ranges(std::iter::once((buffer_memory, segment)))
            .unwrap();
        device.unmap_memory(buffer_memory);
    }
}

fn create_image(
    device: GfxDeviceHandle,
    physical_device: &dyn PhysicalDevice<backend::Backend>,
    width: u32,
    height: u32,
    format: Format,
    usage: img::Usage,
    aspects: Aspects,
) -> (GfxImage, GfxMemory, GfxImageView) {
    // Get a list of available memory types
    let memory_types = physical_device.memory_properties().memory_types;

    let kind = img::Kind::D2(width, height, 1, 1);

    let mut image = unsafe {
        device.borrow().create_image(
            kind,
            1,
            format,
            img::Tiling::Optimal,
            usage,
            ViewCapabilities::empty(),
        )
    }
    .expect("Failed to create unbound image!");

    let requirements = unsafe { device.borrow().get_image_requirements(&image) };

    let device_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, memory_type)| {
            requirements.type_mask & (1_u64 << id) != 0
                && memory_type.properties.contains(Properties::DEVICE_LOCAL)
        })
        .unwrap()
        .into();

    let image_memory = unsafe {
        device
            .borrow()
            .allocate_memory(device_type, requirements.size)
    }
    .expect("Failed to allocate image memory!");

    unsafe {
        device
            .borrow()
            .bind_image_memory(&image_memory, 0, &mut image)
    }
    .expect("Failed to bind image memory!");

    let image_view = unsafe {
        device.borrow().create_image_view(
            &image,
            img::ViewKind::D2,
            format,
            Swizzle::NO,
            img::SubresourceRange {
                aspects,
                levels: 0..1,
                layers: 0..1,
            },
        )
    }
    .expect("Failed to create image view!");

    (image, image_memory, image_view)
}

fn create_render_pass(
    device: GfxDeviceHandle,
    surface_color_fmt: Format,
    _depth_fmt: Format,
) -> GfxRenderPass {
    let color_attachment = Attachment {
        format: Some(surface_color_fmt),
        samples: 1,
        ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::Present,
    };

    let subpass = SubpassDesc {
        colors: &[(0, Layout::ColorAttachmentOptimal)],
        depth_stencil: None,
        inputs: &[],
        resolves: &[],
        preserves: &[],
    };

    unsafe {
        device
            .borrow()
            .create_render_pass(&[color_attachment], &[subpass], &[])
    }
    .expect("Failed to create render pass!")
}

fn create_pipeline(
    device: GfxDeviceHandle,
    vert_shader: &GfxShaderModule,
    frag_shader: &GfxShaderModule,
    render_pass: &GfxRenderPass,
    pipeline_layout: &GfxPipelineLayout,
    primitive: Primitive,
) -> GfxGraphicsPipeline {
    let vs_entry = EntryPoint::<backend::Backend> {
        entry: "main",
        module: vert_shader,
        specialization: Specialization::default(),
    };

    let fs_entry = EntryPoint::<backend::Backend> {
        entry: "main",
        module: frag_shader,
        specialization: Specialization::default(),
    };

    let shader_entries = GraphicsShaderSet {
        vertex: vs_entry,
        hull: None,
        domain: None,
        geometry: None,
        fragment: Some(fs_entry),
    };

    let subpass = Subpass {
        index: 0,
        main_pass: render_pass,
    };

    let mut pipeline_desc = GraphicsPipelineDesc::new(
        shader_entries,
        primitive,
        pso::Rasterizer::FILL,
        pipeline_layout,
        subpass,
    );

    pipeline_desc.blender.targets.push(pso::ColorBlendDesc {
        mask: pso::ColorMask::ALL,
        blend: Some(pso::BlendState::ALPHA),
    });

    // Let our pipeline know about the vertex buffers we are going to use
    pipeline_desc.vertex_buffers.push(VertexBufferDesc {
        binding: 0,
        stride: std::mem::size_of::<Vertex>() as u32,
        rate: pso::VertexInputRate::Vertex,
    });

    // Let our pipeline know about our vertex attributes
    // Position
    pipeline_desc.attributes.push(AttributeDesc {
        location: 0,
        binding: 0,
        element: Element {
            format: Format::Rgb32Sfloat,
            offset: 0,
        },
    });

    // Color
    pipeline_desc.attributes.push(AttributeDesc {
        location: 1,
        binding: 0,
        element: Element {
            format: Format::Rgba32Sfloat,
            offset: 12,
        },
    });

    // UV
    pipeline_desc.attributes.push(AttributeDesc {
        location: 2,
        binding: 0,
        element: Element {
            format: Format::Rg32Sfloat,
            offset: 28,
        },
    });

    unsafe {
        device
            .borrow()
            .create_graphics_pipeline(&pipeline_desc, None)
    }
    .expect("Failed to create graphics pipeline!")
}

fn create_render_program(
    device: GfxDeviceHandle,
    render_pass: &GfxRenderPass,
    vertex_shader_path: &str,
    fragment_shader_path: &str,
    shader_descriptor_bindings: Vec<ShaderDescriptorBinding>,
    primitive: Primitive,
) -> RenderProgram {
    // Load shaders
    let vert_shader = unsafe {
        let mut file = File::open(vertex_shader_path).expect("Failed to open vertex shader file!");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .expect("Failed to read shader file into buffer!");

        let spirv = pso::read_spirv(Cursor::new(&bytes[..])).expect("Failed to read spirv shader!");

        device
            .borrow()
            .create_shader_module(&spirv)
            .expect("Failed to create shader module!")
    };
    let frag_shader = unsafe {
        let mut file =
            File::open(fragment_shader_path).expect("Failed to open fragment shader file!");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .expect("Failed to read shader file into buffer!");

        let spirv = pso::read_spirv(Cursor::new(&bytes[..])).expect("Failed to read spirv shader!");

        device
            .borrow()
            .create_shader_module(&spirv)
            .expect("Failed to create shader module!")
    };

    let (bindings, descriptor_ranges) = {
        let mut bindings = Vec::new();
        let mut descriptor_ranges = Vec::new();

        for (i, shader_desc_binding) in shader_descriptor_bindings.iter().enumerate() {
            bindings.push(DescriptorSetLayoutBinding {
                binding: i as u32,
                ty: shader_desc_binding.ty,
                count: 1,
                stage_flags: shader_desc_binding.stage_flags,
                immutable_samplers: false,
            });

            descriptor_ranges.push(DescriptorRangeDesc {
                ty: shader_desc_binding.ty,
                count: MAX_DESCRIPTOR_SETS,
            })
        }

        (bindings, descriptor_ranges)
    };

    // Create the descriptor set layout
    let descriptor_set_layout =
        unsafe { device.borrow().create_descriptor_set_layout(&bindings, &[]) }
            .expect("Failed to create descriptor set layout!");

    // Create the descriptor pool
    let descriptor_pool = unsafe {
        device.borrow().create_descriptor_pool(
            MAX_DESCRIPTOR_SETS,
            &descriptor_ranges,
            pso::DescriptorPoolCreateFlags::empty(),
        )
    }
    .expect("Failed to create descriptor pool!");

    // Create the pipeline layout from the descriptor set layout
    let pipeline_layout = unsafe {
        device
            .borrow()
            .create_pipeline_layout(vec![&descriptor_set_layout], &[])
    }
    .expect("Failed to create pipeline layout!");

    // Create the pipeline
    let pipeline = create_pipeline(
        device.clone(),
        &vert_shader,
        &frag_shader,
        &render_pass,
        &pipeline_layout,
        primitive,
    );

    RenderProgram {
        device,
        vert_shader: Some(vert_shader),
        frag_shader: Some(frag_shader),
        pipeline: Some(pipeline),
        pipeline_layout: Some(pipeline_layout),
        descriptor_pool: Some(descriptor_pool),
        descriptor_set_layout: Some(descriptor_set_layout),
        shader_descriptor_bindings,
    }
}
