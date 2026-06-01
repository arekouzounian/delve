use std::io::{Stdout, Write, stdout};
use std::sync::atomic::AtomicU32;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, sleep};
use std::time::Instant;

use crossterm::{ExecutableCommand, QueueableCommand, cursor, event, style, terminal};
use rand::prelude::*;

use crate::constants::*;
use crate::input::{apply_camera_forces, listen_for_input};
use crate::space::{Camera, Cube, Light, Scene, Shape, Sphere};
use crate::vector::Vec3;

mod constants;
mod input;
mod space;
mod util;
mod vector;

pub trait BufType: Write + ExecutableCommand + QueueableCommand {}
impl<T: Write + ExecutableCommand + QueueableCommand> BufType for T {}

#[derive(Clone, PartialEq)]
pub struct Cell {
    #[allow(unused)]
    brightness: f32,
    row: u16,
    col: u16,
    rune: u8,
}

impl Cell {
    pub const BRIGHTNESS_SCALE: [u8; 8] = [b'.', b':', b'-', b'+', b'*', b'#', b'%', b'@'];

    pub fn select_from_brightness_scale(brightness: f32, scale: &[u8]) -> u8 {
        let bucket = (brightness.clamp(0.0, 1.0) * ((scale.len() - 1) as f32)) as usize;
        return scale[bucket];
    }

    pub fn default(row: u16, col: u16) -> Self {
        Self {
            brightness: 1.0,
            row,
            col,
            rune: b'@',
        }
    }

    pub fn with_brightness(row: u16, col: u16, brightness: f32) -> Self {
        let rune = Cell::select_from_brightness_scale(brightness, &Cell::BRIGHTNESS_SCALE);
        Self {
            brightness,
            row,
            col,
            rune,
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

    pub fn set_cell(&mut self, c: Cell) {
        let row = c.row as usize;
        let col = c.col as usize;
        assert!(row < self.inner_buf.len());
        assert!(col < self.inner_buf[0].len());

        self.inner_buf[row][col] = Some(c);
    }

    pub fn get_cell(&self, row: usize, col: usize) -> &Option<Cell> {
        assert!(row < self.inner_buf.len());
        assert!(col < self.inner_buf[0].len());

        &self.inner_buf[row][col]
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
        self.stdout_handle.queue(cursor::MoveTo(0, 0))?;

        for (curr_row, row_vec) in self.inner_buf.iter().enumerate() {
            for (curr_col, cell) in row_vec.iter().enumerate() {
                let prv_cell = prv_buf.get_cell(curr_row, curr_col);

                if cell.ne(prv_cell) {
                    self.stdout_handle
                        .queue(cursor::MoveTo(curr_col as u16, curr_row as u16))?;

                    match cell {
                        Some(c) => {
                            self.stdout_handle.queue(style::Print(c.rune as char))?;
                        }
                        None => {
                            self.stdout_handle.queue(style::Print(' '))?;
                        }
                    };
                }
            }
        }

        self.stdout_handle.flush()
    }
}

fn setup() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    stdout()
        .queue(cursor::DisableBlinking)?
        .queue(cursor::Hide)?
        .queue(terminal::DisableLineWrap)?
        .queue(terminal::Clear(terminal::ClearType::Purge))?
        .queue(event::PushKeyboardEnhancementFlags(
            event::KeyboardEnhancementFlags::all(),
        ))?
        .flush()
}

fn teardown() -> std::io::Result<()> {
    terminal::disable_raw_mode()?;
    stdout()
        .queue(terminal::Clear(terminal::ClearType::Purge))?
        .queue(cursor::MoveTo(0, 0))?
        .queue(cursor::EnableBlinking)?
        .queue(cursor::Show)?
        .queue(event::PopKeyboardEnhancementFlags)?
        .flush()?;

    if cfg!(profiling_enabled) {
        let (cols, rows) = terminal::size()?;
        println!("rows: {} cols: {}", rows, cols);
    }

    Ok(())
}

/// swaps buffers after
fn render_frame(
    prv_buf: &mut FrameBuffer,
    buffer: &mut FrameBuffer,
    scene: &mut Scene,
    movement_flags: &Arc<AtomicU32>,
    _rng: &mut ThreadRng,
) -> std::io::Result<()> {
    buffer.clear();

    let width = buffer.cols() as f32;
    let height = buffer.rows() as f32;

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
                let brightness = scene.lambertian_brightness(hit.get_surface_normal());
                buffer.set_cell(Cell::with_brightness(row, col, brightness));
            }
        }
    }

    buffer.draw_to_stdout(prv_buf)?;
    std::mem::swap(buffer, prv_buf);
    Ok(())
}

fn main() -> std::io::Result<()> {
    setup()?;

    let mut rng = rand::rng();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let movement_flags = Arc::new(AtomicU32::new(0));
    let mov_flags_clone = movement_flags.clone();

    let input_thread = thread::spawn(move || listen_for_input(running_clone, mov_flags_clone));

    let profiling_enabled = cfg!(profiling_enabled);
    let mut samples: u64 = 0;
    let mut elapsed_sums: u128 = 0;

    // TODO: how to deal with resizing?
    // guess we would have to reallocate the framebuf on-demand; maybe
    // detect when resizing then only reallocate then
    let (cols, rows) = terminal::size()?;
    let mut curr_framebuf = FrameBuffer::new(rows, cols);
    let mut prev_framebuf = FrameBuffer::new(rows, cols);

    let camera = Camera::new(
        Vec3::new(4.384, 0.0, -1.402),
        Vec3::ORIGIN,
        Camera::PI / 2.0,
    );
    let mut scene = Scene::new(camera, AMBIENT_LIGHTING);

    scene.register_light(Light::new(Vec3::new(1.0, 2.0, -1.0), 0.5));
    scene.register_light(Light::new(Vec3::new(1.0, 0.0, 0.0), 0.6));

    // spheres starting at origin, increasing in radius by 0.1
    let sphere_count = 5;
    let distance_inc = 2.0;
    let radius_inc = 0.2;
    let start_pt = -sphere_count as f32;
    for i in 0..sphere_count {
        let id = format!("{}", i);
        let sphere = Sphere::new(
            Vec3::new(start_pt + (distance_inc * i as f32), 0.0, 0.0),
            0.1 + (radius_inc * i as f32),
        );
        scene.register_shape(id, Shape::Sphere(sphere));
    }

    let cube = Cube::new(1.0, Vec3::new(5.0, 0.0, 5.0));

    scene.register_shape(String::from("cube"), Shape::Cube(cube));

    while running.load(Ordering::SeqCst) {
        let start_time = Instant::now();

        if let Err(_e) = render_frame(
            &mut prev_framebuf,
            &mut curr_framebuf,
            &mut scene,
            &movement_flags,
            &mut rng,
        ) {
            running.store(false, Ordering::SeqCst);
            break;
        }

        let end_time = Instant::now();
        let elapsed = end_time.duration_since(start_time);

        if profiling_enabled {
            elapsed_sums += elapsed.as_micros();
            samples += 1;
        }

        if elapsed < MS_PER_TICK {
            sleep(MS_PER_TICK - elapsed);
        }
    }

    let _ = input_thread.join().expect("something went wrong");

    teardown()?;

    if cfg!(profiling_enabled) {
        println!(
            "last camera position: {:?}",
            scene.get_camera().get_position()
        )
    }

    if profiling_enabled {
        println!(
            "\nelapsed_sums: {}\nsamples: {}\navg frame render time: {} micros",
            elapsed_sums,
            samples,
            (elapsed_sums as f64) / (samples as f64)
        );
    }

    Ok(())
}
