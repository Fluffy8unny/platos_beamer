use glium;
use glium::implement_vertex;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

implement_vertex!(Vertex, position, uv);

pub fn get_quad_buffer(x_size: (f32, f32), y_size: (f32, f32)) -> [Vertex; 4] {
    [
        Vertex {
            position: [x_size.0, y_size.0],
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [x_size.1, y_size.0],
            uv: [1.0, 0.0],
        },
        Vertex {
            position: [x_size.1, y_size.1],
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [x_size.0, y_size.1],
            uv: [0.0, 1.0],
        },
    ]
}

pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

pub struct BufferCollection {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
}
