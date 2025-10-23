use crate::display::display_window::DisplayType;
use crate::display::primitves::{QUAD_INDICES, Vertex, get_quad_buffer};

pub struct VicotryData {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
}

impl VicotryData {
    pub fn new(display: &DisplayType) -> Result<VicotryData, Box<dyn std::error::Error>> {
        let verticies = get_quad_buffer((-1_f32, 1_f32), (-1_f32, 1_f32));
        let vertex_buffer = glium::VertexBuffer::new(display, &verticies)?;
        let index_buffer = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &QUAD_INDICES,
        )?;
        Ok(VicotryData {
            vertex_buffer,
            index_buffer,
        })
    }
}
