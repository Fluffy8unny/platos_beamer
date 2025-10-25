use crate::display::display_window::DisplayType;
use crate::display::primitves::{QUAD_INDICES, Vertex, get_quad_buffer};
use crate::game::util::image_to_gray_texture_r;
use glium::{IndexBuffer, VertexBuffer};
use opencv::prelude::*;
use std::sync::{Arc, Mutex};

pub struct LiveViewData {
    pub live_view_vb: VertexBuffer<Vertex>,
    pub live_view_ib: IndexBuffer<u16>,
    pub live_view_texture: Arc<Mutex<Option<glium::Texture2d>>>,
}

impl LiveViewData {
    pub fn generate_vertex_index_buffer(
        display: &DisplayType,
    ) -> Result<LiveViewData, Box<dyn std::error::Error>> {
        let verticies = get_quad_buffer((-1_f32, 1_f32), (-1_f32, 1_f32));
        let live_view_vb = VertexBuffer::new(display, &verticies)?;
        let live_view_ib = IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &QUAD_INDICES,
        )?;
        Ok(LiveViewData {
            live_view_vb,
            live_view_ib,
            live_view_texture: Arc::new(Mutex::new(None)),
        })
    }

    pub fn set_live_view_texture(
        &mut self,
        display: &DisplayType,
        live_img: &Mat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        *self.live_view_texture.lock().unwrap() = Some(image_to_gray_texture_r(display, live_img)?);
        Ok(())
    }
}
