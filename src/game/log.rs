use crate::game::{physics::*, Point2d, *};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use rand::Rng;
use specs::prelude::*;

#[derive(Clone)]
pub struct LogEvent {
    pub message: String,
    pub color: Color,
}

#[derive(Default)]
pub struct LogState {
    pub logs: Vec<LogEvent>,
}

#[derive(Default)]
pub struct LogSystem {
    log_event_reader: Option<ReaderId<LogEvent>>,
}

impl<'a> System<'a> for LogSystem {
    type SystemData = (
        WriteExpect<'a, RenderState>,
        WriteExpect<'a, LogState>,
        ReadExpect<'a, EventChannel<LogEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.log_event_reader = Some(
            world
                .fetch_mut::<EventChannel<LogEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (mut render, mut log_state, log_events): Self::SystemData) {
        for event in log_events.read(&mut self.log_event_reader.as_mut().unwrap()) {
           log_state.logs.insert(0, event.clone());

           if log_state.logs.len() > 20 {
               log_state.logs.pop();
           }
        }

        let pos_x = 640.0;
        let pos_y = 700.0;
        render.bind_transparency(Transparency::Opaque);
        render.bind_layer(layers::LAYER_UI);
        render.bind_texture(resources::TEX_FONT);
        for (i, log) in log_state.logs.iter().enumerate() {
            let color_lerp_percent = i as f32 / 32.0;
            let color = color_lerp(log.color, COLOR_WHITE, color_lerp_percent);
            render.bind_color(color);

            render.text(pos_x, pos_y - (i as f32 * 16.0), 8, 16, 1.0, &log.message);
        }
    }
}
