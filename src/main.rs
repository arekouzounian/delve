use std::io::{Write, stdout};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::{ExecutableCommand, QueueableCommand, cursor, style, terminal};
use rand::prelude::*;

const FRAMES_PER_SECOND: u64 = 1;
const MS_PER_TICK: Duration = Duration::from_millis(1000 / FRAMES_PER_SECOND);

pub trait PlotQueuer: Write + QueueableCommand {
    fn plot(&mut self, col: u16, row: u16, character: char) -> std::io::Result<()>;
}

impl<T: Write + QueueableCommand> PlotQueuer for T {
    fn plot(&mut self, col: u16, row: u16, character: char) -> std::io::Result<()> {
        self.queue(cursor::MoveTo(col, row))?
            .queue(style::Print(character))?;

        Ok(())
    }
}

fn setup() -> std::io::Result<()> {
    stdout()
        .execute(cursor::DisableBlinking)?
        .execute(cursor::Hide)?
        .execute(terminal::DisableLineWrap)?;

    Ok(())
}

fn render_frame(rng: &mut ThreadRng, rows: u16, cols: u16) -> std::io::Result<()> {
    let mut buffer = stdout();

    buffer.execute(terminal::Clear(terminal::ClearType::All))?;

    // pick a random point, then draw a small square
    let rand_col = rng.random_range(0..rows);
    let rand_row = rng.random_range(0..cols);

    for row in 0..3 {
        for col in 0..4 {
            buffer.plot(rand_col + col, rand_row + row, '@')?;
        }
    }

    buffer.flush()
}

fn teardown() -> std::io::Result<()> {
    let mut buffer = stdout();

    buffer
        .queue(terminal::Clear(terminal::ClearType::Purge))?
        .queue(cursor::MoveTo(0, 0))?
        .queue(cursor::EnableBlinking)?;

    buffer.flush()
}

fn main() -> std::io::Result<()> {
    setup()?;

    let mut rng = rand::rng();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("unable to set ctrl-c handler!");

    let profiling_enabled = cfg!(profiling_enabled);
    let mut samples: u64 = 0;
    let mut elapsed_sums: u128 = 0;

    while running.load(Ordering::SeqCst) {
        let start_time = Instant::now();

        let (rows, cols) = terminal::size()?;
        render_frame(&mut rng, rows, cols)?;

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
