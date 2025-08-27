use glium::Surface;
use opencv::Result;
use opencv::prelude::*;

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
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{Window, WindowId};
pub type DisplayType = glium::Display<glium::glutin::surface::WindowSurface>;
struct PlatoApp {
    pipeline_control_queue: SyncSender<PipelineMessage>,
    window: Option<Window>,
    display: Option<DisplayType>,
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("{event:?}");

        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                window.request_redraw();
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

pub fn start_display(
    pipeline_control_queue: SyncSender<PipelineMessage>,
    result_queue: Receiver<Result<Mat>>,
) -> Result<()> {
    //init pipeline, so defaults will be available
    send_pipeline_msg(&pipeline_control_queue, PipelineMessage::SetReference);

    let mut event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    //let window_attributes = Window::default_attributes().with_title("A fantastic window!");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    let mut app = PlatoApp {
        pipeline_control_queue: pipeline_control_queue.clone(),
        window: Some(window),
        display: Some(display),
    };
    let timeout = Some(Duration::ZERO);
    loop {
        //ask for image every frame. This way it'll be ready asap,
        //since we inited b4 the loop.
        try_sending(
            &pipeline_control_queue,
            PipelineMessage::GenerateImage,
            "window thread",
            "pipeline_control_queue",
        );

        //check if we got updates from the camera
        match result_queue.recv() {
            Ok(result) => match result {
                Ok(mat) => {
                    let mut frame = app.display.as_ref().unwrap().draw();
                    frame.clear_color(1_f32, 1_f32, 1_f32, 1_f32);
                    frame.finish();
                }
                Err(error) => {
                    eprintln!("Window thread reuslt_queue. Received frame is error {error}")
                }
            },
            Err(error) => eprintln!(
                "Receiver error(Window thread, result_queue. COuld not receive frame {error})"
            ),
        }

        //handle user input&&draw frame
        let status = event_loop.pump_app_events(timeout, &mut app);
        if let PumpStatus::Exit(exit_code) = status {
            send_pipeline_msg(&pipeline_control_queue, PipelineMessage::Quit);
            println!("Quitting window gracefully with exit code {:?}", exit_code);
            return Ok(());
        }
    }
}
