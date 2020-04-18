use crate::game::{*, physics::*, Point2d};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;

pub struct WorkstationState {
    pub room_temperature: f32,
}

impl WorkstationState {
    pub fn new() -> Self {
        WorkstationState {
            room_temperature: 10.0,
        }
    }
}

#[derive(Default)]
pub struct WorkstationSystem;

impl<'a> System<'a> for WorkstationSystem {
    type SystemData = (
        WriteExpect<'a, WorkstationState>,
    );

    fn run(&mut self, mut workstation: Self::SystemData) {
        // intercept workstation events like temperature modification
    }
}

#[derive(Default)]
pub struct WorkstationInfoRenderSystem;

impl<'a> System<'a> for WorkstationInfoRenderSystem {
    type SystemData = (
        Write<'a, RenderState>,
        ReadExpect<'a, WorkstationState>,
    );

    fn run(&mut self, (mut render, workstation): Self::SystemData) {
        render.bind_layer(layers::LAYER_UI);
        render.bind_transparency(Transparency::Transparent);
        render.bind_texture(resources::TEX_FONT);

        render.bind_color(COLOR_WHITE);
        render.text(8.0, 175.0, 8, 16, 1.1, &format!("ROOM TEMP: {:.1}", workstation.room_temperature));
    }
}