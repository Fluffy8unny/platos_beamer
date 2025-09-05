use crate::display::timestep::TimeStep;
use opencv::{Result, core::Range, prelude::*};

pub enum SkullState {
    Incomming,
    Hitable,
    Killed,
    Survived,
}

pub struct Skull {
    pub center: (f32, f32),
    pub scale: f32,
    pub rotation: f32,
    pub state: SkullState,

    pub direction: (f32, f32),
    pub max_scale: f32,
    pub hitable_from: f32,
    pub threshold: f32,
    pub speed: f32,
}

pub fn get_bounding_box(skull: &Skull) -> Result<(Range, Range)> {
    let radius = skull.scale / 2_f32;
    Ok((
        Range::new(
            (skull.center.0 - radius) as i32,
            (skull.center.0 + radius) as i32,
        )?,
        Range::new(
            (skull.center.1 - radius) as i32,
            (skull.center.1 + radius) as i32,
        )?,
    ))
}

pub fn hit_test(skull: &Skull, mask: &Mat) -> Result<bool> {
    let bounding_box = get_bounding_box(skull)?;
    let submat = mask.rowscols(bounding_box.0, bounding_box.1)?;
    let values_in_mask = submat
        .data_bytes()?
        .iter()
        .fold(0_f32, |sum, data| (*data as f32) + sum);
    Ok(values_in_mask / (skull.scale * skull.scale) >= skull.threshold)
}

pub fn update(skull: Skull, mask: &Mat, timestep: &TimeStep) -> Result<Skull> {
    let new_scale = skull.scale + timestep.time_delta * skull.speed;
    let new_center = (
        skull.center.0 + timestep.time_delta * skull.direction.0,
        skull.center.1 + timestep.time_delta * skull.direction.1,
    );
    let mut res = Skull {
        center: new_center,
        scale: new_scale,
        ..skull
    };

    match res.state {
        SkullState::Incomming => {
            if res.scale > res.hitable_from {
                res.state = SkullState::Hitable;
            }
            Ok(res)
        }
        SkullState::Hitable => {
            if hit_test(&res, mask)? {
                res.state = SkullState::Killed;
                return Ok(res);
            }
            if res.scale > res.max_scale {
                res.state = SkullState::Survived;
            }
            Ok(res)
        }
        _ => Ok(res),
    }
}
