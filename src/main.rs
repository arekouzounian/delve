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

pub struct Point(u16, u16);

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
        .queue(cursor::DisableBlinking)?
        .queue(cursor::Hide)?
        .queue(terminal::DisableLineWrap)?
        .flush()
}

fn teardown() -> std::io::Result<()> {
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

// Bresenham's line algorithm
fn plot_line<B>(buf: &mut B, p1: Point, p2: Point, rune: char) -> std::io::Result<()>
where
    B: PlotQueuer,
{
    assert!(p1.0 < p2.0);
    assert!(p1.1 < p2.1);

    let dx = p2.0 as i16 - p1.0 as i16;
    let dy = p2.1 as i16 - p1.1 as i16;

    let mut error = 2 * dy - dx;
    let mut curr_y = p1.1;

    for x in p1.0..=p2.0 {
        buf.plot(x, curr_y, rune)?;

        if error > 0 {
            curr_y += 1;
            error += 2 * (dy - dx);
        } else {
            error += 2 * dy;
        }
    }

    Ok(())
}

fn render_frame(rng: &mut ThreadRng, rows: u16, cols: u16) -> std::io::Result<()> {
    let mut buffer = stdout();

    buffer.execute(terminal::Clear(terminal::ClearType::All))?;

    // wireframe
    for x in 0..=cols {
        buffer.plot(x, 0, '@')?;
        buffer.plot(x, rows, '@')?;
    }

    for y in 1..rows - 1 {
        buffer.plot(0, y, '$')?;
        buffer.plot(cols, y, '$')?;
    }

    // pick a random point, then draw a line some random offset away
    let offset = 5;
    let rand_col = rng.random_range(1..cols);
    let rand_row = rng.random_range(1..rows);

    let rand_offset_x = (rand_col + rng.random_range(1..=offset)).clamp(rand_col, cols);
    let rand_offset_y = (rand_row + rng.random_range(1..=offset)).clamp(rand_row, rows);

    plot_line(
        &mut buffer,
        Point(rand_col, rand_row),
        Point(rand_offset_x, rand_offset_y),
        '@',
    )?;

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

        let (cols, rows) = terminal::size()?;
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
