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

#[derive(Clone)]
pub struct Activity {
    pub name: String,
    pub hours_required: i32,
    pub event: GameEvent,
    pub effects: Vec<StatEffect>,
    pub is_repeatable: bool,
    // TODO required_conditions?
}

#[derive(Default)]
pub struct ActivityState {
    pub activities: Vec<Activity>,
    pub is_rebuild_required: bool,
}

impl ActivityState {
    pub fn new() -> Self {
        ActivityState {
            activities: create_activities(),
            is_rebuild_required: false,
        }
    }
}

#[derive(Default)]
pub struct ActivitySystem {
    game_event_reader: Option<ReaderId<GameEvent>>,
    on_clicked_event_reader: Option<ReaderId<OnClickedEvent>>,
}

impl<'a> System<'a> for ActivitySystem {
    type SystemData = (Entities<'a>, WriteExpect<'a, ActivityState>, ReadExpect<'a, EventChannel<OnClickedEvent>>, WriteExpect<'a, EventChannel<GameEvent>>, ReadStorage<'a, ActivityComponent>);

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
    }

    fn run(&mut self, (ents, mut activity_state, on_clicked_events, mut game_events, activity_comps): Self::SystemData) {
        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::RefreshActivities => {
                    println!("Refreshing activities (remove this)");

                    activity_state.is_rebuild_required = true;
                },
                GameEvent::ActivityGoFishing => {
                    println!("You attempt to catch some fish.");
                }
                GameEvent::ActivityPerformMaintenance => {
                    println!("You maintain some fixtures around the lighthouse.");
                }
                GameEvent::ActivityPrayToJand => {
                    println!("You pray to Jand, for protection and fortune. Perhaps it will pity you.");
                }
                GameEvent::ActivityDrinkAlcobev => {
                    println!("You have a drink to dull the pain.");
                }
                _ => {},
            }
        }

        for event in on_clicked_events.read(&mut self.on_clicked_event_reader.as_mut().unwrap()) {
            if let Some(comp) = activity_comps.get(event.ent) {
                game_events.single_write(GameEvent::HandleStatEffects { effects: comp.activity.effects.clone() });
                game_events.single_write(GameEvent::ProgressTime { hours: comp.activity.hours_required });
                game_events.single_write(comp.activity.event.clone());

                if !comp.activity.is_repeatable {
                    ents.delete(event.ent).unwrap();
                }
            }
        }
    }
}

pub fn create_activities() -> Vec<Activity> {
    vec![
        Activity {
            name: String::from("Go Fishing"),
            hours_required: 2,
            event: GameEvent::ActivityGoFishing,
            effects: vec![StatEffect::Add { stat: Stat::Food, amount: 1 }],
            is_repeatable: true,
        },
        Activity {
            name: String::from("Perform Maintenance"),
            hours_required: 3,
            event: GameEvent::ActivityPerformMaintenance,
            effects: vec![StatEffect::Subtract { stat: Stat::Parts, amount: 1 }],
            is_repeatable: false,
        },
        Activity {
            name: String::from("Pray To Jand"),
            hours_required: 1,
            event: GameEvent::ActivityPrayToJand,
            effects: vec![StatEffect::Add { stat: Stat::Sanity, amount: 1 }],
            is_repeatable: false,
        },
        Activity {
            name: String::from("Have a Drink"),
            hours_required: 1,
            event: GameEvent::ActivityDrinkAlcobev,
            effects: vec![StatEffect::Add { stat: Stat::Sanity, amount: 1 }, StatEffect::Subtract { stat: Stat::Food, amount: 1 }],
            is_repeatable: false,
        },
    ]
}

pub fn create_activity_ents(world: &mut World) {
    let collision_groups = CollisionGroups::new();
    let button_bg_sprite_region = SpriteRegion {
        x: 0,
        y: 160,
        w: 160,
        h: 96,
    };

    world.delete_all();
    world.maintain();

    let activities = world.read_resource::<ActivityState>().activities.clone();
    let layout_pos_x = 340.0;
    let mut layout_pos_y = 350.0;
    for activity in activities {
        world
            .create_entity()
            .with(TransformComponent::new(
            Vector2d::new(layout_pos_x, layout_pos_y),
            Vector2f::new(1.5, 1.0),
            ))
            .with(ColliderComponent::new(
                Cuboid::new(Vector2d::new(
                    (240.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
                    (96.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
                )),
                Vector2d::zeros(),
                collision_groups,
                0.0,
            ))
            .with(ActivityComponent::new(activity))
            .with(ClickableComponent::new())
            .with(SpriteComponent::new(
                button_bg_sprite_region,
                resources::TEX_SPRITESHEET_UI,
                Point2f::origin(),
                COLOR_WHITE,
                layers::LAYER_BUTTONS,
                Transparency::Opaque,
            ))
            .build();

        layout_pos_y += 100.0;
    }

    /*
    let collision_groups = CollisionGroups::new();
        world
            .create_entity()
            .with(TransformComponent::new(
                Vector2d::new(340.0, 350.0),
                Vector2f::new(1.5, 1.0),
            ))
            .with(ColliderComponent::new(
                Cuboid::new(Vector2d::new(
                    (240.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
                    (96.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
                )),
                Vector2d::zeros(),
                collision_groups,
                0.0,
            ))
            .with(ActivityComponent::new(Activity {
                name: String::from("Go Fishing"),
                hours_required: 2,
                event: GameEvent::ActivityGoFishing,
                effects: vec![StatEffect::Add { stat: Stat::Food, amount: 1 }],
                is_repeatable: true,
            }))
            .with(ClickableComponent::new())
            .with(SpriteComponent::new(
                SpriteRegion {
                    x: 0,
                    y: 160,
                    w: 160,
                    h: 96,
                },
                resources::TEX_SPRITESHEET_UI,
                Point2f::origin(),
                COLOR_WHITE,
                layers::LAYER_BUTTONS,
                Transparency::Opaque,
            ))
            .build();
    */

    world.write_resource::<ActivityState>().is_rebuild_required = false;
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

            let effect_text = {
                let mut str = String::new();

                let count = activity.activity.effects.len();
                for (i, effect) in activity.activity.effects.iter().enumerate() {
                    match effect {
                        StatEffect::Add { stat, amount } => {
                            str += &format!("+{} {}", amount, stat);
                        }
                        StatEffect::Subtract { stat, amount } => {
                            str += &format!("-{} {}", amount, stat);
                        }
                    }

                    if i != (count - 1) {
                        str += ", ";
                    }
                }

                str
            };

            render.text(
                x, y + 40.0,
                8,
                16,
                1.0,
                &effect_text,
            )
        }
    }
}
