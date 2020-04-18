pub mod alien;
pub mod audio;
pub mod clickable;
pub mod layers;
pub mod physics;
pub mod render;
pub mod resources;
pub mod transform;
pub mod workstation;

use alien::*;
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
use workstation::*;

pub type Vector2f = nalgebra::Vector2<f32>;
pub type Vector2d = nalgebra::Vector2<f64>;
pub type Point2f = nalgebra::Point2<f32>;
pub type Point2d = nalgebra::Point2<f64>;

pub const PIXELS_PER_WORLD_UNIT: u32 = 32;
pub const PIXELS_TO_WORLD_UNITS: f64 = (1.0 / PIXELS_PER_WORLD_UNIT as f64);

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
        world.insert(AudioAssetDb::new());
        world.insert(EventChannel::<CollisionEvent>::new());
        world.insert(EventChannel::<WorkstationEvent>::new());
        world.insert(WorkstationState::new());

        let mut tick_dispatcher = DispatcherBuilder::new()
            .with(ClickableSystem::default(), "clickable", &[])
            .with(AlienSystem::default(), "alien", &[])
            .with(WorkstationSystem::default(), "workstation", &[])
            .with_thread_local(AlienInfoRenderSystem::default())
            .with_thread_local(WorkstationInfoRenderSystem::default())
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

        build_scene(&mut world, width, height);

        GameState {
            world,
            tick_dispatcher,
            physics_dispatcher,
        }
    }
}

fn build_scene(world: &mut World, width: u32, height: u32) {
    // Testing buttons
    let collision_groups = CollisionGroups::new();
    world
        .create_entity()
        .with(TransformComponent::new(
            Vector2d::new(180.0, 360.0),
            Vector2f::new(1.0, 1.0),
        ))
        .with(ColliderComponent::new(
            Cuboid::new(Vector2d::new(
                (49.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
                (49.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
            )),
            Vector2d::zeros(),
            collision_groups,
            0.0,
        ))
        .with(ClickableComponent::new(Some(
            WorkstationEvent::LowerTemperature,
        )))
        .with(SpriteComponent::new(
            SpriteRegion {
                x: 0,
                y: 0,
                w: 49,
                h: 49,
            },
            resources::TEX_SPRITESHEET_BUTTONS,
            Point2f::origin(),
            COLOR_WHITE,
            layers::LAYER_WORKSTATION_CONTROLS,
            Transparency::Opaque,
        ))
        .build();

    world
        .create_entity()
        .with(TransformComponent::new(
            Vector2d::new(180.0, 300.0),
            Vector2f::new(1.0, 1.0),
        ))
        .with(ColliderComponent::new(
            Cuboid::new(Vector2d::new(
                (49.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
                (49.0 / 2.0) * PIXELS_TO_WORLD_UNITS,
            )),
            Vector2d::zeros(),
            collision_groups,
            0.0,
        ))
        .with(ClickableComponent::new(Some(
            WorkstationEvent::RaiseTemperature,
        )))
        .with(SpriteComponent::new(
            SpriteRegion {
                x: 0,
                y: 0,
                w: 49,
                h: 49,
            },
            resources::TEX_SPRITESHEET_BUTTONS,
            Point2f::origin(),
            COLOR_WHITE,
            layers::LAYER_WORKSTATION_CONTROLS,
            Transparency::Opaque,
        ))
        .build();

    // Alien
    world
        .create_entity()
        .with(TransformComponent::new(
            Vector2d::new(width as f64 / 2.0, (height as f64 / 2.0) - 64.0),
            Vector2f::new(1.0, 1.0),
        ))
        .with(SpriteComponent::new(
            SpriteRegion {
                x: 0,
                y: 0,
                w: 56,
                h: 56,
            },
            resources::TEX_SPRITESHEET_ALIEN,
            Point2f::new(0.5, 0.5),
            COLOR_WHITE,
            layers::LAYER_LAB,
            Transparency::Opaque,
        ))
        .with(AlienComponent::new())
        .build();
}

fn lerp(start: f32, end: f32, percentage: f32) -> f32 {
    let percentage = percentage.max(0.0).min(1.0);
    start + ((end - start) * percentage)
}
