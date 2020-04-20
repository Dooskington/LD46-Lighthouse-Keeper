use crate::game::*;
use specs::prelude::*;
use rand::{seq::SliceRandom, Rng};

#[derive(Clone)]
pub struct RandomHappening {
    pub id: i32,
    pub chance: f32,
    pub stat_effects: Vec<StatEffect>,
    pub condition_effects: Vec<ConditionEffect>,
    pub conditions: Vec<GameCondition>,
    pub message: String,
}

pub struct ActivityComponent {
    activity: Activity,
}

impl ActivityComponent {
    pub fn new(activity: Activity) -> Self {
        ActivityComponent { activity }
    }
}

impl Component for ActivityComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Clone)]
pub struct Activity {
    pub name: String,
    pub message: String,
    pub hours_required: i32,
    pub event: GameEvent,
    pub effects: Vec<StatEffect>,
    pub condition_effects: Vec<ConditionEffect>,
    pub conditions: Vec<GameCondition>,
}

#[derive(Default)]
pub struct ActivityState {
    pub activities: Vec<Activity>,
    pub happenings: Vec<RandomHappening>,
    pub is_rebuild_required: bool,
    pub last_happening_id: Option<i32>,
}

impl ActivityState {
    pub fn new() -> Self {
        ActivityState {
            activities: create_activities(),
            happenings: create_happenings(),
            is_rebuild_required: false,
            last_happening_id: None,
        }
    }
}

#[derive(Default)]
pub struct ActivitySystem {
    game_event_reader: Option<ReaderId<GameEvent>>,
    on_clicked_event_reader: Option<ReaderId<OnClickedEvent>>,
}

impl<'a> System<'a> for ActivitySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, ActivityState>,
        ReadExpect<'a, StatsState>,
        ReadExpect<'a, EventChannel<OnClickedEvent>>,
        WriteExpect<'a, EventChannel<GameEvent>>,
        WriteExpect<'a, EventChannel<LogEvent>>,
        ReadStorage<'a, ActivityComponent>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.game_event_reader = Some(
            world
                .fetch_mut::<EventChannel<GameEvent>>()
                .register_reader(),
        );

        self.on_clicked_event_reader = Some(
            world
                .fetch_mut::<EventChannel<OnClickedEvent>>()
                .register_reader(),
        );
    }

    fn run(
        &mut self,
        (ents, mut activity_state, stats, on_clicked_events, mut game_events, mut log_events, activity_comps): Self::SystemData,
    ) {
        let mut queued_happening: Option<RandomHappening> = None;
        for event in game_events.read(&mut self.game_event_reader.as_mut().unwrap()) {
            match event {
                GameEvent::GameOver | GameEvent::RefreshActivities => {
                    activity_state.is_rebuild_required = true;
                }
                GameEvent::NewDayStarted { .. } => {
                    // TODO check for mail (or use a MailSystem)
                }
                GameEvent::NewTimeOfDayStarted { .. } => {
                    activity_state.is_rebuild_required = true;

                    // Choose and run a random event
                    let mut rng = rand::thread_rng();
                    let mut happenings = activity_state.happenings.clone();
                    happenings.shuffle(&mut rng);
                    for happening in happenings {
                        if let Some(id) = activity_state.last_happening_id {
                            // Don't run the same happening twice in a row
                            if happening.id == id {
                                continue;
                            }
                        }

                        for condition in happening.conditions.iter() {
                            if !stats.condition(*condition) {
                                continue;
                            }
                        }

                        let roll: f32 = rng.gen();
                        if roll < happening.chance {
                            queued_happening = Some(happening);
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(happening) = queued_happening {
            log_events.single_write(LogEvent { message: happening.message.clone(), color: COLOR_BLUE });
            game_events.single_write(GameEvent::HandleStatEffects {
                effects: happening.stat_effects,
            });

            game_events.single_write(GameEvent::HandleConditionEffects {
                effects: happening.condition_effects,
            });

            activity_state.last_happening_id = Some(happening.id);
        }

        for event in on_clicked_events.read(&mut self.on_clicked_event_reader.as_mut().unwrap()) {
            if let Some(comp) = activity_comps.get(event.ent) {
                if !comp.activity.message.is_empty() {
                    log_events.single_write(LogEvent { message: comp.activity.message.clone(), color: COLOR_BLACK });
                }

                game_events.single_write(comp.activity.event.clone());
                game_events.single_write(GameEvent::HandleStatEffects {
                    effects: comp.activity.effects.clone(),
                });
                game_events.single_write(GameEvent::HandleConditionEffects {
                    effects: comp.activity.condition_effects.clone(),
                });
                game_events.single_write(GameEvent::ProgressTime {
                    hours: comp.activity.hours_required,
                });
                game_events.single_write(GameEvent::RefreshActivities);
            }
        }
    }
}

pub fn create_activities() -> Vec<Activity> {
    vec![
        Activity {
            name: String::from("Go Fishing"),
            message: String::from(""),
            hours_required: 2,
            event: GameEvent::ActivityGoFishing,
            effects: vec![],
            condition_effects: vec![],
            conditions: vec![],
        },
        Activity {
            name: String::from("Walk on the Beach"),
            message: String::from("You go for a walk along the beach."),
            hours_required: 1,
            event: GameEvent::None,
            effects: vec![],
            condition_effects: vec![],
            conditions: vec![],
        },
        Activity {
            name: String::from("Paint a Picture"),
            message: String::from("You spend some time painting a picture."),
            hours_required: 3,
            event: GameEvent::None,
            effects: vec![StatEffect::Add { stat: Stat::Sanity, amount: 2 }],
            condition_effects: vec![ConditionEffect::Clear { condition: GameCondition::Inspired }],
            conditions: vec![GameCondition::Inspired],
        },
        Activity {
            name: String::from("Perform Maintenance"),
            message: String::from("You maintain some fixtures around the lighthouse."),
            hours_required: 2,
            event: GameEvent::None,
            effects: vec![StatEffect::Subtract {
                stat: Stat::Parts,
                amount: 1,
            }],
            condition_effects: vec![ConditionEffect::Clear { condition: GameCondition::LighthouseDamaged }],
            conditions: vec![GameCondition::LighthouseDamaged],
        },
        Activity {
            name: String::from("Repair Lens"),
            message: String::from("You repair the broken lens."),
            hours_required: 2,
            event: GameEvent::None,
            effects: vec![StatEffect::Subtract {
                stat: Stat::Parts,
                amount: 1,
            }],
            condition_effects: vec![ConditionEffect::Clear { condition: GameCondition::LensBroken }],
            conditions: vec![GameCondition::LensBroken],
        },
        Activity {
            name: String::from("Repair Generator"),
            message: String::from("You repair the generator."),
            hours_required: 3,
            event: GameEvent::None,
            effects: vec![StatEffect::Subtract {
                stat: Stat::Parts,
                amount: 1,
            }],
            condition_effects: vec![ConditionEffect::Clear { condition: GameCondition::GeneratorBroken }],
            conditions: vec![GameCondition::GeneratorBroken],
        },
        Activity {
            name: String::from("Pray To Jand"),
            message: String::from("You pray to Jand, for protection and fortune. Perhaps it will pity you."),
            hours_required: 1,
            event: GameEvent::ActivityPrayToJand,
            effects: vec![StatEffect::Add {
                stat: Stat::Sanity,
                amount: 1,
            }],
            condition_effects: vec![],
            conditions: vec![],
        },
        Activity {
            name: String::from("Have a Drink"),
            message: String::from("You have a drink to dull the pain."),
            hours_required: 1,
            event: GameEvent::ActivityDrinkAlcobev,
            effects: vec![
                StatEffect::Add {
                    stat: Stat::Sanity,
                    amount: 1,
                },
                StatEffect::Subtract {
                    stat: Stat::Food,
                    amount: 1,
                },
            ],
            condition_effects: vec![ConditionEffect::Clear { condition: GameCondition::Dread }],
            conditions: vec![GameCondition::Dread],
        },
        Activity {
            name: String::from("Lay in Bed"),
            message: String::from("You lay in bed for a few hours and think."),
            hours_required: 3,
            event: GameEvent::ActivityDrinkAlcobev,
            effects: vec![],
            condition_effects: vec![ConditionEffect::Clear { condition: GameCondition::Dread }],
            conditions: vec![GameCondition::Dread],
        },
        Activity {
            name: String::from("Hunt Rats"),
            message: String::from("You hunt some of the scrawny rats that scurry about the lighthouse."),
            hours_required: 1,
            event: GameEvent::ActivityHuntRats,
            effects: vec![
                StatEffect::Subtract {
                    stat: Stat::Sanity,
                    amount: 3,
                },
                StatEffect::Add {
                    stat: Stat::Food,
                    amount: 1,
                },
            ],
            condition_effects: vec![],
            conditions: vec![GameCondition::Starving],
        },
        Activity {
            name: String::from("End Game (TODO)"),
            message: String::from("The game is now over."),
            hours_required: 0,
            event: GameEvent::GameOver,
            effects: vec![],
            condition_effects: vec![],
            conditions: vec![GameCondition::FinalDay],
        },
    ]
}

pub fn create_happenings() -> Vec<RandomHappening> {
    vec![
        RandomHappening {
            id: 0,
            message: String::from("Rough winds and waves damage the lighthouse."),
            chance: 0.1,
            stat_effects: vec![],
            condition_effects: vec![ConditionEffect::Set { condition: GameCondition::LighthouseDamaged }],
            conditions: vec![],
        },
        RandomHappening {
            id: 1,
            message: String::from("The lens on the lighthouse cracks."),
            chance: 0.1,
            stat_effects: vec![],
            condition_effects: vec![ConditionEffect::Set { condition: GameCondition::LensBroken }],
            conditions: vec![],
        },
        RandomHappening {
            id: 2,
            message: String::from("The generator makes a strange sound."),
            chance: 0.1,
            stat_effects: vec![],
            condition_effects: vec![ConditionEffect::Set { condition: GameCondition::GeneratorBroken }],
            conditions: vec![],
        },
        RandomHappening {
            id: 3,
            message: String::from("Some food crates wash up on shore. (Food +1)"),
            chance: 0.1,
            stat_effects: vec![StatEffect::Add { stat: Stat::Food, amount: 1 }],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 4,
            message: String::from("Some scrap metal washes up on shore. (Parts +1)"),
            chance: 0.1,
            stat_effects: vec![StatEffect::Add { stat: Stat::Parts, amount: 1 }],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 5,
            message: String::from("A shadow passes you in the stairwell. (Sanity -1)"),
            chance: 0.1,
            stat_effects: vec![StatEffect::Subtract { stat: Stat::Sanity, amount: 1 }],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 6,
            message: String::from("You hear a child screaming. (Sanity -3)"),
            chance: 0.05,
            stat_effects: vec![StatEffect::Subtract { stat: Stat::Sanity, amount: 3 }],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 7,
            message: String::from("The voices beg you to end it. Can you stand to stay here any longer? (Sanity -5)"),
            chance: 0.01,
            stat_effects: vec![StatEffect::Subtract { stat: Stat::Sanity, amount: 5 }],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 8,
            message: String::from("Waves crash against the island."),
            chance: 0.2,
            stat_effects: vec![],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 9,
            message: String::from("The wind howls."),
            chance: 0.2,
            stat_effects: vec![],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 10,
            message: String::from("The island is quiet."),
            chance: 0.2,
            stat_effects: vec![],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 11,
            message: String::from("Some rats have gotten into the pantry. (Food -1)"),
            chance: 0.2,
            stat_effects: vec![StatEffect::Subtract { stat: Stat::Food, amount: 1 }],
            condition_effects: vec![],
            conditions: vec![],
        },
        RandomHappening {
            id: 12,
            message: String::from("You are feeling existential dread."),
            chance: 0.1,
            stat_effects: vec![],
            condition_effects: vec![ConditionEffect::Set { condition: GameCondition::Dread }],
            conditions: vec![],
        },
        RandomHappening {
            id: 12,
            message: String::from("You are feeling inspired and creative."),
            chance: 0.1,
            stat_effects: vec![],
            condition_effects: vec![ConditionEffect::Set { condition: GameCondition::Inspired }],
            conditions: vec![],
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

    if world.read_resource::<StatsState>().condition(GameCondition::GameOver) {
        // Don't create new activities if the game is over
        return;
    }

    let activities = world.read_resource::<ActivityState>().activities.clone();
    let mut layout_pos_x = 975.0;
    let mut layout_pos_y = 16.0;
    let mut counter = 0;
    for activity in activities {
        let mut are_conditions_satisfied = true;
        {
            let stats = world.read_resource::<StatsState>();
            for condition in activity.conditions.iter() {
                if !stats.condition(*condition) {
                    are_conditions_satisfied = false;
                    break;
                }
            }

            for effect in activity.effects.iter() {
                match effect {
                    StatEffect::Subtract { stat, amount } => {
                        if stats.stat(*stat) < *amount {
                            are_conditions_satisfied = false;
                            break;
                        }
                    },
                    _ => {}
                }
            }
        }

        let (time_of_day, hours_passed) = {
            let time = world.read_resource::<TimeState>();
            (time.time_of_day, time.hours_passed)
        };

        if (time_of_day == TimeOfDay::Night) && ((hours_passed + activity.hours_required) > 4) {
            are_conditions_satisfied = false;
        }

        if !are_conditions_satisfied {
            continue;
        }

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
            .with(ActivityComponent::new(activity.clone()))
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

        if ((counter + 1) % 3) == 0 {
            layout_pos_y = 16.0;
            layout_pos_x -= 250.0;
        }

        counter += 1;
    }

    world.write_resource::<ActivityState>().is_rebuild_required = false;
}

#[derive(Default)]
pub struct ActivityInfoRenderSystem;

impl<'a> System<'a> for ActivityInfoRenderSystem {
    type SystemData = (
        Write<'a, RenderState>,
        ReadExpect<'a, StatsState>,
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, ActivityComponent>,
    );

    fn run(&mut self, (mut render, stats, transforms, activity_comps): Self::SystemData) {
        for (transform, activity) in (&transforms, &activity_comps).join() {
            let x = transform.position.x as f32 + 16.0;
            let y = transform.position.y as f32 + 12.0;
            render.bind_transparency(Transparency::Opaque);
            render.bind_layer(layers::LAYER_UI);
            render.bind_texture(resources::TEX_FONT);
            render.bind_color(COLOR_BLACK);
            render.text(x, y, 8, 16, 1.2, &activity.activity.name);

            let hours_text = if activity.activity.hours_required == 1 {
                format!("{} hour", activity.activity.hours_required)
            } else {
                format!("{} hours", activity.activity.hours_required)
            };

            render.text(x, y + 20.0, 8, 16, 1.0, &hours_text);

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

            render.text(x, y + 40.0, 8, 16, 1.0, &effect_text)
        }

        // Game Over screen
        if stats.condition(GameCondition::GameOver) {
            let pos_x = 700.0;
            let pos_y = 250.0;

            // Render text
            render.bind_texture(resources::TEX_FONT);
            render.bind_color(COLOR_BLACK);
            render.text(pos_x + 16.0,pos_y + 16.0, 8, 16, 2.0, "Game Over");
            render.text(
                pos_x + 16.0,
                pos_y + 50.0,
                8,
                16,
                1.0,
                &format!("{}", "TODO"),
            );
        }
    }
}
