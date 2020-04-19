use crate::game::{physics::*, Point2d, *};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;

#[derive(Debug, PartialEq, Eq)]
enum ClickableState {
    Normal,
    Hovered,
    Clicked,
}

pub struct ClickableComponent {
    state: ClickableState,
    on_click_event: Option<GameEvent>,
}

impl ClickableComponent {
    pub fn new(on_click_event: Option<GameEvent>) -> Self {
        ClickableComponent {
            state: ClickableState::Normal,
            on_click_event,
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
        WriteExpect<'a, EventChannel<GameEvent>>,
        WriteStorage<'a, ClickableComponent>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
    }

    fn run(
        &mut self,
        (ents, input, physics, mut workstation_events, mut clickables): Self::SystemData,
    ) {
        // Gather all ents hit by the mouse
        let mut cursor_hit_ents = BitSet::new();
        let mouse_pos_world = input.cursor_pos() * PIXELS_TO_WORLD_UNITS;
        let all_collision_groups = CollisionGroups::new();

        for interference in
            physics.interferences_with_point(&mouse_pos_world, &all_collision_groups)
        {
            let hit_ent = interference
                .1
                .user_data()
                .unwrap()
                .downcast_ref::<Entity>()
                .cloned()
                .unwrap();
            cursor_hit_ents.add(hit_ent.id());
        }

        for (ent, clickable) in (&ents, &mut clickables).join() {
            if cursor_hit_ents.contains(ent.id()) {
                if input.is_mouse_button_held(MouseButton::Left) {
                    if clickable.state != ClickableState::Clicked {
                        //println!("clicked");
                        clickable.state = ClickableState::Clicked;
                        if let Some(event) = clickable.on_click_event {
                            workstation_events.single_write(event);
                        }
                    }
                } else {
                    if clickable.state != ClickableState::Hovered {
                        //println!("hovered");
                        clickable.state = ClickableState::Hovered;
                    }
                }
            } else {
                if clickable.state != ClickableState::Normal {
                    //println!("normal");
                    clickable.state = ClickableState::Normal;
                }
            }
        }
    }
}
