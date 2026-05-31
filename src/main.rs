use std::io::{Stdout, Write, stdout};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, KeyCode},
    style, terminal,
};
use rand::prelude::*;

use crate::space::{Camera, Scene, Shape, Sphere};
use crate::vector::Vec3;

mod space;
mod util;
mod vector;

const FRAMES_PER_SECOND: u64 = 30;
const MS_PER_TICK: Duration = Duration::from_millis(1000 / FRAMES_PER_SECOND);

// adjust depending on terminal? or is it standard?
const CELL_WIDTH_TO_HEIGHT_RATIO: f32 = 0.5;
const MAX_EVENTS_PER_FRAME: u8 = 16;

// closer to 1 -> less decay
const DECAY_SCALE: f32 = 0.9;

// larger than 1 -> increase force
// < 1 -> decrease force
const INPUT_SCALE: f32 = 0.01;

const VELOCITY_THRESHOLD: f32 = 0.0001;

const ROTATION_PER_FRAME_RADIANS: f32 = 0.05;

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
    pub fn default(row: u16, col: u16) -> Self {
        Self {
            brightness: 1.0,
            row,
            col,
            rune: b'@',
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
        .flush()
}

fn teardown() -> std::io::Result<()> {
    terminal::disable_raw_mode()?;
    stdout()
        .queue(terminal::Clear(terminal::ClearType::Purge))?
        .queue(cursor::MoveTo(0, 0))?
        .queue(cursor::EnableBlinking)?
        .queue(cursor::Show)?
        .flush()?;

    let (cols, rows) = terminal::size()?;
    println!("rows: {} cols: {}", rows, cols);

    Ok(())
}

pub enum Movement {
    Forward,
    Backward,
    Left,
    Right,
    PitchUp,
    PitchDown,
    YawLeft,
    YawRight,
}

/// swaps buffers after
fn render_frame(
    prv_buf: &mut FrameBuffer,
    buffer: &mut FrameBuffer,
    scene: &mut Scene,
    input_channel: &Receiver<Movement>,
    rng: &mut ThreadRng,
) -> std::io::Result<()> {
    buffer.clear();

    let width = buffer.cols() as f32;
    let height = buffer.rows() as f32;

    let aspect_ratio = (width / height) * CELL_WIDTH_TO_HEIGHT_RATIO;

    // do we need to recompute these?
    let forward_normal = scene.camera.forward_normal();
    let right_normal = scene.camera.right_normal();
    let up_normal = scene.camera.up_normal();

    // drain event queue
    let mut camera_force = Vec3::ORIGIN;
    let mut target_transform = Vec3::ORIGIN;
    for _ in 0..MAX_EVENTS_PER_FRAME {
        match input_channel.try_recv() {
            Ok(m) => match m {
                Movement::Forward => {
                    camera_force += forward_normal;
                }
                Movement::Backward => {
                    camera_force -= forward_normal;
                }
                Movement::Right => {
                    camera_force += right_normal;
                }
                Movement::Left => {
                    camera_force -= right_normal;
                }
                Movement::PitchUp => {
                    scene.camera.apply_pitch(ROTATION_PER_FRAME_RADIANS);
                }
                Movement::PitchDown => {
                    scene.camera.apply_pitch(-ROTATION_PER_FRAME_RADIANS);
                }
                Movement::YawLeft => {
                    scene.camera.apply_yaw(ROTATION_PER_FRAME_RADIANS);
                }
                Movement::YawRight => {
                    scene.camera.apply_yaw(-ROTATION_PER_FRAME_RADIANS);
                }
            },
            Err(_) => break,
        }
    }

    scene.camera.velocity = scene.camera.velocity.scalar_multiply(DECAY_SCALE)
        + camera_force.scalar_multiply(INPUT_SCALE);

    if scene.camera.velocity.dot(scene.camera.velocity) > VELOCITY_THRESHOLD {
        scene.camera.apply_force(scene.camera.velocity);
    }

    for row in 0..buffer.rows() {
        for col in 0..buffer.cols() {
            // map to normalized device coordinates
            // center is (0,0), top right is (1,1), bottom left is (-1,-1)
            let mut ndc_x = ((2.0 * col as f32) - width) / width;
            let mut ndc_y = (height - (2.0 * row as f32)) / height;

            // adjust for aspect ratio
            ndc_x *= aspect_ratio * scene.camera.fov_factor();
            ndc_y *= scene.camera.fov_factor();

            let normalized_ray_direction = (forward_normal.clone()
                + right_normal.scalar_multiply(ndc_x)
                + up_normal.scalar_multiply(ndc_y))
            .normalize();

            if let Some(_hit) =
                scene.intersect(scene.camera.get_position(), normalized_ray_direction)
            {
                // TODO: character selection logic goes here
                buffer.set_cell(Cell::default(row, col));
            }
        }
    }

    buffer.draw_to_stdout(prv_buf)?;
    std::mem::swap(buffer, prv_buf);
    Ok(())
}

/*
* Listen for input. if we encounter desirable input, send over the channel
* to the render loop.
*
* render loop slowly grows the magnitude of a vector in each direction, or
* drains it with no input.
*/
fn listen_for_input(
    running_flag: Arc<AtomicBool>,
    input_channel: Sender<Movement>,
) -> std::io::Result<()> {
    let last_result = loop {
        let new_event = event::read();

        if let Err(e) = new_event {
            break Err(e);
        }

        let mut next_movement = None;

        match new_event.unwrap() {
            event::Event::Key(key_event) => match key_event.code {
                KeyCode::Esc => break Ok(()),
                KeyCode::Up => next_movement = Some(Movement::PitchUp),
                KeyCode::Down => next_movement = Some(Movement::PitchDown),
                KeyCode::Left => next_movement = Some(Movement::YawLeft),
                KeyCode::Right => next_movement = Some(Movement::YawRight),
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
            },
            _ => (),
        }

        if let Some(m) = next_movement
            && let Err(e) = input_channel.send(m)
        {
            break Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e));
        }
    };

    running_flag.store(false, Ordering::SeqCst);

    last_result
}

fn main() -> std::io::Result<()> {
    setup()?;

    let mut rng = rand::rng();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let (send, recv) = mpsc::channel();

    let input_thread = thread::spawn(move || listen_for_input(running_clone, send));

    let profiling_enabled = cfg!(profiling_enabled);
    let mut samples: u64 = 0;
    let mut elapsed_sums: u128 = 0;

    // TODO: how to deal with resizing?
    // guess we would have to reallocate the framebuf on-demand; maybe
    // detect when resizing then only reallocate then
    let (cols, rows) = terminal::size()?;
    let mut curr_framebuf = FrameBuffer::new(rows, cols);
    let mut prev_framebuf = FrameBuffer::new(rows, cols);

    let camera = Camera::new(Vec3::new(0.0, 0.0, -5.0), Vec3::ORIGIN, Camera::PI / 2.0);
    let mut scene = Scene::new(camera);

    // sphere at origin, radius 1
    let id = String::from("asdf");
    let sphere = Sphere::new(Vec3::ORIGIN, 1.0);
    scene.register_shape(id.clone(), Shape::Sphere(sphere));

    // after 1 seconds, pick another random vec
    // let mut random_direction = Vec3::from_random(&mut rng).scalar_multiply(0.20);
    // let mut frame_counter = 0;
    // let seconds = 1;
    // let frame_max = FRAMES_PER_SECOND * seconds; // 10 seconds

    while running.load(Ordering::SeqCst) {
        let start_time = Instant::now();

        render_frame(
            &mut prev_framebuf,
            &mut curr_framebuf,
            &mut scene,
            &recv,
            &mut rng,
        )
        .expect("fatal: unable to render frame");

        // match scene
        //     .get_shape_mut(&id)
        //     .expect("what happened to my sphere")
        // {
        //     Shape::Sphere(s) => {
        //         s.apply_force(random_direction);
        //     }
        // }

        let end_time = Instant::now();
        let elapsed = end_time.duration_since(start_time);

        if profiling_enabled {
            elapsed_sums += elapsed.as_micros();
            samples += 1;
        }

        // frame_counter += 1;
        // frame_counter %= frame_max;

        // if frame_counter == 0 {
        //     random_direction = Vec3::from_random(&mut rng).scalar_multiply(0.20);
        // }

        if elapsed < MS_PER_TICK {
            sleep(MS_PER_TICK - elapsed);
        }
    }

    let _ = input_thread.join().expect("something went wrong");

    teardown()?;

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
