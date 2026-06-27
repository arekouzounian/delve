use crossterm::event::{self, KeyCode, ModifierKeyCode};
use std::sync::{
    Arc,
    atomic::Ordering,
    atomic::{AtomicBool, AtomicU32},
};

use crate::scene::CameraNormals;
use delve_shared::{constants::*, math::Vec3};

#[repr(u32)]
pub enum Movement {
    Forward = 1 << 0,
    Backward = 1 << 1,
    Left = 1 << 2,
    Right = 1 << 3,
    Up = 1 << 4,
    Down = 1 << 5,
    PitchUp = 1 << 6,
    PitchDown = 1 << 7,
    YawLeft = 1 << 8,
    YawRight = 1 << 9,
}

pub struct CameraForce {
    pub forces: Vec3,
    pub pitch_radians: f32,
    pub yaw_radians: f32,
}

/*
* Listen for input. if we encounter desirable input, send over the channel
* to the render loop.
*
* render loop slowly grows the magnitude of a vector in each direction, or
* drains it with no input.
*/
pub fn listen_for_input(
    running_flag: Arc<AtomicBool>,
    movement_flags: Arc<AtomicU32>,
) -> std::io::Result<()> {
    let last_result = loop {
        let new_event = event::read();

        if let Err(e) = new_event {
            break Err(e);
        } else if !running_flag.load(Ordering::SeqCst) {
            break Ok(()); // set externally, exit now
        }

        let mut next_movement = None;
        let new_event = new_event.unwrap();
        let is_key_down = new_event.is_key_press() || new_event.is_key_repeat();

        if let event::Event::Key(key_event) = new_event {
            match key_event.code {
                KeyCode::Esc => break Ok(()),
                KeyCode::Up => next_movement = Some(Movement::PitchUp),
                KeyCode::Down => next_movement = Some(Movement::PitchDown),
                KeyCode::Left => next_movement = Some(Movement::YawLeft),
                KeyCode::Right => next_movement = Some(Movement::YawRight),
                KeyCode::Modifier(modifier) => match modifier {
                    ModifierKeyCode::LeftShift => next_movement = Some(Movement::Up),
                    ModifierKeyCode::LeftControl => next_movement = Some(Movement::Down),
                    _ => (),
                },
                KeyCode::Char(c) => {
                    if c.to_lowercase().eq('q'.to_lowercase()) {
                        break Ok(());
                    } else if c.to_lowercase().eq('w'.to_lowercase()) {
                        next_movement = Some(Movement::Forward);
                    } else if c.to_lowercase().eq('a'.to_lowercase()) {
                        next_movement = Some(Movement::Left);
                    } else if c.to_lowercase().eq('s'.to_lowercase()) {
                        next_movement = Some(Movement::Backward);
                    } else if c.to_lowercase().eq('d'.to_lowercase()) {
                        next_movement = Some(Movement::Right);
                    }
                }

                _ => (),
            };
        }

        if let Some(m) = next_movement {
            match is_key_down {
                true => movement_flags.fetch_or(m as u32, Ordering::Relaxed),
                false => movement_flags.fetch_and(!(m as u32), Ordering::Relaxed),
            };
        }
    };

    running_flag.store(false, Ordering::SeqCst);

    last_result
}

pub fn construct_camera_forces(normals: CameraNormals, movement_flags: u32) -> CameraForce {
    let mut camera_force = Vec3::ORIGIN;
    let mut pitch = 0.0;
    let mut yaw = 0.0;

    // get rid of the y component to lock us into the xz plane
    let flat_forward = Vec3::new(normals.forward.x, 0.0, normals.forward.z).normalize();
    let flat_right = Vec3::new(normals.right.x, 0.0, normals.right.z).normalize();

    if (movement_flags & (Movement::Forward as u32)) == Movement::Forward as u32 {
        camera_force += flat_forward;
    }
    if (movement_flags & (Movement::Backward as u32)) == Movement::Backward as u32 {
        camera_force -= flat_forward;
    }
    if (movement_flags & (Movement::Right as u32)) == Movement::Right as u32 {
        camera_force += flat_right;
    }
    if (movement_flags & (Movement::Left as u32)) == Movement::Left as u32 {
        camera_force -= flat_right
    }
    if (movement_flags & (Movement::Up as u32)) == Movement::Up as u32 {
        camera_force += Vec3::Y;
    }
    if (movement_flags & (Movement::Down as u32)) == Movement::Down as u32 {
        camera_force -= Vec3::Y;
    }
    if (movement_flags & (Movement::PitchUp as u32)) == Movement::PitchUp as u32 {
        pitch -= ROTATION_PER_FRAME_RADIANS;
    }
    if (movement_flags & (Movement::PitchDown as u32)) == Movement::PitchDown as u32 {
        pitch += ROTATION_PER_FRAME_RADIANS
    }
    if (movement_flags & (Movement::YawRight as u32)) == Movement::YawRight as u32 {
        yaw -= ROTATION_PER_FRAME_RADIANS;
    }
    if (movement_flags & (Movement::YawLeft as u32)) == Movement::YawLeft as u32 {
        yaw += ROTATION_PER_FRAME_RADIANS;
    }

    CameraForce {
        forces: camera_force,
        pitch_radians: pitch,
        yaw_radians: yaw,
    }
}
