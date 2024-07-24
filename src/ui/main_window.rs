use std::sync::atomic;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use pixels::Pixels;
use pixels::SurfaceTexture;
use winit::dpi::PhysicalSize;
use winit::window::WindowBuilder;
use winit::event_loop::EventLoopBuilder;
use winit::event_loop::EventLoopProxy;
use winit::event_loop::ControlFlow;
use winit::event::*;

use crate::config::Config;
use crate::gpu::types::VideoResolution;
use crate::hart_clock::HART_CLOCK_MASTER;
use crate::interrupt_controller::INTERRUPT_CONTROLLER;
use crate::interrupt_controller::InterruptType;
use crate::machine::Machine;
use crate::input::*;

#[derive(Clone)]
enum WindowMessage {
    Exit,
    SetVideoResolution(VideoResolution),
    PresentTexture(*const u8, u32, bool, Arc<Machine>),
}

unsafe impl Send for WindowMessage {}

#[derive(Clone)]
pub struct MainWindow {
    event_proxy: EventLoopProxy<WindowMessage>
}

impl MainWindow {
    pub fn run<F: FnMut(Self) + Send + 'static>(config: &Config, mut application_thread: F) -> ! {
        let event_loop = EventLoopBuilder::<WindowMessage>::with_user_event()
            .build();
        let event_proxy = event_loop.create_proxy();

        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new((512.0 * config.ui_scale) as u32, (384.0 * config.ui_scale) as u32))
            .with_resizable(false)
            .with_title("RVFM")
            .build(&event_loop)
            .unwrap();

        let main_window = MainWindow {
            event_proxy
        };

        let mut video_resolution = VideoResolution::V256x192;
        let mut pixels = {
            let (w, h) = video_resolution.as_w_h();
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(w, h, surface_texture).expect("Failed to create pixels renderer")
        };

        std::thread::spawn(move || application_thread(main_window));

        let mut t_last_frame = Instant::now();

        event_loop.run(move |event, _window_target, control_flow| {
            match event {
                Event::MainEventsCleared => {
                    let t_now = Instant::now();
                    let dt_frame = t_now - t_last_frame;
                    let remaining_micros = 16666 - dt_frame.as_micros() as i64;
                    if remaining_micros > 0 {
                        std::thread::sleep(Duration::from_micros(remaining_micros as u64));
                    }
                    t_last_frame = Instant::now();
                    window.request_redraw();
                },
                Event::RedrawRequested(_window_id) => {
                    HART_CLOCK_MASTER.next_frame();
                }
                Event::UserEvent(WindowMessage::Exit) => {
                    *control_flow = ControlFlow::ExitWithCode(0);
                },
                Event::UserEvent(WindowMessage::SetVideoResolution(new_resolution)) => {
                    if video_resolution != new_resolution {
                        video_resolution = new_resolution;
                        let (w, h) = video_resolution.as_w_h();
                        pixels.resize_buffer(w, h).expect("Failed to resize pixel buffer");
                    }
                },
                Event::UserEvent(WindowMessage::PresentTexture(texture_data, completion_address, interrupt, machine)) => {
                    let size = video_resolution.pixel_count() * 4;
                    let texture_data_slice = unsafe { std::slice::from_raw_parts(texture_data, size) };
                    pixels.frame_mut().copy_from_slice(texture_data_slice);
                    println!("writing present completion: {:010X}", completion_address);
                    machine.write_u32(completion_address, 1);
                    atomic::fence(atomic::Ordering::Release);
                    if interrupt {
                        println!("triggering present interrupt!");
                        INTERRUPT_CONTROLLER.trigger_interrupt(InterruptType::Present);
                    }
                    pixels.render().expect("Failed to render screen");
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::ExitWithCode(0);
                        },
                        WindowEvent::KeyboardInput { input, .. } => {
                            input_keyboard_event_handler(input);
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        })
    }

    pub fn exit(&self) {
        let _ = self.event_proxy.send_event(WindowMessage::Exit);
    }

    pub fn present_texture(&self, texture: *const u8, completion_addr: u32, interrupt: bool, machine: Arc<Machine>) {
        let _ = self.event_proxy.send_event(WindowMessage::PresentTexture(texture, completion_addr, interrupt, machine));
    }

    pub fn set_video_resolution(&self, resolution: VideoResolution) {
        let _ = self.event_proxy.send_event(WindowMessage::SetVideoResolution(resolution));
    }
}
