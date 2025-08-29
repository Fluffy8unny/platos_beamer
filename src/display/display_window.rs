use glium::Surface;
use opencv::prelude::*;

use crate::display::minimap::create_minimap;
use crate::threads::try_sending;
use crate::types::thread_types::*;

use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

extern crate glium;
// Use the re-exported winit dependency to avoid version mismatches.
// Requires the `simple_window_builder` feature.
use glium::winit;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event::{ElementState, KeyEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::Key;
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowId};

pub type DisplayType = glium::Display<glium::glutin::surface::WindowSurface>;

struct PlatoApp {
    pipeline_control_queue: SyncSender<PipelineMessage>,
    window: Window,
    display: DisplayType,
}

impl PlatoApp {
    fn new(
        pipeline_control_queue: SyncSender<PipelineMessage>,
        event_loop: &EventLoop<()>,
    ) -> PlatoApp {
        let (window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(event_loop);
        PlatoApp {
            pipeline_control_queue: pipeline_control_queue.clone(),
            window,
            display,
        }
    }
}

fn send_pipeline_msg(pipeline_control_queue: &SyncSender<PipelineMessage>, msg: PipelineMessage) {
    try_sending(
        pipeline_control_queue,
        msg,
        "window thread",
        "pipeline_control_queue",
    );
}

impl ApplicationHandler for PlatoApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("{event:?}");

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key.as_ref() {
                Key::Character("Q") => {
                    send_pipeline_msg(&self.pipeline_control_queue, PipelineMessage::Quit);
                    event_loop.exit();
                }
                Key::Character("R") => {
                    send_pipeline_msg(&self.pipeline_control_queue, PipelineMessage::SetReference);
                }
                _ => (),
            },
            _ => (),
        }
    }
}

pub fn clear_frame(frame: &mut glium::Frame) {
    frame.clear_color(1_f32, 0_f32, 0_f32, 1_f32);
}

pub fn start_display(
    pipeline_control_queue: SyncSender<PipelineMessage>,
    result_queue: Receiver<opencv::Result<Mat>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    let mut app = PlatoApp::new(pipeline_control_queue.clone(), &event_loop);
    let timeout = Some(Duration::ZERO);
    let mut minimap = create_minimap(&app.display)?;

    //init pipeline, so defaults will be available
    send_pipeline_msg(&pipeline_control_queue, PipelineMessage::SetReference);
    loop {
        //ask for image every frame. This way it'll be ready asap,
        //since we inited b4 the loop.
        send_pipeline_msg(&pipeline_control_queue, PipelineMessage::GenerateImage);
        //check if we got updates from the camera
        match result_queue.recv() {
            Ok(result) => match result {
                Ok(mat) => {
                    minimap.update_texture(&mat, &app.display)?;
                    //update game state
                }
                Err(error) => {
                    eprintln!("Window thread reuslt_queue. Received frame is error {error}")
                }
            },
            Err(error) => eprintln!(
                "Receiver error(Window thread, result_queue. COuld not receive frame {error})"
            ),
        }

        let mut frame = app.display.draw();
        clear_frame(&mut frame);
        minimap.draw(&mut frame)?;
        //draw game
        frame.finish()?;

        //handle window events
        let status = event_loop.pump_app_events(timeout, &mut app);

        //end this whole mess if we're told to do so. This has to be done AFTER \
        //all frame stuff happend
        if let PumpStatus::Exit(exit_code) = status {
            send_pipeline_msg(&pipeline_control_queue, PipelineMessage::Quit);
            println!("Quitting window gracefully with exit code {:?}", exit_code);
            return Ok(());
        }
    }
}
