use crate::{input::InputState, renderer::Renderer};
use ::winit::{
    dpi::LogicalSize,
    event::Event as WinitEvent,
    event::WindowEvent as WinitWindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::time::{Duration, Instant};

pub use ::winit::window::Window as WinitWindow;

const SIXTY_FPS_DT: f64 = 1.0 / 60.0;

pub struct WindowState {
    pub fps: u32,
    pub window_scale: f32,
    pub dpi_scale_factor: f32,
}

pub type DeltaTime = f64;

pub fn run<T>(
    title: &str,
    width: u32,
    height: u32,
    render_scale: f32,
    app_state: T,
    init_callback: impl FnMut(&mut T, &mut Renderer) + 'static,
    tick_callback: impl FnMut(&mut T, &WindowState, &InputState, DeltaTime) + 'static,
    render_callback: impl FnMut(&T, u128, f64, &WindowState, &mut Renderer) + 'static,
) where
    T: 'static,
{
    let event_loop = EventLoop::new();
    let window_size = LogicalSize::new(
        (width as f32 * render_scale) as u32,
        (height as f32 * render_scale) as u32,
    );
    let window: WinitWindow = WindowBuilder::new()
        .with_title(title)
        .with_min_inner_size(window_size)
        .with_inner_size(window_size)
        .with_resizable(false)
        .build(&event_loop)
        .expect("Failed to create window!");

    let mut init_callback = Box::new(init_callback);
    let mut tick_callback = Box::new(tick_callback);
    let mut render_callback = Box::new(render_callback);

    let mut app_state: T = app_state;
    let mut renderer: Renderer = Renderer::new(&window, render_scale);
    let mut input_state: InputState = InputState::new();
    let mut window_state = WindowState {
        fps: 0,
        window_scale: render_scale,
        dpi_scale_factor: window.scale_factor() as f32,
    };

    let one_second: Duration = Duration::from_secs(1);
    let mut fps_timer: Duration = Duration::from_secs(0);
    let mut fps_counter: u32 = 0;

    let target_dt: f64 = SIXTY_FPS_DT;
    let mut time: f64 = 0.0;
    let mut current_time = Instant::now();
    let mut accumulator: f64 = 0.0;
    let mut frame_time: Duration = Duration::from_secs(0);

    let mut ticks: u128 = 0;

    init_callback(&mut app_state, &mut renderer);
    renderer.rebuild_swapchain();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            WinitEvent::WindowEvent { event, window_id } => match event {
                WinitWindowEvent::CloseRequested => {
                    if window_id == window.id() {
                        *control_flow = ControlFlow::Exit
                    }
                }
                WinitWindowEvent::Resized(size) => {
                    println!("[Window] Resized to ({}, {})", size.width, size.height);

                    renderer.resize(size.width, size.height);
                    window.request_redraw();
                }
                WinitWindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    println!(
                        "[Window] Scale factor changed to {}. New inner size = {:?}",
                        scale_factor, new_inner_size
                    );

                    window_state.dpi_scale_factor = scale_factor as f32;
                    renderer.resize(new_inner_size.width, new_inner_size.height);
                    window.request_redraw();
                }
                WinitWindowEvent::KeyboardInput {
                    input,
                    is_synthetic,
                    ..
                } => {
                    if is_synthetic {
                        // Synthetic key press events are generated for all keys pressed when a window gains focus.
                        // Likewise, synthetic key release events are generated for all keys pressed when a window goes out of focus.
                        // Ignore these.
                        return;
                    }

                    input_state.handle_keyboard_input(&input);
                }
                _ => {}
            },
            WinitEvent::MainEventsCleared => {
                let new_time = Instant::now();
                frame_time = new_time - current_time;
                frame_time = frame_time.min(std::time::Duration::from_secs_f64(0.1));
                current_time = new_time;

                let dt = frame_time.as_secs_f64();
                accumulator += dt;
                while accumulator >= target_dt {
                    tick_callback(&mut app_state, &window_state, &input_state, dt);
                    input_state.clear_pressed_and_released();

                    accumulator -= target_dt;
                    time += target_dt;
                    ticks += 1;
                    fps_counter += 1;
                }

                fps_timer = fps_timer + frame_time;
                if fps_timer >= one_second {
                    fps_timer = std::time::Duration::from_secs(0);
                    window_state.fps = fps_counter;
                    fps_counter = 0;
                }

                let lerp = accumulator / target_dt;
                render_callback(&app_state, ticks, lerp, &window_state, &mut renderer);
                window.request_redraw();
            }
            _ => (),
        }
    });
}
