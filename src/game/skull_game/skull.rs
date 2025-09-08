use rand::{Rng, rng};

use crate::{display::timestep::TimeStep, game::skull_game::config::SkullSettings};
use opencv::{Result, core::Range, prelude::*};

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

    pub direction: (f32, f32),
    pub max_scale: f32,
    pub hitable_from: f32,
    pub scale_speed: f32,
    pub move_speed: f32,
    pub threshold: f32,
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
        Range::new(
            convert(-skull.center.1 - radius, dims.0),
            convert(-skull.center.1 + radius, dims.0),
        )?,
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let time_delta_s = timestep.time_delta / 1000_f32;
        let new_scale = self.scale + time_delta_s * self.scale_speed;
        let new_center = (
            self.center.0 + time_delta_s * self.direction.0 * self.move_speed,
            self.center.1 + time_delta_s * self.direction.1 * self.move_speed,
        );
        self.center = new_center;
        self.scale = new_scale.clamp(0_f32, self.max_scale);

        match self.state {
            SkullState::Incomming => {
                if self.scale > self.hitable_from {
                    self.state = SkullState::Hitable;
                }
            }
            SkullState::Hitable => {
                if let Some(mask_val) = mask {
                    if hit_test(self, &mask_val)? {
                        self.state = SkullState::Killed;
                    }
                }

                if self.scale >= self.max_scale {
                    self.state = SkullState::Survived;
                }
            }
            _ => {}
        };
        Ok(())
    }
}

pub struct SkullSpawner {
    pub time_since: f32,
    pub settings: SkullSettings,
}

impl SkullSpawner {
    pub fn maybe_spawn(&mut self, skulls: &mut Vec<Skull>, timestep: &TimeStep) {
        self.time_since += timestep.time_delta / 1000.0;
        println!("{:?},{:?}", skulls.len(), self.settings.max_number);
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
            let pos_magnitude = (x_pos * x_pos + y_pos * y_pos).sqrt();
            let dir = (x_pos / pos_magnitude, y_pos / pos_magnitude);

            let new_skull = Skull {
                center: (x_pos, y_pos),
                scale: self.settings.start_scale,
                rotation,
                state: SkullState::Incomming,
                hitable_from: self.settings.hitable_from,
                max_scale: self.settings.max_scale,
                direction: dir,
                scale_speed: self.settings.scale_speed,
                move_speed: self.settings.move_speed,
                threshold: self.settings.threshold,
            };
            skulls.push(new_skull);
            self.time_since -= self.settings.spawn_rate;
        }
    }
}
