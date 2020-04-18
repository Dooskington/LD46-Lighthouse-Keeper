use crate::game::{physics::*, Point2d, *};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum WorkstationEvent {
    RaiseTemperature,
    LowerTemperature,
}

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
pub struct WorkstationSystem {
    workstation_event_reader: Option<ReaderId<WorkstationEvent>>,
}

impl<'a> System<'a> for WorkstationSystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<WorkstationEvent>>,
        WriteExpect<'a, WorkstationState>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.workstation_event_reader = Some(
            world
                .fetch_mut::<EventChannel<WorkstationEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (workstation_events, mut workstation): Self::SystemData) {
        for event in workstation_events.read(&mut self.workstation_event_reader.as_mut().unwrap()) {
            match event {
                WorkstationEvent::RaiseTemperature => {
                    workstation.room_temperature += 0.25;
                }
                WorkstationEvent::LowerTemperature => {
                    workstation.room_temperature -= 0.25;
                }
            }
        }

        workstation.room_temperature = workstation.room_temperature.min(alien::MAX_TEMPERATURE).max(alien::MIN_TEMPERATURE);
    }
}

#[derive(Default)]
pub struct WorkstationInfoRenderSystem;

impl<'a> System<'a> for WorkstationInfoRenderSystem {
    type SystemData = (Write<'a, RenderState>, ReadExpect<'a, WorkstationState>);

    fn run(&mut self, (mut render, workstation): Self::SystemData) {
        render.bind_layer(layers::LAYER_UI);
        render.bind_transparency(Transparency::Transparent);
        render.bind_texture(resources::TEX_FONT);

        render.bind_color(COLOR_WHITE);
        render.text(
            8.0,
            175.0,
            8,
            16,
            1.1,
            &format!("ROOM TEMP: {:.1}", workstation.room_temperature),
        );
    }
}
