use opencv::prelude::*;

use crate::PlatoConfig;
use crate::display::minimap::Minimap;
use crate::threads::try_sending;
use crate::types::{GameTrait, thread_types::*};
use std::sync::mpsc::{Receiver, SyncSender};
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
        let mut app = PlatoApp {
            pipeline_control_queue: pipeline_control_queue.clone(),
            window,
            display,
            minimap,
            game,
            config,
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
    }

    fn update(&mut self, mask: Mat) -> Result<(), Box<dyn std::error::Error>> {
        self.minimap.update_texture(&mask, &self.display)?;
        self.game.update(&mask, &self.display)?;
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut frame = self.display.draw();
        clear_frame(&mut frame);
        self.game.draw(&mut frame)?;
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
        println!("{event:?}");
        if let WindowEvent::KeyboardInput {
            event: KeyEvent { logical_key, .. },
            ..
        } = &event
        {
            self.game.key_event(&logical_key);
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
                Key::Character(val) if val == self.config.key_config.quit_key => {
                    send_pipeline_msg(&self.pipeline_control_queue, PipelineMessage::Quit);
                    event_loop.exit();
                }
                Key::Character(val) if val == self.config.key_config.reset_key => {
                    send_pipeline_msg(&self.pipeline_control_queue, PipelineMessage::SetReference);
                    self.reset();
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

    //init pipeline, so defaults will be available
    send_pipeline_msg(&pipeline_control_queue, PipelineMessage::SetReference);

    loop {
        //ask for image every frame. This way it'll be ready asap,
        //since we inited b4 the loop.
        send_pipeline_msg(&pipeline_control_queue, PipelineMessage::GenerateImage);
        //check if we got updates from the camera
        match result_queue.recv() {
            Ok(result) => match result {
                Ok(mask) => {
                    app.update(mask)?;
                }
                Err(error) => {
                    eprintln!("Window thread reuslt_queue. Received frame is error {error}")
                }
            },
            Err(error) => eprintln!(
                "Receiver error(Window thread, result_queue. COuld not receive frame {error})"
            ),
        }
        //draw everything and swap buffers
        app.draw()?;

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
