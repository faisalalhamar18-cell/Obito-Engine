use std::os::raw::c_char;
use std::ffi::CStr;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::Graphics_AAA::GraphicsContext;

struct ObitoApp {
    title: String,
    width: u32,
    height: u32,
    window: Option<Arc<Window>>,
    graphics_context: Option<GraphicsContext>,
}

impl ApplicationHandler for ObitoApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(&self.title)
                .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));

            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    println!("Obito Engine: AAA Window Created Successfully!");
                    let window_arc = Arc::new(window);
                    self.window = Some(window_arc.clone());

                    let gfx = pollster::block_on(GraphicsContext::new(window_arc));
                    self.graphics_context = Some(gfx);
                    println!("Obito Engine: wgpu Graphics Pipeline Initialized successfully!");
                }
                Err(err) => {
                    eprintln!("Obito Engine Error: Failed to create window: {:?}", err);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Obito Engine: Closing Window...");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(gfx) = &mut self.graphics_context {
                    gfx.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(gfx) = &mut self.graphics_context {
                    match gfx.render() {
                        Ok(_) => {}
                        Err(err_msg) => {
                            // إذا فُقد السطح نقوم بإعادة الحجم، وإذا نفدت الذاكرة نغلق بأمان
                            if err_msg.contains("Lost") {
                                if let Some(w) = &self.window {
                                    gfx.resize(w.inner_size());
                                }
                            } else if err_msg.contains("OutOfMemory") {
                                eprintln!("Obito Engine Error: GPU Out of memory, exiting...");
                                event_loop.exit();
                            } else {
                                eprintln!("Render error: {}", err_msg);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_obito_window(title: *const c_char, width: u32, height: u32) {
    let c_str = unsafe {
        if title.is_null() {
            CStr::from_bytes_with_nul(b"Obito Engine\0").unwrap()
        } else {
            CStr::from_ptr(title)
        }
    };
    let window_title = c_str.to_str().unwrap_or("Obito Engine").to_string();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ObitoApp {
        title: window_title,
        width,
        height,
        window: None,
        graphics_context: None,
    };

    println!("Starting Obito Engine Loop...");
    let _ = event_loop.run_app(&mut app);
}