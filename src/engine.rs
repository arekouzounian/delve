use std::io::{Write, stdout};
use std::sync::atomic::AtomicU32;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::sleep;
use std::time::Instant;

use crossterm::{QueueableCommand, cursor, event, terminal};

use crate::{
    constants::*,
    render::{FrameBuffer, render_frame_swap_buffers},
    scene::Scene,
};

pub struct DelveEngine {
    // TODO: how to deal with resizing?
    // guess we would have to reallocate the framebuf on-demand; maybe
    // detect when resizing then only reallocate then
    curr_framebuf: FrameBuffer,
    prev_framebuf: FrameBuffer,
    scene: Scene,
    movement_flags: Arc<AtomicU32>,
    is_running: Arc<AtomicBool>,
}

impl DelveEngine {
    pub fn new(
        scene: Scene,
        movement_flags: Arc<AtomicU32>,
        is_running: Arc<AtomicBool>,
    ) -> std::io::Result<Self> {
        let (framebuf_cols, framebuf_rows) = terminal::size()?;

        Ok(Self {
            curr_framebuf: FrameBuffer::new(framebuf_rows, framebuf_cols),
            prev_framebuf: FrameBuffer::new(framebuf_rows, framebuf_cols),
            scene,
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
        let mut rng = rand::rng();

        while self.is_running.load(Ordering::SeqCst) {
            let start_time = Instant::now();

            self.scene.update_all();

            if let Err(_e) = render_frame_swap_buffers(
                &mut self.prev_framebuf,
                &mut self.curr_framebuf,
                &mut self.scene,
                &self.movement_flags,
                &mut rng,
            ) {
                self.is_running.store(false, Ordering::SeqCst);
                break;
            }

            let elapsed = Instant::now().duration_since(start_time);

            #[cfg(profiling_enabled)]
            {
                elapsed_sums += elapsed.as_micros();
                samples += 1;
            }

            if elapsed < MS_PER_TICK {
                sleep(MS_PER_TICK - elapsed);
            }
        }

        DelveEngine::teardown()?;

        #[cfg(profiling_enabled)]
        {
            println!(
                "last camera position: {:?}",
                self.scene.get_camera().get_position()
            );

            println!(
                "\nelapsed_sums: {}\nsamples: {}\navg frame render time: {} micros",
                elapsed_sums,
                samples,
                (elapsed_sums as f64) / (samples as f64)
            );
        }

        Ok(())
    }
}
