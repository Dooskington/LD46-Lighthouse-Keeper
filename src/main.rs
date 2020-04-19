mod game;

use game::{
    activity::*,
    audio::{AudioAssetDb, AudioAssetId},
    physics::PhysicsState,
    render::RenderState,
    resources::*,
    time::*,
    GameState,
};
use gfx::{
    color::*,
    image::*,
    input::InputState,
    renderer::*,
    texture::*,
    window::{self, *},
};
use specs::prelude::*;

fn main() {
    let window_title: &str = "LD46 - Keep It Alive";
    let window_width: u32 = 576;
    let window_height: u32 = 1024;
    let render_scale: f32 = 1.0;
    let state = GameState::new(window_width, window_height);

    window::run(
        window_title,
        window_width,
        window_height,
        render_scale,
        state,
        move |game, renderer| {
            // Import textures
            {
                import_texture(
                    game::resources::TEX_COSTANZA,
                    "res/textures/costanza.png",
                    renderer,
                );
                import_texture(
                    game::resources::TEX_SPRITESHEET_UI,
                    "res/textures/ui-sprites.png",
                    renderer,
                );
                import_texture(game::resources::TEX_FONT, "res/textures/font.png", renderer);
                import_texture(
                    game::resources::TEX_BG_LIGHTHOUSE,
                    "res/textures/lighthouse-bg.png",
                    renderer,
                );
                import_texture(
                    game::resources::TEX_BG_LIGHTHOUSE_LIGHT,
                    "res/textures/lighthouse-light-bg.png",
                    renderer,
                );
            }

            // TODO
            // Import audio
            /*
            {
                let mut audio_db = game.world.write_resource::<AudioAssetDb>();
            }
            */
        },
        move |game, _window, input, dt| {
            game.world.insert::<InputState>(input.clone());
            game.world.insert::<DeltaTime>(dt);
            game.world.write_resource::<RenderState>().clear_commands();

            game.tick_dispatcher.dispatch(&mut game.world);
            game.physics_dispatcher.dispatch(&mut game.world);

            if game
                .world
                .read_resource::<ActivityState>()
                .is_rebuild_required
            {
                create_activity_ents(&mut game.world);
            }

            game.world.maintain();
        },
        move |game, _ticks, lerp, window, renderer| {
            game.world.write_resource::<PhysicsState>().lerp = lerp;

            let mut render = game.world.write_resource::<RenderState>();

            // FPS text
            let msg = format!("{}", window.fps);
            render.bind_color(COLOR_BLUE);
            render.bind_layer(game::layers::LAYER_UI);
            render.bind_transparency(Transparency::Transparent);
            render.bind_texture(game::resources::TEX_FONT);
            render.text(2.0, window_height as f32 - 18.0, 8, 16, 1.0, &msg);

            // Lighthouse Background Layer
            render.bind_color(COLOR_WHITE);
            render.bind_layer(game::layers::LAYER_BG);
            render.bind_transparency(Transparency::Opaque);
            render.bind_texture(game::resources::TEX_BG_LIGHTHOUSE);
            render.textured_quad(
                (0.0, window_height as f32),
                (window_width as f32, window_height as f32),
                (0.0, 0.0),
                (window_width as f32, 0.0),
            );

            // Lighthouse light (during night)
            if game.world.read_resource::<TimeState>().time_of_day == TimeOfDay::Night {
                // TODO
                // Don't do this if the StatsState says that the lighthouse isn't working

                render.bind_layer(game::layers::LAYER_BG + 1);
                render.bind_transparency(Transparency::Opaque);
                render.bind_texture(game::resources::TEX_BG_LIGHTHOUSE_LIGHT);
                render.textured_quad(
                    (0.0, window_height as f32),
                    (window_width as f32, window_height as f32),
                    (0.0, 0.0),
                    (window_width as f32, 0.0),
                );
            }

            // Process commands into batches and send to the renderer
            let batches = renderer.process_commands(render.commands());
            renderer.render(window.dpi_scale_factor, batches);
        },
    );
}

fn import_texture(id: TextureId, path: &str, renderer: &mut Renderer) -> Texture {
    let image: RgbaImage = gfx::image::open(path)
        .expect(&format!("Failed to open image {}!", path))
        .to_rgba();

    let width: u32 = image.width();
    let height: u32 = image.height();
    let pixels: Vec<u8> = image.into_raw();
    renderer.create_gpu_texture(id, width, height, &pixels);

    Texture::new(id, width, height, pixels)
}
