pub mod audio;
pub mod clickable;
pub mod layers;
pub mod physics;
pub mod render;
pub mod resources;
pub mod transform;
pub mod time;
pub mod stats;
pub mod activity;

use activity::*;
use stats::*;
use time::*;
use audio::AudioAssetDb;
use clickable::*;
use gfx::{color::*, renderer::Transparency, sprite::SpriteRegion};
use layers::*;
use ncollide2d::{pipeline::CollisionGroups, shape::Cuboid};
use nphysics2d::object::BodyStatus;
use physics::*;
use render::{RenderState, SpriteComponent, SpriteRenderSystem};
use shrev::EventChannel;
use specs::prelude::*;
use std::default::Default;
use transform::TransformComponent;

pub type Vector2f = nalgebra::Vector2<f32>;
pub type Vector2d = nalgebra::Vector2<f64>;
pub type Point2f = nalgebra::Point2<f32>;
pub type Point2d = nalgebra::Point2<f64>;

pub const PIXELS_PER_WORLD_UNIT: u32 = 32;
pub const PIXELS_TO_WORLD_UNITS: f64 = (1.0 / PIXELS_PER_WORLD_UNIT as f64);

#[derive(Clone)]
pub enum GameEvent {
    NewGameStarted,
    NewDayStarted,
    ProgressTime { hours: i32 },
    HandleStatEffects { effects: Vec<StatEffect> },
    PayDay,
    MerchantArrived,
    StarvationGameOver,
    InsanityGameOver,
    RefreshActivities,
    ActivityGoFishing,
    ActivityPerformMaintenance,
    ActivityPrayToJand,
    ActivityDrinkAlcobev,
}

pub struct GameState<'a, 'b> {
    pub world: World,
    pub tick_dispatcher: Dispatcher<'a, 'b>,
    pub physics_dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(width: u32, height: u32) -> GameState<'a, 'b> {
        let mut world = World::new();

        // Resources
        world.insert(RenderState::new());
        world.insert(PhysicsState::new());
        world.insert(TimeState::new());
        world.insert(StatsState::new());
        world.insert(ActivityState::new());
        world.insert(AudioAssetDb::new());
        world.insert(EventChannel::<CollisionEvent>::new());
        world.insert(EventChannel::<OnClickedEvent>::new());
        world.insert(EventChannel::<GameEvent>::new());

        let mut tick_dispatcher = DispatcherBuilder::new()
            .with(ClickableSystem::default(), "clickable", &[])
            .with(TimeSystem::default(), "time", &[])
            .with(StatsSystem::default(), "stats", &[])
            .with(ActivitySystem::default(), "activity", &["clickable"])
            .with_thread_local(TimeInfoRenderSystem::default())
            .with_thread_local(StatsInfoRenderSystem::default())
            .with_thread_local(ActivityInfoRenderSystem::default())
            .with_thread_local(SpriteRenderSystem::default())
            .build();

        tick_dispatcher.setup(&mut world);

        let mut physics_dispatcher = DispatcherBuilder::new()
            .with_thread_local(RigidbodySendPhysicsSystem::default())
            .with_thread_local(ColliderSendPhysicsSystem::default())
            .with_thread_local(WorldStepPhysicsSystem)
            .with_thread_local(RigidbodyReceivePhysicsSystem)
            .build();

        physics_dispatcher.setup(&mut world);

        world.write_resource::<EventChannel<GameEvent>>().single_write(GameEvent::NewGameStarted);

        build_scene(&mut world, width, height);

        GameState {
            world,
            tick_dispatcher,
            physics_dispatcher,
        }
    }
}

fn build_scene(world: &mut World, width: u32, height: u32) {
    // todo "building scene" is now just handled via the activity system.

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

        world
            .create_entity()
            .with(TransformComponent::new(
                Vector2d::new(340.0, 450.0),
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
                name: String::from("Perform Maintenance"),
                hours_required: 3,
                event: GameEvent::ActivityPerformMaintenance,
                effects: vec![StatEffect::Subtract { stat: Stat::Parts, amount: 1 }],
                is_repeatable: false,
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

        world
            .create_entity()
            .with(TransformComponent::new(
                Vector2d::new(340.0, 550.0),
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
                name: String::from("Pray To Jand"),
                hours_required: 1,
                event: GameEvent::ActivityPrayToJand,
                effects: vec![StatEffect::Add { stat: Stat::Sanity, amount: 1 }],
                is_repeatable: false,
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

        world
            .create_entity()
            .with(TransformComponent::new(
                Vector2d::new(340.0, 650.0),
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
                name: String::from("Have a Drink"),
                hours_required: 1,
                event: GameEvent::ActivityDrinkAlcobev,
                effects: vec![StatEffect::Add { stat: Stat::Sanity, amount: 1 }, StatEffect::Subtract { stat: Stat::Food, amount: 1 }],
                is_repeatable: false,
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
}

fn lerp(start: f32, end: f32, percentage: f32) -> f32 {
    let percentage = percentage.max(0.0).min(1.0);
    start + ((end - start) * percentage)
}

fn color_lerp(start: Color, end: Color, percentage: f32) -> Color {
    let mut c = Color::new(0, 0, 0, 0);
    c.r = lerp(start.r, end.r, percentage);
    c.g = lerp(start.g, end.g, percentage);
    c.b = lerp(start.b, end.b, percentage);
    c.a = lerp(start.a, end.a, percentage);

    c
}