use std::time::Duration;

pub const FRAMES_PER_SECOND: u64 = 30;
pub const MS_PER_TICK: Duration = Duration::from_millis(1000 / FRAMES_PER_SECOND);

// adjust depending on terminal? or is it standard?
pub const CELL_WIDTH_TO_HEIGHT_RATIO: f32 = 0.5;

// closer to 1 -> less decay
pub const DECAY_SCALE: f32 = 0.9;

// larger than 1 -> increase force
// < 1 -> decrease force
pub const INPUT_SCALE: f32 = 0.01;

pub const VELOCITY_DAMP: f32 = 0.9;

pub const VELOCITY_THRESHOLD: f32 = 0.0001;

pub const ROTATION_PER_FRAME_RADIANS: f32 = 0.05;

pub const AMBIENT_LIGHTING: f32 = 0.10;
