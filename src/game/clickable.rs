use crate::game::{*, physics::*, Point2d};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;

#[derive(Debug)]
enum ClickableState {
    Normal,
    Hovered,
    Clicked,
}

pub struct ClickableComponent {
    state: ClickableState,
}

impl ClickableComponent {
    pub fn new() -> Self {
        ClickableComponent {
            state: ClickableState::Normal,
        }
    }
}

impl Component for ClickableComponent {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct ClickableSystem;

impl<'a> System<'a> for ClickableSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, InputState>,
        ReadExpect<'a, PhysicsState>,
        WriteStorage<'a, ClickableComponent>,
    );

    fn run(&mut self, (ents, input, physics, mut clickables): Self::SystemData) {
        // Gather all ents hit by the mouse
        let mut cursor_hit_ents = BitSet::new();
        let mouse_pos_world = input.cursor_pos() * PIXELS_TO_WORLD_UNITS;
        let all_collision_groups = CollisionGroups::new();

        for interference in physics.interferences_with_point(
            &mouse_pos_world,
            &all_collision_groups,
        ) {
            let hit_ent = interference.1.user_data().unwrap().downcast_ref::<Entity>().cloned().unwrap();
            cursor_hit_ents.add(hit_ent.id());
        }

        for (ent, clickable) in (&ents, &mut clickables).join() {
            if cursor_hit_ents.contains(ent.id()) {
                match clickable.state {
                    ClickableState::Normal => {
                        println!("hovered");
                        clickable.state = ClickableState::Hovered;
                    },
                    _ => {},
                }
            } else {
                match clickable.state {
                    ClickableState::Hovered => {
                        println!("normal");
                        clickable.state = ClickableState::Normal;
                    },
                    _ => {},
                }
            }
        }
    }
}
