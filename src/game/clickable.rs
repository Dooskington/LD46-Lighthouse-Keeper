use crate::game::{physics::*, Point2d, *};
use gfx::input::*;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;

pub struct OnClickedEvent {
    pub ent: Entity,
}

#[derive(Debug, PartialEq, Eq)]
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
        WriteExpect<'a, EventChannel<OnClickedEvent>>,
        WriteStorage<'a, ClickableComponent>,
    );

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
    }

    fn run(
        &mut self,
        (ents, input, physics, mut on_clicked_events, mut clickables): Self::SystemData,
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

        // How do we change the sprite state?
        // Could just grab the sprite components
        // new field on clickable component
        // hovered_sprite: Option<SpriteRegion>
        // clicked_sprite: Option<SpriteRegion>
        // at the end of the loop, set the sprite based on the ClickableState
        // (if normal_sprite is none, just set it to whatever the sprite is right now)

        for (ent, clickable) in (&ents, &mut clickables).join() {
            if cursor_hit_ents.contains(ent.id()) {
                if input.is_mouse_button_held(MouseButton::Left) {
                    if clickable.state != ClickableState::Clicked {
                        //println!("clicked");
                        clickable.state = ClickableState::Clicked;
                        on_clicked_events.single_write(OnClickedEvent { ent });
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
