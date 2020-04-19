use crate::game::*;
use specs::prelude::*;

pub struct ActivityComponent {
    activity: Activity,
}

impl ActivityComponent {
    pub fn new(activity: Activity) -> Self {
        ActivityComponent {
            activity,
        }
    }
}

impl Component for ActivityComponent {
    type Storage = VecStorage<Self>;
}

pub struct Activity {
    pub name: String,
    pub hours_required: i32,
    pub event: GameEvent,
    // TODO required_condition?
}

#[derive(Default)]
pub struct ActivitySystem {
    game_event_reader: Option<ReaderId<GameEvent>>,
    on_clicked_event_reader: Option<ReaderId<OnClickedEvent>>,
    activities: Vec<Activity>,
}

impl<'a> System<'a> for ActivitySystem {
    type SystemData = (ReadExpect<'a, EventChannel<OnClickedEvent>>, WriteExpect<'a, EventChannel<GameEvent>>, WriteStorage<'a, ActivityComponent>);

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.game_event_reader = Some(
            world
                .fetch_mut::<EventChannel<GameEvent>>()
                .register_reader()
            );

        self.on_clicked_event_reader = Some(
            world
                .fetch_mut::<EventChannel<OnClickedEvent>>()
                .register_reader()
            );

        self.activities = create_activities();
    }

    fn run(&mut self, (on_clicked_events, mut game_events, mut activity_comps): Self::SystemData) {
        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::RefreshActivities => {
                    println!("Refreshing activities (remove this)");

                    // TODO
                    // Need to create and store activities somewhere
                },
                _ => {},
            }
        }

        for event in on_clicked_events.read(&mut self.on_clicked_event_reader.as_mut().unwrap()) {
            if let Some(activity) = activity_comps.get(event.ent) {
                game_events.single_write(GameEvent::ProgressTime { hours: activity.activity.hours_required });
                game_events.single_write(activity.activity.event);
            }
        }
    }
}

pub fn create_activities() -> Vec<Activity> {
    // TODO
    vec![
    ]
}

#[derive(Default)]
pub struct ActivityInfoRenderSystem;

impl<'a> System<'a> for ActivityInfoRenderSystem {
    type SystemData = (Write<'a, RenderState>, ReadStorage<'a, TransformComponent>, ReadStorage<'a, ActivityComponent>);

    fn run(&mut self, (mut render, transforms, activity_comps): Self::SystemData) {
        for (transform, activity) in (&transforms, &activity_comps).join() {
            let x = transform.position.x as f32 + 16.0;
            let y = transform.position.y as f32 + 12.0;
            render.bind_transparency(Transparency::Opaque);
            render.bind_layer(layers::LAYER_UI);
            render.bind_texture(resources::TEX_FONT);
            render.bind_color(COLOR_BLACK);
            render.text(
                x,
                y,
                8,
                16,
                1.2,
                &activity.activity.name,
            );

            let hours_text =  if activity.activity.hours_required == 1 {
                format!("{} hour", activity.activity.hours_required)
            } else {
                format!("{} hours", activity.activity.hours_required)
            };

            render.text(
                x,
                y + 20.0,
                8,
                16,
                1.0,
                &hours_text,
            );
        }
    }
}
