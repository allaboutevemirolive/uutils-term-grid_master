#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(nonstandard_style)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused)]
#![deny(unsafe_code)]

//! This library arranges textual data in a grid format suitable for
//! fixed-width fonts, using an algorithm to minimise the amount of space
//! needed. For example:
//!
//! ```rust
//! use term_grid::{Grid, GridOptions, Direction, Filling, Cell};
//!
//! let mut grid = Grid::new(GridOptions {
//!     filling: Filling::Spaces(1),
//!     direction: Direction::LeftToRight,
//!     tab_size: 8,
//! });
//!
//! for s in &["one", "two", "three", "four", "five", "six", "seven",
//!            "eight", "nine", "ten", "eleven", "twelve"]
//! {
//!     grid.add(Cell::from(*s));
//! }
//!
//! println!("{}", grid.fit_into_width(24).unwrap());
//! ```
//!
//! Produces the following tabular result:
//!
//! ```text
//! one  two three  four
//! five six seven  eight
//! nine ten eleven twelve
//! ```
//!
//!
//! ## Creating a grid
//!
//! To add data to a grid, first create a new [`Grid`] value, and then add
//! cells to them with the `add` function.
//!
//! There are three options that must be specified in the [`GridOptions`] value
//! that dictate how the grid is formatted:
//!
//! - `filling`: what to put in between two columns — either a number of
//!    spaces, or a text string;
//! - `direction`, which specifies whether the cells should go along
//!    rows, or columns:
//!     - `Direction::LeftToRight` starts them in the top left and
//!        moves *rightwards*, going to the start of a new row after reaching the
//!        final column;
//!     - `Direction::TopToBottom` starts them in the top left and moves
//!        *downwards*, going to the top of a new column after reaching the final
//!        row.
//! - `tab_size`: the size of the tab field (default: 8 spaces) that will replace 8
//!    consecutive spaces with a tab in the separator.
//!
//!
//! ## Displaying a grid
//!
//! When display a grid, you can either specify the number of columns in advance,
//! or try to find the maximum number of columns that can fit in an area of a
//! given width.
//!
//! Splitting a series of cells into columns — or, in other words, starting a new
//! row every <var>n</var> cells — is achieved with the [`fit_into_columns`] function
//! on a `Grid` value. It takes as its argument the number of columns.
//!
//! Trying to fit as much data onto one screen as possible is the main use case
//! for specifying a maximum width instead. This is achieved with the
//! [`fit_into_width`] function. It takes the maximum allowed width, including
//! separators, as its argument. However, it returns an *optional* [`Display`]
//! value, depending on whether any of the cells actually had a width greater than
//! the maximum width! If this is the case, your best bet is to just output the
//! cells with one per line.
//!
//!
//! ## Cells and data
//!
//! Grids to not take `String`s or `&str`s — they take [`Cell`] values.
//!
//! A **Cell** is a struct containing an individual cell’s contents, as a string,
//! and its pre-computed length, which gets used when calculating a grid’s final
//! dimensions. Usually, you want the *Unicode width* of the string to be used for
//! this, so you can turn a `String` into a `Cell` with the `.into()` function.
//!
//! However, you may also want to supply your own width: when you already know the
//! width in advance, or when you want to change the measurement, such as skipping
//! over terminal control characters. For cases like these, the fields on the
//! `Cell` values are public, meaning you can construct your own instances as
//! necessary.
//!
//! [`Cell`]: ./struct.Cell.html
//! [`Display`]: ./struct.Display.html
//! [`Grid`]: ./struct.Grid.html
//! [`fit_into_columns`]: ./struct.Grid.html#method.fit_into_columns
//! [`fit_into_width`]: ./struct.Grid.html#method.fit_into_width
//! [`GridOptions`]: ./struct.GridOptions.html

use std::fmt;
use unicode_width::UnicodeWidthStr;

/// A **Cell** is the combination of a string and its pre-computed length.
///
/// The easiest way to create a Cell is just by using `string.into()`, which
/// uses the **unicode width** of the string (see the `unicode_width` crate).
/// However, the fields are public, if you wish to provide your own length.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Cell {
    /// The string to display when this cell gets rendered.
    pub contents: String,

    /// The pre-computed length of the string.
    pub width: Width,
}

impl From<String> for Cell {
    fn from(string: String) -> Self {
        Self {
            width: UnicodeWidthStr::width(&*string),
            contents: string,
        }
    }
}

impl<'a> From<&'a str> for Cell {
    fn from(string: &'a str) -> Self {
        Self {
            width: UnicodeWidthStr::width(string),
            contents: string.into(),
        }
    }
}

/// Direction cells should be written in — either across, or downwards.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Direction {
    /// Starts at the top left and moves rightwards, going back to the first
    /// column for a new row, like a typewriter.
    LeftToRight,

    /// Starts at the top left and moves downwards, going back to the first
    /// row for a new column, like how `ls` lists files by default.
    TopToBottom,
}

/// The width of a cell, in columns.
pub type Width = usize;

/// The text to put in between each pair of columns.
/// This does not include any spaces used when aligning cells.
#[derive(PartialEq, Eq, Debug)]
pub enum Filling {
    /// Use a specific number of spaces as the separator.
    Spaces(Width),

    /// An arbitrary string.
    /// `"|"` is a common choice.
    Text(String),
}

impl Filling {
    fn width(&self) -> Width {
        match *self {
            Filling::Spaces(w) => w,
            Filling::Text(ref t) => UnicodeWidthStr::width(&t[..]),
        }
    }
}

/// The user-assignable options for a grid view that should be passed to
/// [`Grid::new()`](struct.Grid.html#method.new).
#[derive(Debug)]
pub struct GridOptions {
    /// The direction that the cells should be written in — either
    /// across, or downwards.
    pub direction: Direction,

    /// The number of spaces to put in between each column of cells.
    pub filling: Filling,

    /// The size of the tab field based on space (default: 8 spaces).
    pub tab_size: i32,
}

#[derive(PartialEq, Eq, Debug)]
struct Dimensions {
    /// The number of lines in the grid.
    num_lines: Width,

    /// The width of each column in the grid. The length of this vector serves
    /// as the number of columns.
    widths: Vec<Width>,
}

impl Dimensions {
    fn total_width(&self, separator_width: Width) -> Width {
        if self.widths.is_empty() {
            0
        } else {
            let values = self.widths.iter().sum::<Width>();
            let separators = separator_width * (self.widths.len() - 1);
            values + separators
        }
    }
}

/// Everything needed to format the cells with the grid options.
///
/// For more information, see the [`term_grid` crate documentation](index.html).
#[derive(Debug)]
pub struct Grid {
    options: GridOptions,
    cells: Vec<Cell>,
    widest_cell_length: Width,
    width_sum: Width,
    cell_count: usize,
}

impl Grid {
    /// Creates a new grid view with the given options.
    pub fn new(options: GridOptions) -> Self {
        let cells = Vec::new();
        Self {
            options,
            cells,
            widest_cell_length: 0,
            width_sum: 0,
            cell_count: 0,
        }
    }

    /// Reserves space in the vector for the given number of additional cells
    /// to be added. (See the `Vec::reserve` function.)
    pub fn reserve(&mut self, additional: usize) {
        self.cells.reserve(additional)
    }

    /// Adds another cell onto the vector.
    pub fn add(&mut self, cell: Cell) {
        if cell.width > self.widest_cell_length {
            self.widest_cell_length = cell.width;
        }
        self.width_sum += cell.width;
        self.cell_count += 1;
        self.cells.push(cell)
    }

    /// Returns a displayable grid that’s been packed to fit into the given
    /// width in the fewest number of rows.
    ///
    /// Returns `None` if any of the cells has a width greater than the
    /// maximum width.
    pub fn fit_into_width(&self, maximum_width: Width) -> Option<Display<'_>> {
        self.width_dimensions(maximum_width).map(|dims| Display {
            grid: self,
            dimensions: dims,
        })
    }

    /// Returns a displayable grid with the given number of columns, and no
    /// maximum width.
    pub fn fit_into_columns(&self, num_columns: usize) -> Display<'_> {
        Display {
            grid: self,
            dimensions: self.columns_dimensions(num_columns),
        }
    }

    fn columns_dimensions(&self, num_columns: usize) -> Dimensions {
        let num_lines = div_ceil(self.cells.len(), num_columns);
        self.column_widths(num_lines, num_columns)
    }

    fn column_widths(&self, num_lines: usize, num_columns: usize) -> Dimensions {
        let mut widths = vec![0; num_columns];
        for (index, cell) in self.cells.iter().enumerate() {
            let index = match self.options.direction {
                Direction::LeftToRight => index % num_columns,
                Direction::TopToBottom => index / num_lines,
            };
            if cell.width > widths[index] {
                widths[index] = cell.width;
            }
        }

        Dimensions { num_lines, widths }
    }

    fn theoretical_max_num_lines(&self, maximum_width: usize) -> usize {
        // TODO: Make code readable / efficient.
        let mut widths: Vec<_> = self.cells.iter().map(|c| c.width).collect();

        // Sort widths in reverse order
        widths.sort_unstable_by(|a, b| b.cmp(a));

        let mut col_total_width_so_far = 0;
        for (i, width) in widths.iter().enumerate() {
            if width + col_total_width_so_far <= maximum_width {
                col_total_width_so_far += self.options.filling.width() + width;
            } else {
                return div_ceil(self.cell_count, i);
            }
        }

        // If we make it to this point, we have exhausted all cells before
        // reaching the maximum width; the theoretical max number of lines
        // needed to display all cells is 1.
        1
    }

    fn width_dimensions(&self, maximum_width: Width) -> Option<Dimensions> {
        if self.widest_cell_length > maximum_width {
            // Largest cell is wider than maximum width; it is impossible to fit.
            return None;
        }

        if self.cell_count == 0 {
            return Some(Dimensions {
                num_lines: 0,
                widths: Vec::new(),
            });
        }

        if self.cell_count == 1 {
            let the_cell = &self.cells[0];
            return Some(Dimensions {
                num_lines: 1,
                widths: vec![the_cell.width],
            });
        }

        let theoretical_max_num_lines = self.theoretical_max_num_lines(maximum_width);
        if theoretical_max_num_lines == 1 {
            // This if—statement is neccesary for the function to work correctly
            // for small inputs.
            return Some(Dimensions {
                num_lines: 1,
                widths: self.cells.iter().map(|cell| cell.width).collect(),
            });
        }
        // Instead of numbers of columns, try to find the fewest number of *lines*
        // that the output will fit in.
        let mut smallest_dimensions_yet = None;
        for num_lines in (1..=theoretical_max_num_lines).rev() {
            // The number of columns is the number of cells divided by the number
            // of lines, *rounded up*.
            let num_columns = div_ceil(self.cell_count, num_lines);

            // Early abort: if there are so many columns that the width of the
            // *column separators* is bigger than the width of the screen, then
            // don’t even try to tabulate it.
            // This is actually a necessary check, because the width is stored as
            // a usize, and making it go negative makes it huge instead, but it
            // also serves as a speed-up.
            let total_separator_width = (num_columns - 1) * self.options.filling.width();
            if maximum_width < total_separator_width {
                continue;
            }

            // Remove the separator width from the available space.
            let adjusted_width = maximum_width - total_separator_width;

            let potential_dimensions = self.column_widths(num_lines, num_columns);
            if potential_dimensions.widths.iter().sum::<Width>() < adjusted_width {
                smallest_dimensions_yet = Some(potential_dimensions);
            } else {
                return smallest_dimensions_yet;
            }
        }

        None
    }
}

/// A displayable representation of a [`Grid`](struct.Grid.html).
///
/// This type implements `Display`, so you can get the textual version
/// of the grid by calling `.to_string()`.
#[derive(Debug)]
pub struct Display<'grid> {
    /// The grid to display.
    grid: &'grid Grid,

    /// The pre-computed column widths for this grid.
    dimensions: Dimensions,
}

impl Display<'_> {
    /// Returns how many columns this display takes up, based on the separator
    /// width and the number and width of the columns.
    pub fn width(&self) -> Width {
        self.dimensions
            .total_width(self.grid.options.filling.width())
    }

    /// Returns how many rows this display takes up.
    pub fn row_count(&self) -> usize {
        self.dimensions.num_lines
    }

    /// Returns whether this display takes up as many columns as were allotted
    /// to it.
    ///
    /// It’s possible to construct tables that don’t actually use up all the
    /// columns that they could, such as when there are more columns than
    /// cells! In this case, a column would have a width of zero. This just
    /// checks for that.
    pub fn is_complete(&self) -> bool {
        self.dimensions.widths.iter().all(|&x| x > 0)
    }
}

impl fmt::Display for Display<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let separator = match &self.grid.options.filling {
            Filling::Spaces(n) => {
                if self.grid.options.tab_size <= 0 {
                    " ".to_string().repeat(*n)
                } else {
                    let tab_count = n / self.grid.options.tab_size as usize;
                    let remaining_spaces = n % self.grid.options.tab_size as usize;
                    "\t".repeat(tab_count) + &" ".repeat(remaining_spaces)
                }
            }
            Filling::Text(s) => s.clone(),
        };

        // Initialize a buffer of spaces. The idea here is that any cell
        // that needs padding gets a slice of this buffer of the needed
        // size. This avoids the need of creating a string of spaces for
        // each cell that needs padding.
        //
        // We overestimate how many spaces we need, but this is not
        // part of the loop and it's therefore not super important to
        // get exactly right.
        let padding = " ".repeat(self.grid.widest_cell_length);

        for y in 0..self.dimensions.num_lines {
            for x in 0..self.dimensions.widths.len() {
                let num = match self.grid.options.direction {
                    Direction::LeftToRight => y * self.dimensions.widths.len() + x,
                    Direction::TopToBottom => y + self.dimensions.num_lines * x,
                };

                // Abandon a line mid-way through if that’s where the cells end
                if num >= self.grid.cells.len() {
                    continue;
                }

                let cell = &self.grid.cells[num];
                let contents = &cell.contents;
                let last_in_row = x == self.dimensions.widths.len() - 1;

                let col_width = self.dimensions.widths[x];
                let padding_size = col_width - cell.width;

                // The final column doesn’t need to have trailing spaces,
                // as long as it’s left-aligned.
                //
                // We use write_str directly instead of a the write! macro to
                // avoid some of the formatting overhead. For example, if we pad
                // using `write!("{contents:>width}")`, the unicode width will
                // have to be independently calculated by the macro, which is slow and
                // redundant because we already know the width.
                //
                // For the padding, we instead slice into a buffer of spaces defined
                // above, so we don't need to call `" ".repeat(n)` each loop.
                // We also only call `write_str` when we actually need padding as
                // another optimization.
                f.write_str(contents)?;
                if !last_in_row {
                    if padding_size > 0 {
                        f.write_str(&padding[0..padding_size])?;
                    }
                    f.write_str(&separator)?;
                }
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

// Adapted from the unstable API:
// https://doc.rust-lang.org/std/primitive.usize.html#method.div_ceil
/// Division with upward rouding
pub const fn div_ceil(lhs: usize, rhs: usize) -> usize {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 && rhs > 0 {
        d + 1
    } else {
        d
    }
}
