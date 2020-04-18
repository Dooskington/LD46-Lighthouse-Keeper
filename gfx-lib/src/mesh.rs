use crate::{color::Color, sprite::SpriteRegion, Point2f, Vector2f};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

pub fn add_sprite(
    mesh: &mut Mesh,
    x: f32,
    y: f32,
    origin: Point2f,
    scale: Vector2f,
    color: Color,
    region: SpriteRegion,
    spritesheet_width: u32,
    spritesheet_height: u32,
) {
    let vertex_count: u32 = mesh.vertices.len() as u32;
    let color: [f32; 4] = color.data();

    // TODO
    // need to pre compute these uvs

    let u: f32 = region.x as f32 / spritesheet_width as f32;
    let v: f32 = region.y as f32 / spritesheet_height as f32;
    let u_width: f32 = region.w as f32 / spritesheet_width as f32;
    let v_height: f32 = region.h as f32 / spritesheet_height as f32;

    // Offset the render position based on the sprite origin
    let x = x - (origin.x as f32 * scale.x);
    let y = y - (origin.y as f32 * scale.y);
    let w = region.w as f32 * scale.x;
    let h = region.h as f32 * scale.y;

    let new_vertices: [Vertex; 4] = [
        // Top left
        Vertex {
            position: [x, y, 0.0],
            color,
            uv: [u, v],
        },
        // Top right
        Vertex {
            position: [x + w, y, 0.0],
            color,
            uv: [u + u_width, v],
        },
        // Bottom right
        Vertex {
            position: [x + w, y + h, 0.0],
            color,
            uv: [u + u_width, v + v_height],
        },
        // Bottom left
        Vertex {
            position: [x, y + h, 0.0],
            color,
            uv: [u, v + v_height],
        },
    ];

    let new_indices: [u32; 6] = [
        vertex_count,
        vertex_count + 1,
        vertex_count + 2,
        vertex_count + 2,
        vertex_count + 3,
        vertex_count,
    ];

    mesh.vertices.extend_from_slice(&new_vertices);
    mesh.indices.extend_from_slice(&new_indices);
}

pub fn add_quad(
    mesh: &mut Mesh,
    bl: (f32, f32),
    br: (f32, f32),
    tl: (f32, f32),
    tr: (f32, f32),
    color: Color,
) {
    let vertex_count: u32 = mesh.vertices.len() as u32;
    let color: [f32; 4] = color.data();

    let new_vertices: [Vertex; 4] = [
        // Top left
        Vertex {
            position: [tl.0, tl.1, 0.0],
            color,
            uv: [0.0, 0.0],
        },
        // Top right
        Vertex {
            position: [tr.0, tr.1, 0.0],
            color,
            uv: [1.0, 0.0],
        },
        // Bottom right
        Vertex {
            position: [br.0, br.1, 0.0],
            color,
            uv: [1.0, 1.0],
        },
        // Bottom left
        Vertex {
            position: [bl.0, bl.1, 0.0],
            color,
            uv: [0.0, 1.0],
        },
    ];

    let new_indices: [u32; 6] = [
        vertex_count,
        vertex_count + 1,
        vertex_count + 2,
        vertex_count + 2,
        vertex_count + 3,
        vertex_count,
    ];

    mesh.vertices.extend_from_slice(&new_vertices);
    mesh.indices.extend_from_slice(&new_indices);
}
