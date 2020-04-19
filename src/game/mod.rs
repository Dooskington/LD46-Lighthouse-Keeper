pub mod activity;
pub mod audio;
pub mod clickable;
pub mod layers;
pub mod merchant;
pub mod physics;
pub mod render;
pub mod resources;
pub mod stats;
pub mod time;
pub mod transform;

use activity::*;
use audio::AudioAssetDb;
use clickable::*;
use gfx::{color::*, renderer::Transparency, sprite::SpriteRegion};
use layers::*;
use merchant::*;
use ncollide2d::{pipeline::CollisionGroups, shape::Cuboid};
use nphysics2d::object::BodyStatus;
use physics::*;
use render::{RenderState, SpriteComponent, SpriteRenderSystem};
use shrev::EventChannel;
use specs::prelude::*;
use stats::*;
use std::default::Default;
use time::*;
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
    NewDayStarted { day: i32 },
    NewTimeOfDayStarted { time_of_day: TimeOfDay },
    ProgressTime { hours: i32 },
    HandleStatEffects { effects: Vec<StatEffect> },
    MerchantArrived,
    GameOver,
    FinalDayGameWin,
    RefreshActivities,
    ActivityGoFishing,
    ActivityPerformMaintenance,
    ActivityPrayToJand,
    ActivityDrinkAlcobev,
    ActivityHuntRats,
    ActivityTinker,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum GameCondition {
    FinalDay,
    GameOver,
    GeneratorBroken,
    LensBroken,
    LighthouseDamaged,
    Starving,
    BuggingOut,
    Insane,
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
        world.insert(MerchantState::new());
        world.insert(AudioAssetDb::new());
        world.insert(EventChannel::<CollisionEvent>::new());
        world.insert(EventChannel::<OnClickedEvent>::new());
        world.insert(EventChannel::<GameEvent>::new());

        let mut tick_dispatcher = DispatcherBuilder::new()
            .with(ClickableSystem::default(), "clickable", &[])
            .with(TimeSystem::default(), "time", &[])
            .with(StatsSystem::default(), "stats", &[])
            .with(MerchantSystem::default(), "merchant", &[])
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

        world
            .write_resource::<EventChannel<GameEvent>>()
            .single_write(GameEvent::NewGameStarted);

        GameState {
            world,
            tick_dispatcher,
            physics_dispatcher,
        }
    }
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
