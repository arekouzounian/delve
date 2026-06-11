use rand::prelude::*;
use std::io::{Stdout, Write, stdout};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crossterm::{ExecutableCommand, QueueableCommand, cursor, style, terminal};

use crate::constants::CELL_WIDTH_TO_HEIGHT_RATIO;
use crate::input::apply_camera_forces;
use crate::scene::Scene;

pub trait BufType: Write + ExecutableCommand + QueueableCommand {}
impl<T: Write + ExecutableCommand + QueueableCommand> BufType for T {}

#[derive(Clone, PartialEq)]
pub struct Cell {
    #[allow(unused)]
    brightness: f32,
    rune: char,
}

impl Cell {
    pub const DEFAULT_BRIGHTNESS_SCALE: [char; 8] = ['.', ':', '-', '+', '*', '#', '%', '@'];
    // pub const BRIGHTNESS_SCALE: [char; 4] = ['░', '▒', '▓', '█'];

    pub fn select_from_brightness_scale(brightness: f32, scale: &[char]) -> char {
        let bucket = (brightness.clamp(0.0, 1.0) * ((scale.len() - 1) as f32)) as usize;

        scale[bucket]
    }

    pub fn default_scale(brightness: f32) -> Self {
        Cell::with_scale(brightness, &Cell::DEFAULT_BRIGHTNESS_SCALE)
    }

    pub fn with_scale(brightness: f32, scale: &[char]) -> Self {
        Self {
            brightness,
            rune: Cell::select_from_brightness_scale(brightness, scale),
        }
    }
}

// assuming stdout for now
pub struct FrameBuffer {
    inner_buf: Vec<Vec<Option<Cell>>>,
    stdout_handle: Stdout,
}

impl FrameBuffer {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            inner_buf: vec![vec![None; cols as usize]; rows as usize],
            stdout_handle: stdout(),
        }
    }

    // clears stdout and sets all cells to None
    pub fn clear_all(&mut self) -> std::io::Result<()> {
        self.stdout_handle
            .execute(terminal::Clear(terminal::ClearType::All))?;

        self.clear();

        Ok(())
    }

    pub fn clear(&mut self) {
        for row in &mut self.inner_buf {
            for cell in row {
                cell.take();
            }
        }
    }

    pub fn rows(&self) -> u16 {
        self.inner_buf.len() as u16
    }

    pub fn cols(&self) -> u16 {
        self.inner_buf[0].len() as u16
    }

    pub fn draw_to_stdout(&mut self, prv_buf: &mut FrameBuffer) -> std::io::Result<()> {
        let rows = self.inner_buf.len();
        let cols = self.inner_buf[0].len();

        assert!(rows == prv_buf.inner_buf.len());
        assert!(cols == prv_buf.inner_buf[0].len());

        let mut curr_row = 0;
        while curr_row < rows {
            let mut curr_col = 0;

            while curr_col < cols {
                let mut curr_cell = &self.inner_buf[curr_row][curr_col];
                let mut prev_cell = &prv_buf.inner_buf[curr_row][curr_col];

                if curr_cell.ne(prev_cell) {
                    self.stdout_handle
                        .queue(cursor::MoveTo(curr_col as u16, curr_row as u16))?;

                    while curr_col < cols - 1 && curr_cell.ne(prev_cell) {
                        match curr_cell {
                            Some(c) => self.stdout_handle.queue(style::Print(c.rune))?,
                            None => self.stdout_handle.queue(style::Print(' '))?,
                        };

                        curr_col += 1;

                        curr_cell = &self.inner_buf[curr_row][curr_col];
                        prev_cell = &prv_buf.inner_buf[curr_row][curr_col];
                    }
                }

                curr_col += 1;
            }

            curr_row += 1;
        }

        self.stdout_handle.flush()
    }
}

pub fn render_frame_swap_buffers(
    prv_buf: &mut FrameBuffer,
    buffer: &mut FrameBuffer,
    scene: &mut Scene,
    movement_flags: &Arc<AtomicU32>,
    _rng: &mut ThreadRng,
) -> std::io::Result<()> {
    buffer.clear();

    let width = buffer.cols() as f32;
    let height = buffer.rows() as f32;

    // let scale: Vec<char> = ".`-_':,;^~+=<>ilI!?1rctjuoezasxvnypwkbdfhqmgJCLUOZQG0DYXKVPAWSB#RHENM$&@".chars().collect();

    let aspect_ratio = (width / height) * CELL_WIDTH_TO_HEIGHT_RATIO;

    apply_camera_forces(
        scene.get_camera_mut(),
        movement_flags.load(Ordering::Relaxed),
    );

    // all movement should be done for camera at this point so we can cache location
    let camera_position = scene.get_camera().get_position();
    let (forward_normal, right_normal, up_normal) = scene.get_normals();

    for row in 0..buffer.rows() {
        for col in 0..buffer.cols() {
            // map to normalized device coordinates
            // center is (0,0), top right is (1,1), bottom left is (-1,-1)
            let mut ndc_x = ((2.0 * col as f32) - width) / width;
            let mut ndc_y = (height - (2.0 * row as f32)) / height;

            let fov_factor = scene.get_camera().fov_factor();
            // adjust for aspect ratio
            ndc_x *= aspect_ratio * fov_factor;
            ndc_y *= fov_factor;

            let normalized_ray_direction = (forward_normal
                + right_normal.scalar_multiply(ndc_x)
                + up_normal.scalar_multiply(ndc_y))
            .normalize();

            if let Some(hit) = scene.intersect(camera_position, normalized_ray_direction) {
                let brightness = scene.lambertian_brightness(&hit);
                buffer.inner_buf[row as usize][col as usize] =
                    Some(Cell::default_scale(brightness));
            }
        }
    }

    buffer.draw_to_stdout(prv_buf)?;
    std::mem::swap(buffer, prv_buf);
    Ok(())
}
