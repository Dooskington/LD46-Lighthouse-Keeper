use crate::game::{
    physics::{ColliderComponent, PhysicsState, RigidbodyComponent},
    transform::TransformComponent,
    Point2d, Vector2d,
};
use gfx::{
    color::*,
    renderer::{Renderable, TextureId, Transparency},
    sprite::*,
    Point2f, Vector2f,
};
use ncollide2d::{procedural::Polyline, shape::Shape, transformation::ToPolyline};
use specs::prelude::*;
use std::marker::PhantomData;

#[derive(Default)]
pub struct RenderState {
    commands: Vec<gfx::renderer::RenderCommand>,
    bound_transparency: Transparency,
    bound_texture_id: TextureId,
    bound_layer: u8,
    bound_color: Color,
}

impl RenderState {
    pub fn new() -> Self {
        RenderState {
            ..Default::default()
        }
    }

    pub fn bind_transparency(&mut self, val: Transparency) {
        self.bound_transparency = val;
    }

    pub fn bind_texture(&mut self, val: TextureId) {
        self.bound_texture_id = val;
    }

    pub fn bind_layer(&mut self, val: u8) {
        self.bound_layer = val;
    }

    pub fn bind_color(&mut self, val: Color) {
        self.bound_color = val;
    }

    pub fn sprite(
        &mut self,
        x: f32,
        y: f32,
        pivot: Point2f,
        scale: Vector2f,
        region: SpriteRegion,
    ) {
        self.commands.push(gfx::renderer::RenderCommand {
            transparency: self.bound_transparency,
            shader_program_id: 1,
            tex_id: self.bound_texture_id,
            layer: self.bound_layer,
            data: Renderable::Sprite {
                x,
                y,
                pivot,
                scale,
                color: self.bound_color,
                region,
            },
        });
    }

    pub fn text(&mut self, x: f32, y: f32, w: u32, h: u32, scale: f32, text: &str) {
        let cols: u32 = 16;
        for (i, c) in text.chars().enumerate() {
            let ascii: u8 = c as u8;
            let sprite_col: u32 = ascii as u32 % cols;
            let sprite_row: u32 = ascii as u32 / cols;
            self.commands.push(gfx::renderer::RenderCommand {
                transparency: self.bound_transparency,
                shader_program_id: 1,
                tex_id: self.bound_texture_id,
                layer: self.bound_layer,
                data: Renderable::Sprite {
                    x: x + (i as f32 * (w as f32 * scale)),
                    y: y,
                    pivot: Point2f::origin(),
                    scale: Vector2f::new(scale, scale),
                    color: self.bound_color,
                    region: SpriteRegion {
                        x: sprite_col * w,
                        y: sprite_row * h,
                        w,
                        h,
                    },
                },
            });
        }
    }

    pub fn textured_quad(
        &mut self,
        bl: (f32, f32),
        br: (f32, f32),
        tl: (f32, f32),
        tr: (f32, f32),
    ) {
        self.commands.push(gfx::renderer::RenderCommand {
            transparency: self.bound_transparency,
            shader_program_id: 1,
            tex_id: self.bound_texture_id,
            layer: self.bound_layer,
            data: Renderable::Quad {
                bl,
                br,
                tl,
                tr,
                color: self.bound_color,
            },
        });
    }

    pub fn clear_commands(&mut self) {
        self.bound_transparency = Transparency::default();
        self.bound_texture_id = 0;
        self.bound_layer = 0;
        self.bound_color = Color::default();
        self.commands.clear();
    }

    pub fn commands(&mut self) -> Vec<gfx::renderer::RenderCommand> {
        self.commands.clone()
    }
}

#[derive(Debug)]
pub struct SpriteComponent {
    pub region: SpriteRegion,
    pub spritesheet_tex_id: TextureId,
    pub pivot: Point2f,
    pub pivot_pixels: Point2f,
    pub color: Color,
    pub layer: u8,
    pub transparency: Transparency,
}

impl SpriteComponent {
    pub fn new(
        region: SpriteRegion,
        spritesheet: TextureId,
        pivot: Point2f,
        color: Color,
        layer: u8,
        transparency: Transparency,
    ) -> Self {
        let pivot_pixels = Point2f::new(pivot.x * region.w as f32, pivot.y * region.h as f32);

        SpriteComponent {
            region,
            spritesheet_tex_id: spritesheet,
            pivot,
            pivot_pixels,
            color,
            layer,
            transparency,
        }
    }
}

impl Component for SpriteComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct SpriteRenderSystem;

impl<'a> System<'a> for SpriteRenderSystem {
    type SystemData = (
        ReadExpect<'a, PhysicsState>,
        Write<'a, RenderState>,
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, SpriteComponent>,
        ReadStorage<'a, RigidbodyComponent>,
    );

    fn run(&mut self, (physics, mut render, transforms, sprites, rigidbodies): Self::SystemData) {
        for (transform, sprite, rigidbody) in (&transforms, &sprites, (&rigidbodies).maybe()).join()
        {
            let (x, y) = if let Some(_) = rigidbody {
                let x = (transform.position.x * physics.lerp)
                    + (transform.last_position.x * (1.0 - physics.lerp));
                let y = (transform.position.y * physics.lerp)
                    + (transform.last_position.y * (1.0 - physics.lerp));
                (x, y)
            } else {
                (transform.position.x, transform.position.y)
            };

            render.bind_transparency(sprite.transparency);
            render.bind_texture(sprite.spritesheet_tex_id);
            render.bind_color(sprite.color);
            render.bind_layer(sprite.layer);
            render.sprite(
                x as f32,
                y as f32,
                sprite.pivot_pixels,
                transform.scale,
                sprite.region,
            );
        }
    }
}
