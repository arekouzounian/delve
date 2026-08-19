use std::io::{Stdout, Write, stdout};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    style::{self, Stylize},
    terminal,
};

pub trait BufType: Write + ExecutableCommand + QueueableCommand {}
impl<T: Write + ExecutableCommand + QueueableCommand> BufType for T {}

#[derive(Clone, PartialEq)]
pub struct Cell {
    brightness: f32,
    rune: char,
}

impl Cell {
    pub const DEFAULT_BRIGHTNESS_SCALE: [char; 8] = ['.', ':', '-', '+', '*', '#', '%', '@'];
    // pub const DEFAULT_BRIGHTNESS_SCALE =
    //   ".`-_':,;^~+=<>ilI!?1rctjuoezasxvnypwkbdfhqmgJCLUOZQG0DYXKVPAWSB#RHENM$&@";
    // pub const DEFAULT_BRIGHTNESS_SCALE: [char; 4] = ['░', '▒', '▓', '█'];

    pub fn select_from_brightness_scale(brightness: f32, scale: &[char]) -> char {
        let brightness = brightness.clamp(0.0, 1.0);
        let bucket = (brightness * ((scale.len() - 1) as f32)) as usize;

        scale[bucket]
    }

    pub fn with_rune(rune: char, brightness: f32) -> Self {
        Self { brightness, rune }
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

    pub fn draw<'a, 'b>(&'a self, stdout_handle: &'b mut Stdout) -> std::io::Result<&'b mut Stdout>
    where
        'a: 'b,
    {
        if self.brightness < 0.33 {
            stdout_handle.queue(style::Print(self.rune.dim()))
        } else if self.brightness < 0.66 {
            stdout_handle.queue(style::Print(self.rune))
        } else {
            stdout_handle.queue(style::Print(self.rune.bold()))
        }
    }
}

// assuming stdout for now
pub struct FrameBuffer {
    // outer vec holds partitions, each vec is a single partition
    inner_buf: Vec<Vec<Vec<Option<Cell>>>>,
    stdout_handle: Stdout,
    rows: u16,
    cols: u16,
}

impl FrameBuffer {
    /// partitions_vec is a flat array that tells us how many rows each partition should handle
    /// each partition is a collection of rows that gets handed off to its own thread
    pub fn new(partitions_vec: Vec<u16>, total_rows: u16, columns_per_row: u16) -> Self {
        let mut outer = Vec::with_capacity(partitions_vec.len());
        for num_rows_in_partition in &partitions_vec {
            let rows_in_partition =
                vec![vec![None; columns_per_row as usize]; *num_rows_in_partition as usize];

            outer.push(rows_in_partition);
        }

        Self {
            inner_buf: outer,
            stdout_handle: stdout(),
            rows: total_rows,
            cols: columns_per_row,
        }
    }

    // clears stdout and sets all cells to None
    pub fn clear_all(&mut self) -> std::io::Result<()> {
        self.stdout_handle
            .execute(terminal::Clear(terminal::ClearType::All))?;

        self.clear();

        Ok(())
    }

    /// these operations are cheap because the vec object itself is a fat pointer,
    /// not the actual memory
    pub fn take_partition(&mut self, index: usize) -> Vec<Vec<Option<Cell>>> {
        std::mem::take(&mut self.inner_buf[index])
    }

    pub fn put_partition(&mut self, index: usize, partition: Vec<Vec<Option<Cell>>>) {
        self.inner_buf[index] = partition;
    }

    pub fn clear(&mut self) {
        for partition in &mut self.inner_buf {
            for row in partition {
                for cell in row {
                    cell.take();
                }
            }
        }
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn partitions(&self) -> usize {
        self.inner_buf.len()
    }

    pub fn draw_to_stdout(&mut self, prv_buf: &FrameBuffer) -> std::io::Result<()> {
        assert_eq!(self.rows, prv_buf.rows);
        assert_eq!(self.cols, prv_buf.cols);
        assert_eq!(self.inner_buf.len(), prv_buf.inner_buf.len());

        let num_partitions = self.inner_buf.len();

        let mut global_row = 0;
        for curr_partition in 0..num_partitions {
            let num_rows = self.inner_buf[curr_partition].len();
            assert_eq!(num_rows, prv_buf.inner_buf[curr_partition].len());

            for curr_row in 0..num_rows {
                let mut curr_col = 0u16;

                while curr_col < self.cols {
                    let curr_cell = &self.inner_buf[curr_partition][curr_row][curr_col as usize];
                    let prev_cell = &prv_buf.inner_buf[curr_partition][curr_row][curr_col as usize];

                    if curr_cell.eq(prev_cell) {
                        curr_col += 1;
                        continue;
                    }

                    self.stdout_handle
                        .queue(cursor::MoveTo(curr_col, global_row))?;

                    // coalesce contiguous differences within this row
                    while curr_col < self.cols {
                        let curr = &self.inner_buf[curr_partition][curr_row][curr_col as usize];
                        let prev = &prv_buf.inner_buf[curr_partition][curr_row][curr_col as usize];
                        if curr.eq(prev) {
                            break;
                        }

                        match curr {
                            Some(c) => {
                                c.draw(&mut self.stdout_handle)?;
                            }
                            None => {
                                self.stdout_handle.queue(style::Print(' '))?;
                            }
                        };

                        curr_col += 1;
                    }
                }

                global_row += 1;
            }
        }

        self.stdout_handle.flush()
    }
}
