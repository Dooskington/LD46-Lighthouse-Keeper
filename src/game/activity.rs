use crate::game::*;
use specs::prelude::*;

pub struct Activity {
    name: String,
    position: Vector2d,
    event: GameEvent,
}

#[derive(Default)]
pub struct ActivitySystem {
    game_event_reader: Option<ReaderId<GameEvent>>,
}

impl<'a> System<'a> for ActivitySystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<GameEvent>>,
        WriteExpect<'a, StatsState>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.game_event_reader = Some(
            world
                .fetch_mut::<EventChannel<GameEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (game_events, _): Self::SystemData) {
        // TODO

        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::RefreshActivities => {
                    println!("Refreshing activities (remove this)");
                },
                _ => {},
            }
        }
    }
}