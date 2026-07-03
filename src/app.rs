//! winit `ApplicationHandler` that owns the window, renderer and game, and
//! drives the update/redraw loop on both native and web.
//!
//! wgpu device creation is async. On native we simply `block_on` it inside
//! `resumed`. On web we can't block, so we build the renderer in a spawned
//! future and hand it back to the event loop as a user event.

use std::sync::Arc;

use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::data::FONT_PNG;
use crate::game::Game;
use crate::input::Input;
use crate::renderer::Renderer;

/// The user event: a fully-built renderer, delivered from the async web path.
type UserEvent = Renderer;

enum Stage {
    Uninit,
    Ready { renderer: Renderer, game: Game },
}

pub struct App {
    stage: Stage,
    input: Input,
    last: Instant,
    // Used only on the web async surface-creation path; held on native too.
    #[allow(dead_code)]
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
}

impl App {
    fn new(proxy: winit::event_loop::EventLoopProxy<UserEvent>) -> Self {
        App {
            stage: Stage::Uninit,
            input: Input::new(),
            last: Instant::now(),
            proxy,
        }
    }

    fn set_renderer(&mut self, mut renderer: Renderer) {
        let game = Game::new(&mut renderer);
        renderer.window().request_redraw();
        self.last = Instant::now();
        self.stage = Stage::Ready { renderer, game };
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if matches!(self.stage, Stage::Ready { .. }) {
            return;
        }

        // `mut` is used only by the native test-window path below.
        #[allow(unused_mut)]
        let mut attrs = Window::default_attributes()
            .with_title("Hero of the Overworld")
            .with_inner_size(LogicalSize::new(960.0, 540.0));
        // For the rustautogui e2e suite: run borderless-fullscreen so the game
        // owns the whole screen (guaranteed focus, no occlusion) and maps the
        // 320x180 canvas at a clean integer scale for reproducible screenshots.
        #[cfg(not(target_arch = "wasm32"))]
        let test_window = std::env::var("HOTO_TEST_WINDOW").is_ok();
        #[cfg(not(target_arch = "wasm32"))]
        if test_window {
            attrs = attrs
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                .with_active(true);
        }
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));

        // Actively claim keyboard focus so the rustautogui suite can drive us.
        #[cfg(not(target_arch = "wasm32"))]
        if test_window {
            window.focus_window();
        }

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            // Mount the winit canvas into the page and size it to the viewport.
            let web_window = web_sys::window().expect("no window");
            let canvas = window.canvas().expect("no canvas");
            web_window
                .document()
                .and_then(|d| d.body())
                .expect("no body")
                .append_child(&canvas)
                .expect("append canvas");
            let w = web_window
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(960.0);
            let h = web_window
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(540.0);
            let _ = window.request_inner_size(LogicalSize::new(w, h));

            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let renderer = Renderer::new(window, FONT_PNG).await;
                let _ = proxy.send_event(renderer);
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer = pollster::block_on(Renderer::new(window, FONT_PNG));
            self.set_renderer(renderer);
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, renderer: UserEvent) {
        // Web async renderer is ready.
        self.set_renderer(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Stage::Ready { renderer, game } = &mut self.stage else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                renderer.window().request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if !repeat {
                    self.input.set_key(code, state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last).as_secs_f32().min(0.05);
                self.last = now;

                game.update(&self.input, renderer, dt);
                self.input.end_frame();
                game.draw(renderer);
                // Surface loss is handled inside render() by reconfiguring.
                renderer.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Stage::Ready { renderer, .. } = &self.stage {
            renderer.window().request_redraw();
        }
    }
}

/// Build the event loop and run the app. Blocks on native; returns immediately
/// on web (the loop is driven by the browser).
pub fn run() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("build event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let proxy = event_loop.create_proxy();
    let app = App::new(proxy);

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        event_loop.run_app(&mut app).expect("run app");
    }
}
