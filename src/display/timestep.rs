use std::time::Instant;

type TimeFloat = f32;
type CountInteger = u64;

#[derive(Debug, Clone, Copy)]
pub struct MovingAverage {
    buffer: TimeFloat,
    alpha: TimeFloat,
}

impl MovingAverage {
    pub fn new(alpha: TimeFloat) -> MovingAverage {
        MovingAverage {
            buffer: (0.0),
            alpha: (alpha),
        }
    }

    pub fn update(&mut self, t: &TimeFloat) {
        self.buffer = t * self.alpha + (1.0 - self.alpha) * self.buffer;
    }

    pub fn get_fps(&self) -> Option<TimeFloat> {
        match self.buffer {
            t if t < 0.0 || t > 0.0 => Some(1000.0 / t), //ms->s
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeStep {
    pub start_time: Instant,
    pub last_time: Instant,
    pub time_delta: TimeFloat,
    pub frame_count: CountInteger,
    pub runtime: TimeFloat,
    pub averager: MovingAverage,
}

impl TimeStep {
    pub fn new() -> TimeStep {
        TimeStep {
            start_time: Instant::now(),
            last_time: Instant::now(),
            time_delta: 0.0,
            frame_count: 0,
            runtime: 0.0,
            averager: MovingAverage::new(0.9),
        }
    }

    pub fn update(&mut self) {
        let current = Instant::now();
        self.time_delta = current.duration_since(self.last_time).as_millis() as TimeFloat;
        self.last_time = current;
        self.runtime += self.time_delta;
        self.frame_count += 1;
        self.averager.update(&self.time_delta);
    }

    pub fn reset(&mut self) {
        self.last_time = Instant::now();
        self.frame_count = 0;
        self.runtime = 0.0;
    }
}
