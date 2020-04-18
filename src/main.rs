mod game;

use game::{
    audio::{AudioAssetDb, AudioAssetId},
    physics::PhysicsState,
    render::RenderState,
    resources::*,
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
    let window_width: u32 = 640;
    let window_height: u32 = 480;
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
                    game::resources::TEX_SPRITESHEET_BUTTONS,
                    "res/textures/sprites.png",
                    renderer,
                );
                import_texture(game::resources::TEX_FONT, "res/textures/font.png", renderer);
                import_texture(
                    game::resources::TEX_BG_WORKSTATION,
                    "res/textures/workstation-bg.png",
                    renderer,
                );
                import_texture(
                    game::resources::TEX_BG_LAB,
                    "res/textures/lab-bg.png",
                    renderer,
                );
                import_texture(
                    game::resources::TEX_BG_GLASS,
                    "res/textures/glass-bg.png",
                    renderer,
                );
                import_texture(
                    game::resources::TEX_SPRITESHEET_ALIEN,
                    "res/textures/alien-sprites.png",
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

            game.world.maintain();
        },
        move |game, _ticks, lerp, window, renderer| {
            game.world.write_resource::<PhysicsState>().lerp = lerp;

            let mut render = game.world.write_resource::<RenderState>();

            // Day text (todo move this)
            render.bind_color(COLOR_WHITE);
            render.bind_layer(game::layers::LAYER_UI);
            render.bind_transparency(Transparency::Transparent);
            render.bind_texture(game::resources::TEX_FONT);
            render.text(8.0, 8.0, 8, 16, 1.25, "Day 1");

            // FPS text
            let msg = format!("FPS: {}", window.fps);
            render.bind_color(COLOR_WHITE);
            render.bind_layer(game::layers::LAYER_UI);
            render.bind_transparency(Transparency::Transparent);
            render.bind_texture(game::resources::TEX_FONT);
            let fps_text_x = window_width as f32 - (msg.len() as f32 * 8.0) - 2.0;
            render.text(fps_text_x, 2.0, 8, 16, 1.0, &msg);

            // Lab Background Layer
            render.bind_color(COLOR_WHITE);
            render.bind_layer(game::layers::LAYER_BG_LAB);
            render.bind_transparency(Transparency::Opaque);
            render.bind_texture(game::resources::TEX_BG_LAB);
            render.textured_quad((0.0, 480.0), (640.0, 480.0), (0.0, 0.0), (640.0, 0.0));

            // Glass Background Layer
            render.bind_color(COLOR_WHITE);
            render.bind_layer(game::layers::LAYER_BG_GLASS);
            render.bind_transparency(Transparency::Opaque);
            render.bind_texture(game::resources::TEX_BG_GLASS);
            render.textured_quad((0.0, 480.0), (640.0, 480.0), (0.0, 0.0), (640.0, 0.0));

            // Workstation Background Layer
            render.bind_color(COLOR_WHITE);
            render.bind_layer(game::layers::LAYER_BG_WORKSTATION);
            render.bind_transparency(Transparency::Opaque);
            render.bind_texture(game::resources::TEX_BG_WORKSTATION);
            render.textured_quad((0.0, 480.0), (640.0, 480.0), (0.0, 0.0), (640.0, 0.0));

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
