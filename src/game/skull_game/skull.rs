use opencv::{Result, core::Range, prelude::*};
use rand::{Rng, rng};

use crate::display::display_window::DisplayType;
use crate::game::skull_game::util::generate_index_for_quad;
use crate::{display::timestep::TimeStep, game::skull_game::config::SkullSettings};

use ::glium::{IndexBuffer, VertexBuffer};
use glium::implement_vertex;
use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Copy)]
pub enum SkullState {
    Incomming,
    Hitable,
    Killed,
    Survived,
    ToRemove,
}

#[derive(Debug, Clone, Copy)]
pub struct Skull {
    pub center: (f32, f32),
    pub scale: f32,
    pub rotation: f32,
    pub state: SkullState,

    pub max_scale: f32,
    pub hitable_from: f32,
    pub scale_speed: f32,
    pub move_speed: f32,
    pub threshold: f32,
    pub timer: TimeStep,
    pub erratic_movement: f32,
    pub noise: Perlin,
}

#[derive(Copy, Clone)]
pub struct SkullVertex {
    pub position: [f32; 2],
    pub center: [f32; 2],
    pub scale: f32,
    pub uv: [f32; 2],
    pub rotation: f32,
    pub state: u32,
    pub blend_value: f32,
    pub flashing: f32,
    pub texture_id: f32,
}

implement_vertex!(
    SkullVertex,
    position,
    center,
    scale,
    uv,
    rotation,
    state,
    blend_value,
    flashing,
    texture_id
);

fn skull_state_to_id(state: &SkullState) -> u32 {
    match state {
        SkullState::Incomming => 0,
        SkullState::Hitable => 1,
        SkullState::Killed => 2,
        SkullState::Survived => 3,
        SkullState::ToRemove => 4,
    }
}

pub fn create_skull_vertex_buffer(
    skull_vb: &mut glium::VertexBuffer<SkullVertex>,
    skulls: &Vec<Skull>,
    index_buffer_data: &mut Vec<u32>,
) {
    for (i, (skull, vb_entry)) in skulls.iter().zip(skull_vb.map().chunks_mut(4)).enumerate() {
        let radius = skull.scale / 2_f32;
        let blend = (skull.scale / skull.hitable_from).clamp(0_f32, 1_f32);
        let state_id = skull_state_to_id(&skull.state);
        let texture_id = match skull.state {
            SkullState::Incomming => (skull.timer.runtime / 50_f32) % 4_f32,
            SkullState::Hitable => (skull.timer.runtime / 50_f32) % 4_f32,
            _ => (skull.timer.runtime / 50_f32) % 2_f32,
        };
        let flashing = if skull.timer.runtime % 3_f32 == 0_f32 {
            1_f32
        } else {
            0_f32
        };

        let update_vb =
            |x: f32, y: f32, u: f32, v: f32, vb_entry: &mut [SkullVertex], idx: usize| {
                vb_entry[idx].position[0] = x;
                vb_entry[idx].position[1] = y;
                vb_entry[idx].center[0] = skull.center.0;
                vb_entry[idx].center[1] = skull.center.1;
                vb_entry[idx].scale = skull.scale;
                vb_entry[idx].uv[0] = u;
                vb_entry[idx].uv[1] = v;
                vb_entry[idx].rotation = skull.rotation;
                vb_entry[idx].blend_value = blend;
                vb_entry[idx].flashing = flashing;
                vb_entry[idx].texture_id = texture_id;
                vb_entry[idx].state = state_id;
            };

        update_vb(
            skull.center.0 - radius,
            skull.center.1 + radius,
            0_f32,
            0_f32,
            vb_entry,
            0,
        );
        update_vb(
            skull.center.0 + radius,
            skull.center.1 + radius,
            1_f32,
            0_f32,
            vb_entry,
            1,
        );
        update_vb(
            skull.center.0 - radius,
            skull.center.1 - radius,
            0_f32,
            1_f32,
            vb_entry,
            2,
        );
        update_vb(
            skull.center.0 + radius,
            skull.center.1 - radius,
            1_f32,
            1_f32,
            vb_entry,
            3,
        );

        generate_index_for_quad(i, index_buffer_data);
    }
}
pub fn hit_test(skull: &Skull, mask: &Mat) -> Result<bool> {
    let dims = (mask.rows(), mask.cols());
    let bounding_box = get_bounding_box(skull, dims)?;
    let submat = mask
        .rowscols(bounding_box.0, bounding_box.1)?
        .clone_pointee();
    if submat.rows() == 0 || submat.cols() == 0 {
        return Ok(false);
    }
    let values_in_mask = submat
        .data_bytes()?
        .iter()
        .fold(0_f32, |sum, data| (*data as f32) + sum);
    Ok(values_in_mask / (skull.scale * skull.scale) >= skull.threshold)
}

pub fn get_bounding_box(skull: &Skull, dims: (i32, i32)) -> Result<(Range, Range)> {
    let convert = |rel_val: f32, img_size: i32| {
        (((img_size as f32) * (rel_val + 1_f32) / 2_f32) as i32).clamp(0, img_size - 1)
    };
    let radius = skull.scale / 2_f32;
    Ok((
        Range::new(0_i32, dims.0)?,
        Range::new(
            convert(-skull.center.0 - radius, dims.1),
            convert(-skull.center.0 + radius, dims.1),
        )?,
    ))
}

impl Skull {
    pub fn update(
        &mut self,
        mask: &Option<Mat>,
        timestep: &TimeStep,
    ) -> Result<Option<GameEvent>, Box<dyn std::error::Error>> {
        let time_delta_s = timestep.time_delta / 1000_f32;
        let new_scale = (self.scale + time_delta_s * self.scale_speed).clamp(0_f32, self.max_scale);
        let get_noise = |p| self.noise.get([(p * self.erratic_movement) as f64]) as f32;
        let (nx, ny) = (get_noise(self.center.0), get_noise(self.center.1));
        let noise_magnitude = ((nx * nx) + (ny * ny)).sqrt();
        let update_pos = |p, n| p + time_delta_s * n * self.move_speed / noise_magnitude;
        let new_center = (update_pos(self.center.0, ny), update_pos(self.center.1, nx));
        self.scale = new_scale;
        self.timer.update();

        let mut randomizer = rng();
        self.rotation += randomizer.random_range(-5.0..5.0) * time_delta_s;
        match self.state {
            SkullState::Incomming => {
                self.center = new_center;
                if self.scale > self.hitable_from {
                    self.state = SkullState::Hitable;
                }
            }
            SkullState::Hitable => {
                self.center = new_center;
                if let Some(mask_val) = mask {
                    if hit_test(self, mask_val)? {
                        self.state = SkullState::Killed;
                        self.timer.reset();
                        return Ok(Some(GameEvent::Killed {
                            pos: self.center,
                            skull_scale: self.scale,
                        }));
                    }
                }

                if self.scale >= self.max_scale {
                    self.state = SkullState::Survived;
                    self.timer.reset();
                    return Ok(Some(GameEvent::Escaped {
                        pos: self.center,
                        scale: self.scale,
                    }));
                }
            }
            SkullState::Killed => {
                if self.timer.runtime > 250_f32 {
                    self.state = SkullState::ToRemove;
                }
            }
            SkullState::Survived => {
                if self.timer.runtime > 250_f32 {
                    self.state = SkullState::ToRemove;
                }
            }
            SkullState::ToRemove => {}
        };
        Ok(None)
    }
}

pub struct SkullData {
    pub skull_vb: VertexBuffer<SkullVertex>,
    pub skull_idxb: IndexBuffer<u32>,
    pub skulls: Vec<Skull>,
}

pub fn update_skull_state(
    skulls: Vec<Skull>,
    display: &DisplayType,
) -> Result<SkullData, Box<dyn std::error::Error>> {
    let skull_count = skulls.len();

    let mut skull_vb: glium::VertexBuffer<SkullVertex> =
        glium::VertexBuffer::empty_dynamic(display, skull_count * 4)?;
    let mut index_buffer_data: Vec<u32> = Vec::with_capacity(skull_count * 6);
    //we can't map over a Vertex buffer length 0
    if skull_count > 0 {
        create_skull_vertex_buffer(&mut skull_vb, &skulls, &mut index_buffer_data);
    }

    let skull_idxb: glium::IndexBuffer<u32> = glium::IndexBuffer::new(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &index_buffer_data,
    )?;

    let res_vec = skulls
        .into_iter()
        .filter(|skull| !matches!(skull.state, SkullState::ToRemove))
        .collect();

    Ok(SkullData {
        skull_vb,
        skull_idxb,
        skulls: res_vec,
    })
}

pub enum GameEvent {
    Killed { pos: (f32, f32), skull_scale: f32 },
    Escaped { pos: (f32, f32), scale: f32 },
}

pub struct SkullSpawner {
    pub time_since: f32,
    pub settings: SkullSettings,
}

impl SkullSpawner {
    pub fn maybe_spawn(&mut self, skulls: &mut Vec<Skull>, timestep: &TimeStep) {
        self.time_since += timestep.time_delta / 1000.0;
        if skulls.len() > self.settings.max_number {
            return;
        }

        let mut randomizer = rng();
        while self.time_since > self.settings.spawn_rate && skulls.len() <= self.settings.max_number
        {
            let x_pos: f32 =
                randomizer.random_range(self.settings.x_start.0..self.settings.x_start.1);
            let y_pos: f32 =
                randomizer.random_range(self.settings.y_start.0..self.settings.y_start.1);
            let rotation: f32 = randomizer.random_range(self.settings.rot.0..self.settings.rot.1);

            let new_skull = Skull {
                center: (x_pos, y_pos),
                scale: self.settings.start_scale,
                rotation,
                state: SkullState::Incomming,
                hitable_from: self.settings.hitable_from,
                max_scale: self.settings.max_scale,
                scale_speed: self.settings.scale_speed,
                move_speed: self.settings.move_speed,
                threshold: self.settings.threshold,
                timer: TimeStep::new(),
                noise: Perlin::new(randomizer.random_range(400..1000)),
                erratic_movement: self.settings.erratic_movement,
            };
            skulls.push(new_skull);
            self.time_since -= self.settings.spawn_rate;
        }
    }
}
