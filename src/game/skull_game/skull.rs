use std::intrinsics::sqrtf128;

use rand::{thread_rng, Rng};
use toml::value::Time;

use crate::{display::timestep::{self, TimeStep}, game::skull_game::{config::SkullSettings, skull}};
use opencv::{Result, core::Range, prelude::*};

pub enum SkullState {
    Incomming,
    Hitable,
    Killed,
    Survived,
    ToRemove,
}

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
        .rowscols(bounding_box.1, bounding_box.0)?
        .clone_pointee();
    let values_in_mask = submat
        .data_bytes()?
        .iter()
        .fold(0_f32, |sum, data| (*data as f32) + sum);
    Ok(values_in_mask / (skull.scale * skull.scale) >= skull.threshold)
}

pub fn get_bounding_box(skull: &Skull, dims: (i32, i32)) -> Result<(Range, Range)> {
    let convert =
        |rel_val: f32, img_size: i32| ((img_size as f32) * (rel_val + 1_f32) / 2_f32) as i32;
    let radius = skull.scale / 2_f32;
    Ok((
        Range::new(
            convert(skull.center.0 - radius, dims.0),
            convert(skull.center.0 + radius, dims.0),
        )?,
        Range::new(
            convert(skull.center.1 - radius, dims.1),
            convert(skull.center.1 + radius, dims.1),
        )?,
    ))
}

impl Skull{
    pub fn update(&mut self, mask: &Option<Mat>, timestep: &TimeStep) {
        let time_delta_s = timestep.time_delta/1000_f32;
        let new_scale = skull.scale + time_delta_s * skull.scale_speed;
        let new_center = (
            skull.center.0 + time_delta_s * skull.direction.0 * skull.move_speed,
            skull.center.1 + time_delta_s * skull.direction.1 * skull.move_speed,
        );
        self.center = new_center;
        self.scale = new_scale.clamp(0_f32,skull.max_scale);
        
        match self.state {
            SkullState::Incomming => {
                if self.scale > self.hitable_from {
                    self.state = SkullState::Hitable;
                }
                Ok(res)
            },
            SkullState::Hitable => {
                if hit_test(&self, mask)? {
                    self.state = SkullState::Killed;
                }else if res.scale > res.max_scale {
                    self.state = SkullState::Survived;
                }
            },
            _ =>{}
        }
    }
}

struct SkullSpawner{
    time_since : f32,
    settings   : SkullSettings,
}

impl SkullSpawner{
    fn maybe_spawn(&mut self, skulls: &mut Vec<Skull>, timestep: TimeStep){
        self.time_since += timestep.time_delta;
        if skulls.len()>self.settings.max_number{
            return;
       } 

       let randomizer = thread_rng();
       while self.time_since > self.settings.spawn_rate {
        let x_pos : f32 = randomizer.gen_range( self.settings.x_start.0, self.settings.x_start.1);
        let y_pos : f32 = randomizer.gen_range( self.settings.y_start.0, self.settings.y_start.1);
        let rotation : f32 = randomizer.gen_range( self.settings.rot.0, self.settings.rot.1);
        let pos_magnitude = (x_pos*x_pos + y_pos*y_pos).sqrt();
        let dir = (x_pos/pos_magnitude, y_pos/pos_magnitude);
        
        let new_skull = Skull {
            center: (x_pos,y_pos),
            scale: self.settings.start_scale,
            rotation: rot,
            state: SkullState::Incomming,
            hitable_from: self.settings.hitable_from,
            max_scale: self.settings.max_scale,
            direction: dir,
            scale_speed: self.settings.scale_speed,
            move_speed: self.settings.move_speed,
            threshold: self.settings.threshold,
        };
        skulls.push(new_skull);
        self.time_since -= self.spawn_rate;
       }


    }
}