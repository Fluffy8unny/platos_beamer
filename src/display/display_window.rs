use opencv::prelude::*;

use crate::PlatoConfig;
use crate::display::minimap::Minimap;
use crate::display::timestep::TimeStep;
use crate::threads::try_sending;
use crate::types::{GameTrait, thread_types::*};
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError};
use std::time::Duration;

extern crate glium;
use glium::Surface;
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
    minimap: Minimap,
    game: Box<dyn GameTrait>,
    config: PlatoConfig,
    timestep: TimeStep,
}

impl PlatoApp {
    fn new(
        pipeline_control_queue: SyncSender<PipelineMessage>,
        event_loop: &EventLoop<()>,
        game: Box<dyn GameTrait>,
        config: PlatoConfig,
    ) -> Result<PlatoApp, Box<dyn std::error::Error>> {
        let (window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(event_loop);
        let minimap = Minimap::new(&display, &config)?;
        let timestep = TimeStep::new();
        let mut app = PlatoApp {
            pipeline_control_queue: pipeline_control_queue.clone(),
            window,
            display,
            minimap,
            game,
            config,
            timestep,
        };
        app.init()?;
        Ok(app)
    }

    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //init game state
        self.game.init(&self.display, self.config.clone())?;
        Ok(())
    }

    fn reset(&mut self) {
        self.game.reset();
        self.timestep.reset();
    }

    fn update(&mut self, image: Mat, mask: Mat) -> Result<(), Box<dyn std::error::Error>> {
        self.minimap.update_texture(&image, &mask, &self.display)?;
        self.game.update(&image, &mask, &self.display)?;
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.timestep.update();
        let mut frame = self.display.draw();
        clear_frame(&mut frame);
        self.game.draw(&mut frame, &self.display, &self.timestep)?;
        if self.config.minimap_config.show {
            self.minimap.draw(&mut frame)?;
        }

        frame.finish()?;
        Ok(())
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
        if let WindowEvent::KeyboardInput {
            event: KeyEvent { logical_key, .. },
            ..
        } = &event
        {
            self.game.key_event(logical_key);
        }

        match &event {
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
                Key::Character(val) if val.to_lowercase() == self.config.key_config.quit_key => {
                    send_pipeline_msg(&self.pipeline_control_queue, PipelineMessage::Quit);
                    event_loop.exit();
                }
                Key::Character(val) if val.to_lowercase() == self.config.key_config.reset_key => {
                    send_pipeline_msg(&self.pipeline_control_queue, PipelineMessage::SetReference);
                    self.reset();
                }
                Key::Character(val)
                    if val.to_lowercase() == self.config.key_config.toggle_minimap_key =>
                {
                    self.config.minimap_config.show = !self.config.minimap_config.show;
                }
                _ => (),
            },
            _ => (),
        }
    }
}

pub fn clear_frame(frame: &mut glium::Frame) {
    frame.clear_color(0_f32, 0_f32, 0_f32, 1_f32);
}

pub fn start_display(
    pipeline_control_queue: SyncSender<PipelineMessage>,
    result_queue: Receiver<BackgroundResult>,
    game: Box<dyn GameTrait>,
    config: PlatoConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    let mut app = PlatoApp::new(
        pipeline_control_queue.clone(),
        &event_loop,
        game,
        config.clone(),
    )?;

    //the camera is way slower than the actual renderer. We shouldn't call render, until we
    //actually got a real image, and are sure everything is updated. In theory this could be
    //handled by Optionals and exceptions, but this way is easier and more natural
    let mut got_image = false;

    //init pipeline, so defaults will be available
    send_pipeline_msg(&pipeline_control_queue, PipelineMessage::SetReference);
    send_pipeline_msg(&pipeline_control_queue, PipelineMessage::GenerateImage);
    loop {
        match result_queue.try_recv() {
            Ok(result) => {
                send_pipeline_msg(&pipeline_control_queue, PipelineMessage::GenerateImage);
                match (result.image, result.mask) {
                    (Ok(image), Ok(mask)) => {
                        got_image = true;
                        app.update(image, mask)?;
                    }
                    (Err(error), _) => {
                        eprintln!("Window thread reuslt_queue. Received image is error {error}")
                    }
                    (_, Err(error)) => {
                        eprintln!("Window thread reuslt_queue. Received mask is error {error}")
                    }
                }
            }
            Err(error) => match error {
                TryRecvError::Empty => (),
                TryRecvError::Disconnected => eprint!(
                    "Receiver error(Window thread, result_queue. Could not receive frame {error}"
                ),
            },
        }

        //draw everything and swap buffers
        if got_image {
            app.draw()?;
        }

        //handle window events
        let status = event_loop.pump_app_events(Some(Duration::ZERO), &mut app);

        //end this whole mess if we're told to do so. This has to be done AFTER
        //all frame stuff happend
        if let PumpStatus::Exit(exit_code) = status {
            send_pipeline_msg(&pipeline_control_queue, PipelineMessage::Quit);
            println!("Quitting window gracefully with exit code {:?}", exit_code);
            return Ok(());
        }
    }
}
