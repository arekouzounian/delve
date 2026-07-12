use std::io::{Write, stdout};
use std::sync::atomic::AtomicU32;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::thread::sleep;
use std::time::Instant;

use crossterm::{QueueableCommand, cursor, event, terminal};
use delve_shared::traits::Entity;

use crate::{
    input::construct_camera_forces,
    render::{Cell, FrameBuffer},
    scene::Scene,
};
use delve_shared::constants::*;

pub struct DelveEngine {
    // TODO: how to deal with resizing?
    // guess we would have to reallocate the framebuf on-demand; maybe
    // detect when resizing then only reallocate then
    curr_framebuf: FrameBuffer,
    prev_framebuf: FrameBuffer,
    scene: Arc<RwLock<Scene>>,
    movement_flags: Arc<AtomicU32>,
    is_running: Arc<AtomicBool>,
}

impl DelveEngine {
    pub fn new(
        scene: Scene,
        movement_flags: Arc<AtomicU32>,
        is_running: Arc<AtomicBool>,
    ) -> std::io::Result<Self> {
        let (cols, rows) = terminal::size()?;
        let partitions_vec = Self::distribute_rows_over_threads(rows);

        Ok(Self {
            curr_framebuf: FrameBuffer::new(partitions_vec.clone(), rows, cols),
            prev_framebuf: FrameBuffer::new(partitions_vec, rows, cols),
            scene: Arc::new(RwLock::new(scene)),
            movement_flags,
            is_running,
        })
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

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
    }

    fn get_thread_count() -> u16 {
        std::thread::available_parallelism()
            .unwrap()
            .get()
            .try_into()
            .expect("we have more than 65k threads... what")
    }

    /// returns an array of length (# of threads).
    /// arr[i] = the number of rows that thread should render.
    /// tries to evenly distribute rows among threads, but overflow
    /// goes to the first threads.
    fn distribute_rows_over_threads(mut rows: u16) -> Vec<u16> {
        // if there are fewer rows than threads, don't spin up empty partitions.
        let thread_count = (Self::get_thread_count() as usize)
            .min(rows as usize)
            .max(1);
        let mut rows_vec = vec![0u16; thread_count];

        let mut ind = 0;
        while rows > 0 {
            rows_vec[ind] += 1;
            rows -= 1;
            ind = (ind + 1) % thread_count;
        }

        rows_vec
    }

    /// blocks until is_running is set to false.
    /// panics if called more than once.
    pub fn run(mut self) -> std::io::Result<()> {
        assert!(!self.is_running.load(Ordering::SeqCst));
        self.is_running.store(true, Ordering::SeqCst);

        #[cfg(profiling_enabled)]
        let mut samples: u64 = 0;
        #[cfg(profiling_enabled)]
        let mut elapsed_sums: u128 = 0;

        DelveEngine::setup()?;
        #[allow(unused)]
        let mut rng = rand::rng();

        let total_rows = self.curr_framebuf.rows();
        let cols_per_row = self.curr_framebuf.cols();
        let partition_row_counts = Self::distribute_rows_over_threads(total_rows);
        let thread_count = partition_row_counts.len();
        assert_eq!(thread_count, self.curr_framebuf.partitions());

        let mut handles = Vec::with_capacity(thread_count);
        let mut thread_senders = Vec::with_capacity(thread_count);

        // threads pass around a vector, which is a partition of the current framebuf.
        // each partition holds a number of contiguous rows.
        let (main_send, main_recv) = std::sync::mpsc::channel::<(usize, Vec<Vec<Option<Cell>>>)>();

        let mut partition_start_row: u16 = 0;
        for &row_count in &partition_row_counts {
            let (thread_send, thread_recv) =
                std::sync::mpsc::channel::<(usize, Vec<Vec<Option<Cell>>>)>();

            let scene = self.scene.clone();
            let main_send = main_send.clone();
            let is_running = self.is_running.clone();

            // worker i should always process partition i, otherwise this logic breaks
            // but we shouldn't ever be reordering the partitions so this should suffice.
            let start_row = partition_start_row;

            handles.push(std::thread::spawn(move || {
                let width = cols_per_row as f32;
                let height = total_rows as f32;
                let aspect_ratio = (width / height) * CELL_WIDTH_TO_HEIGHT_RATIO;

                // on each frame the main thread sends us our partition — the block of rows
                // starting at `start_row`.
                for (partition_index, mut partition) in thread_recv.iter() {
                    // scene read lock acquired, ensures no more modifications and can be shared
                    // among multiple concurrent readers
                    let scene = scene.read().unwrap();

                    let normals = scene.get_normals();
                    let fov_factor = scene.get_camera().fov_factor();
                    let camera_position = scene.get_camera().entity_fields().position;

                    for (row_idx, row_buf) in partition.iter_mut().enumerate() {
                        let row = start_row + row_idx as u16;
                        let mut ndc_y = (height - (2.0 * row as f32)) / height;
                        ndc_y *= fov_factor;

                        for col in 0..cols_per_row {
                            let mut ndc_x = ((2.0 * col as f32) - width) / width;
                            ndc_x *= aspect_ratio * fov_factor;

                            let normalized_ray_direction = (normals.forward
                                + normals.right.scalar_multiply(ndc_x)
                                + normals.up.scalar_multiply(ndc_y))
                            .normalize();

                            if let Some(hit) =
                                scene.intersect(camera_position, normalized_ray_direction)
                            {
                                let brightness = scene.lambertian_brightness(&hit);
                                row_buf[col as usize] = Some(Cell::default_scale(brightness));
                            } else {
                                row_buf[col as usize] = None;
                            }
                        }
                    }

                    if let Err(_e) = main_send.send((partition_index, partition)) {
                        is_running.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }));

            thread_senders.push(thread_send);
            partition_start_row += row_count;
        }

        // drop the original, workers have copies
        drop(main_send);

        'outer: while self.is_running.load(Ordering::SeqCst) {
            let start_time = Instant::now();

            {
                // perform any physics/rotations updates here, with write access to the scene
                let mut scene = match self.scene.write() {
                    Ok(guard) => guard,
                    Err(_e) => {
                        self.is_running.store(false, Ordering::SeqCst);
                        break;
                    }
                };

                let camera_force = construct_camera_forces(
                    scene.get_normals(),
                    self.movement_flags.load(Ordering::Relaxed),
                );
                scene.apply_camera_force(camera_force);

                scene.update_all();
            }

            // from the current framebuf, assign a partition to each worker.
            for (i, worker) in thread_senders.iter_mut().enumerate() {
                if let Err(_e) = worker.send((i, self.curr_framebuf.take_partition(i))) {
                    self.is_running.store(false, Ordering::SeqCst);
                    break 'outer;
                }
            }

            for _ in 0..thread_count {
                if let Ok((partition_index, partition)) = main_recv.recv_timeout(MS_PER_TICK) {
                    self.curr_framebuf.put_partition(partition_index, partition);
                } else {
                    self.is_running.store(false, Ordering::SeqCst);
                    break 'outer;
                }
            }

            if let Err(_e) = self.curr_framebuf.draw_to_stdout(&self.prev_framebuf) {
                self.is_running.store(false, Ordering::SeqCst);
                break;
            }
            std::mem::swap(&mut self.curr_framebuf, &mut self.prev_framebuf);

            let elapsed = Instant::now().duration_since(start_time);

            #[cfg(profiling_enabled)]
            {
                elapsed_sums += elapsed.as_millis();
                samples += 1;
            }

            if elapsed < MS_PER_TICK {
                sleep(MS_PER_TICK - elapsed);
            }
        }

        drop(thread_senders);
        for thread in handles {
            let _ = thread.join();
        }

        DelveEngine::teardown()?;

        #[cfg(profiling_enabled)]
        {
            let scene = self.scene.read().unwrap();
            println!(
                "last camera position: {:?}",
                scene.get_camera().entity_fields().position
            );

            println!(
                "\nelapsed_sums: {}\nsamples: {}\navg frame render time: {} millis",
                elapsed_sums,
                samples,
                (elapsed_sums as f64) / (samples as f64)
            );
        }

        Ok(())
    }
}
