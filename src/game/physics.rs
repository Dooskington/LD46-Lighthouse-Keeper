use crate::game::{
    render::SpriteComponent, transform::TransformComponent, Point2d, Vector2d,
    PIXELS_PER_WORLD_UNIT, PIXELS_TO_WORLD_UNITS,
};
use nalgebra::{Isometry2, Vector2};
use ncollide2d::pipeline::InterferencesWithPoint;
use ncollide2d::{
    pipeline::{CollisionGroups, ContactEvent},
    shape::{Shape, ShapeHandle},
};
use nphysics2d::{
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    math::Velocity,
    object::{
        Body, BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodyHandle, DefaultBodySet,
        DefaultColliderHandle, DefaultColliderSet, Ground, RigidBodyDesc,
    },
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use shrev::EventChannel;
use specs::prelude::*;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Debug)]
pub enum CollisionType {
    Started,
    Stopped,
}

pub struct CollisionEvent {
    pub entity_a: Option<Entity>,
    pub collider_handle_a: DefaultColliderHandle,
    pub entity_b: Option<Entity>,
    pub collider_handle_b: DefaultColliderHandle,
    pub normal: Option<Vector2<f64>>,
    pub collision_point: Option<Point2d>,
    pub ty: CollisionType,
}

pub struct PhysicsState {
    pub lerp: f64,
    pub bodies: DefaultBodySet<f64>,
    pub colliders: DefaultColliderSet<f64>,
    mechanical_world: DefaultMechanicalWorld<f64>,
    geometrical_world: DefaultGeometricalWorld<f64>,
    joint_constraints: DefaultJointConstraintSet<f64>,
    force_generators: DefaultForceGeneratorSet<f64>,
    ent_body_handles: HashMap<u32, DefaultBodyHandle>,
    ent_collider_handles: HashMap<u32, DefaultColliderHandle>,
    ground_body_handle: DefaultBodyHandle,
}

impl PhysicsState {
    pub fn new() -> Self {
        let mut bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();

        let gravity = Vector2::new(0.0, -9.81);
        let mut mechanical_world = DefaultMechanicalWorld::new(gravity);
        mechanical_world
            .integration_parameters
            .max_ccd_position_iterations = 10;

        mechanical_world.integration_parameters.max_ccd_substeps = 1;

        let geometrical_world = DefaultGeometricalWorld::new();
        let joint_constraints = DefaultJointConstraintSet::new();
        let force_generators = DefaultForceGeneratorSet::new();

        let body_handles = HashMap::new();
        let collider_handles = HashMap::new();
        let ground_body_handle = bodies.insert(Ground::new());

        PhysicsState {
            lerp: 0.0,
            bodies,
            colliders,
            mechanical_world,
            geometrical_world,
            joint_constraints,
            force_generators,
            ent_body_handles: body_handles,
            ent_collider_handles: collider_handles,
            ground_body_handle,
        }
    }

    pub fn step(&mut self) {
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );
    }

    pub fn interferences_with_point<'a, 'b>(
        &'a self,
        point: &'b Point2d,
        groups: &'b CollisionGroups,
    ) -> InterferencesWithPoint<'a, 'b, f64, DefaultColliderSet<f64>> {
        self.geometrical_world
            .interferences_with_point(&self.colliders, point, groups)
    }
}

#[derive(Debug)]
pub struct RigidbodyComponent {
    pub handle: Option<DefaultBodyHandle>,
    pub velocity: Velocity<f64>,
    pub last_velocity: Velocity<f64>,
    pub max_linear_velocity: f64,
    pub mass: f64,
    pub status: BodyStatus,
}

impl RigidbodyComponent {
    pub fn new(
        mass: f64,
        linear_velocity: Vector2<f64>,
        max_linear_velocity: f64,
        status: BodyStatus,
    ) -> Self {
        let velocity = Velocity::new(linear_velocity, 0.0);
        RigidbodyComponent {
            handle: None,
            velocity,
            last_velocity: velocity,
            max_linear_velocity,
            mass,
            status,
        }
    }
}

impl Component for RigidbodyComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

pub struct ColliderComponent {
    pub shape: ShapeHandle<f64>,
    pub center: Vector2<f64>,
    pub offset: Vector2<f64>,
    pub collision_groups: CollisionGroups,
    pub density: f64,
    pub ccd_enabled: bool,
}

impl ColliderComponent {
    pub fn new<S: Shape<f64>>(
        shape: S,
        offset: Vector2<f64>,
        collision_groups: CollisionGroups,
        density: f64,
    ) -> Self {
        ColliderComponent {
            shape: ShapeHandle::new(shape),
            center: Vector2d::zeros(),
            offset,
            collision_groups,
            density,
            // CCD seems kinda buggy at the moment https://github.com/rustsim/nphysics/issues/255
            ccd_enabled: true,
        }
    }
}

impl Component for ColliderComponent {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Default)]
pub struct RigidbodySendPhysicsSystem {
    pub inserted_bodies: BitSet,
    pub modified_bodies: BitSet,
    pub removed_bodies: BitSet,
    pub modified_transforms: BitSet,
    pub transform_reader_id: Option<ReaderId<ComponentEvent>>,
    pub rigidbody_reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for RigidbodySendPhysicsSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, PhysicsState>,
        WriteStorage<'a, RigidbodyComponent>,
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (entities, mut physics, mut rigidbodies, transforms): Self::SystemData) {
        self.inserted_bodies.clear();
        self.modified_bodies.clear();
        self.removed_bodies.clear();
        self.modified_transforms.clear();

        // Process TransformComponent events into a bitset
        let transform_events = transforms
            .channel()
            .read(self.transform_reader_id.as_mut().unwrap());
        for event in transform_events {
            match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.modified_transforms.add(*id);
                }
                _ => {}
            }
        }

        // Process RigidbodyComponent events into bitsets
        let rigidbody_events = rigidbodies
            .channel()
            .read(self.rigidbody_reader_id.as_mut().unwrap());
        for event in rigidbody_events {
            match event {
                ComponentEvent::Inserted(id) => {
                    self.inserted_bodies.add(*id);
                }
                ComponentEvent::Modified(id) => {
                    self.modified_bodies.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed_bodies.add(*id);
                }
            }
        }

        // Handle removed rigidbodies
        for ent_id in (&self.removed_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.remove(&ent_id) {
                physics.bodies.remove(rb_handle);
                println!(
                    "[RigidbodySendPhysicsSystem] Removed rigidbody. Entity Id = {}",
                    ent_id
                );
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to remove rigidbody because it didn't exist! Entity Id = {}", ent_id);
            }
        }

        // Handle inserted rigidbodies
        for (ent, transform, rigidbody, ent_id) in (
            &entities,
            &transforms,
            &mut rigidbodies,
            &self.inserted_bodies,
        )
            .join()
        {
            if let Some(rb_handle) = physics.ent_body_handles.remove(&ent.id()) {
                eprintln!("[RigidbodySendPhysicsSystem] Duplicate rigidbody found in physics world! Removing it. Entity Id = {}, Handle = {:?}", ent_id, rb_handle);
                physics.bodies.remove(rb_handle);
            }

            let rigid_body = RigidBodyDesc::new()
                .translation(transform.position * PIXELS_TO_WORLD_UNITS)
                .rotation(0.0)
                .gravity_enabled(false)
                .status(rigidbody.status)
                .velocity(rigidbody.velocity)
                .mass(rigidbody.mass)
                .linear_motion_interpolation_enabled(true)
                // TODO uncomment once bugfix is released:
                // https://github.com/rustsim/nphysics/pull/254
                //.max_linear_velocity(rigidbody.max_linear_velocity)
                .user_data(ent)
                .build();

            let rb_handle = physics.bodies.insert(rigid_body);
            rigidbody.handle = Some(rb_handle);
            physics.ent_body_handles.insert(ent.id(), rb_handle);
            println!(
                "[RigidbodySendPhysicsSystem] Inserted rigidbody. Entity Id = {}, Handle = {:?}",
                ent_id, rb_handle
            );
        }

        // Handle modified rigidbodies
        for (ent, rigidbody, ent_id) in (&entities, &rigidbodies, &self.modified_bodies).join() {
            if let Some(rb_handle) = physics.ent_body_handles.get(&ent.id()).cloned() {
                let rb = physics.bodies.rigid_body_mut(rb_handle).unwrap();
                rb.set_velocity(rigidbody.velocity);
                rb.set_status(rigidbody.status);
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent_id);
            }
        }

        // Handle modified transforms
        for (ent, transform, _, _) in (
            &entities,
            &transforms,
            &rigidbodies,
            &self.modified_transforms,
        )
            .join()
        {
            if let Some(rb_handle) = physics.ent_body_handles.get(&ent.id()).cloned() {
                let rb = physics.bodies.rigid_body_mut(rb_handle).unwrap();
                rb.set_position(Isometry2::new(
                    transform.position * PIXELS_TO_WORLD_UNITS,
                    0.0,
                ));
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent.id());
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.transform_reader_id =
            Some(WriteStorage::<TransformComponent>::fetch(&world).register_reader());
        self.rigidbody_reader_id =
            Some(WriteStorage::<RigidbodyComponent>::fetch(&world).register_reader());
    }
}

#[derive(Default)]
pub struct ColliderSendPhysicsSystem {
    pub inserted_colliders: BitSet,
    pub modified_colliders: BitSet,
    pub removed_colliders: BitSet,
    pub modified_transforms: BitSet,
    pub transform_reader_id: Option<ReaderId<ComponentEvent>>,
    pub collider_reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for ColliderSendPhysicsSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, PhysicsState>,
        WriteStorage<'a, ColliderComponent>,
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, RigidbodyComponent>,
        ReadStorage<'a, SpriteComponent>,
    );

    fn run(
        &mut self,
        (entities, mut physics, mut colliders, transforms, rigidbodies, sprites): Self::SystemData,
    ) {
        self.inserted_colliders.clear();
        self.modified_colliders.clear();
        self.removed_colliders.clear();
        self.modified_transforms.clear();

        // Process TransformComponent events into a bitset
        let transform_events = transforms
            .channel()
            .read(self.transform_reader_id.as_mut().unwrap());
        for event in transform_events {
            match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.modified_transforms.add(*id);
                }
                _ => {}
            }
        }

        // Process ColliderComponent events into bitsets
        let collider_events = colliders
            .channel()
            .read(self.collider_reader_id.as_mut().unwrap());
        for event in collider_events {
            match event {
                ComponentEvent::Inserted(id) => {
                    self.inserted_colliders.add(*id);
                }
                ComponentEvent::Modified(id) => {
                    self.modified_colliders.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed_colliders.add(*id);
                }
            }
        }

        // Handle removed colliders
        for ent_id in (&self.removed_colliders).join() {
            if let Some(collider_handle) = physics.ent_collider_handles.remove(&ent_id) {
                physics.colliders.remove(collider_handle);
                println!(
                    "[ColliderSendPhysicsSystem] Removed collider. Entity Id = {}",
                    ent_id
                );
            } else {
                eprintln!("[ColliderSendPhysicsSystem] Failed to remove collider because it didn't exist! Entity Id = {}", ent_id);
            }
        }

        colliders.set_event_emission(false);

        // Handle inserted colliders
        for (ent, transform, collider, _) in (
            &entities,
            &transforms,
            &mut colliders,
            &self.inserted_colliders,
        )
            .join()
        {
            if let Some(collider_handle) = physics.ent_collider_handles.remove(&ent.id()) {
                eprintln!("[ColliderSendPhysicsSystem] Duplicate collider found in physics world! Removing it. Entity Id = {}, Handle = {:?}", ent.id(), collider_handle);
                physics.colliders.remove(collider_handle);
            }

            // If the entity also has a sprite component, estimate the collider center to be the center of the sprite
            if let Some(sprite) = sprites.get(ent) {
                let dist_x = 0.5 - (sprite.pivot.x as f64);
                let dist_y = 0.5 - (sprite.pivot.y as f64);
                collider.center.x = dist_x * (sprite.region.w as f64);
                collider.center.y = dist_y * (sprite.region.h as f64);
            }

            // If this entity has a rigidbody, we need to attach the collider to it.
            // Otherwise we just attach it to the "ground".
            let (parent_body_handle, translation) =
                if let Some(rb_handle) = physics.ent_body_handles.get(&ent.id()) {
                    (
                        rb_handle.clone(),
                        (collider.center + collider.offset) * PIXELS_TO_WORLD_UNITS,
                    )
                } else {
                    (
                        physics.ground_body_handle.clone(),
                        (transform.position + collider.center + collider.offset)
                            * PIXELS_TO_WORLD_UNITS,
                    )
                };

            let collider_desc = ColliderDesc::new(collider.shape.clone())
                .density(collider.density)
                .translation(translation)
                .margin(0.02)
                .ccd_enabled(collider.ccd_enabled)
                .collision_groups(collider.collision_groups.clone())
                .user_data(ent)
                .build(BodyPartHandle(parent_body_handle, 0));
            let collider_handle = physics.colliders.insert(collider_desc);
            physics
                .ent_collider_handles
                .insert(ent.id(), collider_handle);
            println!(
                "[ColliderSendPhysicsSystem] Inserted collider. Entity Id = {}, Handle = {:?}",
                ent.id(),
                collider_handle
            );
        }

        // Handle modified colliders (exclude new colliders)
        for (ent, _, _) in (&entities, &colliders, &self.modified_colliders).join() {
            if let Some(_) = physics.ent_collider_handles.get(&ent.id()).cloned() {
                // TODO
                println!(
                    "[ColliderSendPhysicsSystem] Modified collider: {}",
                    ent.id()
                );
            } else {
                eprintln!("[ColliderSendPhysicsSystem] Failed to update collider because it didn't exist! Entity Id = {}", ent.id());
            }
        }

        // Handle modified transforms (ignoring rigidbodies, because they will update themselves)
        for (ent, transform, collider, _, _) in (
            &entities,
            &transforms,
            &colliders,
            &self.modified_transforms,
            !&rigidbodies,
        )
            .join()
        {
            if let Some(collider_handle) = physics.ent_collider_handles.get(&ent.id()).cloned() {
                let phys_collider = physics.colliders.get_mut(collider_handle).unwrap();
                phys_collider.set_position(Isometry2::new(
                    (transform.position + collider.center + collider.offset)
                        * PIXELS_TO_WORLD_UNITS,
                    0.0,
                ));
            } else {
                eprintln!("[RigidbodySendPhysicsSystem] Failed to update rigidbody because it didn't exist! Entity Id = {}", ent.id());
            }
        }

        colliders.set_event_emission(true);
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.transform_reader_id =
            Some(WriteStorage::<TransformComponent>::fetch(&world).register_reader());
        self.collider_reader_id =
            Some(WriteStorage::<ColliderComponent>::fetch(&world).register_reader());
    }
}

#[derive(Default)]
pub struct WorldStepPhysicsSystem;

impl<'a> System<'a> for WorldStepPhysicsSystem {
    type SystemData = (
        WriteExpect<'a, PhysicsState>,
        WriteExpect<'a, EventChannel<CollisionEvent>>,
    );

    fn run(&mut self, (mut physics, mut collision_events): Self::SystemData) {
        physics.step();

        // Iterate through contact events in reverse order
        // So that that the ball reacts to the most recent contact event first. Until we can get the contact_pair bug sorted
        for event in physics.geometrical_world.contact_events().iter().rev() {
            let new_collision_events = match event {
                ContactEvent::Started(handle1, handle2) => {
                    //println!("contact started: handle1: {:?}, handle2: {:?}", handle1, handle2);
                    if let Some((handle_a, collider_a, handle_b, collider_b, _, manifold)) = physics
                        .geometrical_world
                        .contact_pair(&physics.colliders, *handle1, *handle2, false)
                    {
                        let entity_a = collider_a
                            .user_data()
                            .unwrap()
                            .downcast_ref::<Entity>()
                            .cloned();
                        let entity_b = collider_b
                            .user_data()
                            .unwrap()
                            .downcast_ref::<Entity>()
                            .cloned();

                        let (normal, collision_a_point, collision_b_point) =
                            if let Some(c) = manifold.deepest_contact().cloned() {
                                let collision_a_point =
                                    c.contact.world1 * (PIXELS_PER_WORLD_UNIT as f64);
                                let collision_b_point =
                                    c.contact.world2 * (PIXELS_PER_WORLD_UNIT as f64);
                                (
                                    Some(c.contact.normal.into_inner()),
                                    Some(collision_a_point),
                                    Some(collision_b_point),
                                )
                            } else {
                                (None, None, None)
                            };

                        let event_a = CollisionEvent {
                            entity_a,
                            collider_handle_a: handle_a,
                            entity_b,
                            collider_handle_b: handle_b,
                            normal,
                            collision_point: collision_a_point,
                            ty: CollisionType::Started,
                        };

                        let event_b = CollisionEvent {
                            entity_a: entity_b,
                            collider_handle_a: handle_b,
                            entity_b: entity_a,
                            collider_handle_b: handle_a,
                            normal,
                            collision_point: collision_b_point,
                            ty: CollisionType::Started,
                        };

                        Some(vec![event_a, event_b])
                    } else {
                        eprintln!("No contact pair found for collision!");

                        None
                    }
                }
                ContactEvent::Stopped(_handle1, _handle2) => {
                    //println!("contact stopped: handle1: {:?}, handle2: {:?}", handle1, handle2);
                    // TODO
                    None
                }
            };

            if let Some(events) = new_collision_events {
                collision_events.iter_write(events);
            }
        }
    }
}

pub struct RigidbodyReceivePhysicsSystem;

impl<'a> System<'a> for RigidbodyReceivePhysicsSystem {
    type SystemData = (
        ReadExpect<'a, PhysicsState>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, RigidbodyComponent>,
    );

    fn run(&mut self, (physics, mut transforms, mut rigidbodies): Self::SystemData) {
        for (mut rigidbody, transform) in (&mut rigidbodies, &mut transforms).join() {
            if let Some(body) = physics.bodies.rigid_body(rigidbody.handle.unwrap()) {
                transform.last_position = transform.position;
                rigidbody.last_velocity = rigidbody.velocity.clone();

                transform.position =
                    body.position().translation.vector * PIXELS_PER_WORLD_UNIT as f64;
                rigidbody.velocity = body.velocity().clone();
            }
        }
    }
}
