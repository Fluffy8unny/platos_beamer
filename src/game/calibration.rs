use crate::PlatoConfig;
use crate::display::display_window::DisplayType;
use crate::display::primitves::{QUAD_INDICES, Vertex, get_quad_buffer};
use crate::display::timestep::TimeStep;
use crate::game::util::{image_to_gray_texture_r, load_shaders};
use crate::types::GameTrait;

use glium::draw_parameters::{DrawParameters, PolygonMode};
use glium::implement_vertex;
use glium::uniform;
use glium::winit::keyboard::Key;
use glium::{Surface, VertexBuffer};
use opencv::prelude::*;

pub struct CalibrationGame {
    program: Option<glium::Program>,
    live_img: Option<glium::Texture2d>,
    line_program: Option<glium::Program>,
    vertex_buffer: Option<glium::VertexBuffer<Vertex>>,
    line_buffer: Option<glium::VertexBuffer<LineVertex>>,
    index_buffer: Option<glium::IndexBuffer<u16>>,
}
#[derive(Copy, Clone)]
pub struct LineVertex {
    pub position: [f32; 2],
}

implement_vertex!(LineVertex, position,);

impl CalibrationGame {
    pub fn new() -> CalibrationGame {
        CalibrationGame {
            program: None,
            live_img: None,
            line_program: None,
            vertex_buffer: None,
            line_buffer: None,
            index_buffer: None,
        }
    }
}

fn generate_line_buffer(display: &DisplayType) -> VertexBuffer<LineVertex> {
    let vertices = vec![
        LineVertex {
            position: [-0.9, -0.9],
        },
        LineVertex {
            position: [-0.9, -0.7],
        },
        LineVertex {
            position: [-0.7, -0.8],
        },
        LineVertex {
            position: [-0.9, -0.8],
        },
        LineVertex {
            position: [-0.9, 0.9],
        },
        LineVertex {
            position: [-0.9, 0.7],
        },
        LineVertex {
            position: [-0.7, 0.8],
        },
        LineVertex {
            position: [-0.9, 0.8],
        },
        LineVertex {
            position: [0.9, -0.9],
        },
        LineVertex {
            position: [0.9, -0.7],
        },
        LineVertex {
            position: [0.7, -0.8],
        },
        LineVertex {
            position: [0.9, -0.8],
        },
        LineVertex {
            position: [0.9, 0.9],
        },
        LineVertex {
            position: [0.9, 0.7],
        },
        LineVertex {
            position: [0.7, 0.8],
        },
        LineVertex {
            position: [0.9, 0.8],
        },
        LineVertex {
            position: [-0.1, -0.1],
        },
        LineVertex {
            position: [0.1, -0.1],
        },
        LineVertex {
            position: [-0.1, 0.1],
        },
        LineVertex {
            position: [0.1, 0.1],
        },
        LineVertex {
            position: [-0.1, -0.1],
        },
        LineVertex {
            position: [-0.1, 0.1],
        },
        LineVertex {
            position: [0.1, -0.1],
        },
        LineVertex {
            position: [0.1, 0.1],
        },
    ];
    glium::VertexBuffer::new(display, &vertices).unwrap()
}

impl GameTrait for CalibrationGame {
    fn init(
        &mut self,
        display: &DisplayType,
        _config: PlatoConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let verticies = get_quad_buffer((-1_f32, 1_f32), (-1_f32, 1_f32));
        let vertex_buffer = glium::VertexBuffer::new(display, &verticies)?;
        let index_buffer = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &QUAD_INDICES,
        )?;
        let program = load_shaders("src/shaders/calibration.toml", display)?;
        let line_program = load_shaders("src/shaders/calibration_lines.toml", display)?;

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.line_buffer = Some(generate_line_buffer(display));
        self.program = Some(program);
        self.line_program = Some(line_program);
        Ok(())
    }

    fn update(
        &mut self,
        image: &Mat,
        _mask: &Mat,
        display: &DisplayType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.live_img = Some(image_to_gray_texture_r(display, image)?);
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut glium::Frame,
        _display: &DisplayType,
        _timestep: &TimeStep,
    ) -> Result<(), Box<dyn std::error::Error>> {
        frame.draw(
            self.vertex_buffer.as_ref().ok_or("no vertext buffer")?,
            self.index_buffer.as_ref().ok_or("no index buffer")?,
            self.program.as_ref().ok_or("no program")?,
            &uniform! {tex : self.live_img.as_ref().ok_or("no image")?},
            &glium::DrawParameters::default(),
        )?;

        let params = DrawParameters {
            polygon_mode: PolygonMode::Line,
            line_width: Some(5_f32),
            ..Default::default()
        };
        frame.draw(
            self.line_buffer.as_ref().ok_or("no line vertext buffer")?,
            glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
            self.line_program.as_ref().ok_or("no program")?,
            &glium::uniforms::EmptyUniforms {},
            &params,
        )?;
        Ok(())
    }

    fn key_event(&mut self, _event: &Key) {}
    fn reset(&mut self) {}
}
