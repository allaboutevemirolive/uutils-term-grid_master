// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(nonstandard_style)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused)]
#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::fmt;
use textwrap::core::display_width;

/// Direction cells should be written in: either across or downwards.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Direction {
    /// Starts at the top left and moves rightwards, going back to the first
    /// column for a new row, like a typewriter.
    LeftToRight,

    /// Starts at the top left and moves downwards, going back to the first
    /// row for a new column, like how `ls` lists files by default.
    TopToBottom,
}

/// The text to put in between each pair of columns.
///
/// This does not include any spaces used when aligning cells.
#[derive(PartialEq, Eq, Debug)]
pub enum Filling {
    /// A number of spaces
    Spaces(usize),

    /// An arbitrary string
    ///
    /// `"|"` is a common choice.
    Text(String),
}

impl Filling {
    fn width(&self) -> usize {
        match self {
            Filling::Spaces(w) => *w,
            Filling::Text(t) => display_width(t),
        }
    }
}

/// The options for a grid view that should be passed to [`Grid::new`]
#[derive(Debug)]
pub struct GridOptions {
    /// The direction that the cells should be written in
    pub direction: Direction,

    /// The string to put in between each column of cells
    pub filling: Filling,

    /// The width to fill with the grid
    pub width: usize,
}

#[derive(PartialEq, Eq, Debug)]
struct Dimensions {
    /// The number of lines in the grid.
    num_lines: usize,

    /// The width of each column in the grid. The length of this vector serves
    /// as the number of columns.
    widths: Vec<usize>,
}

impl Dimensions {
    fn total_width(&self, separator_width: usize) -> usize {
        if self.widths.is_empty() {
            0
        } else {
            let values = self.widths.iter().sum::<usize>();
            let separators = separator_width * (self.widths.len() - 1);
            values + separators
        }
    }
}

/// Everything needed to format the cells with the grid options.
#[derive(Debug)]
pub struct Grid<T: AsRef<str>> {
    options: GridOptions,
    cells: Vec<T>,
    widths: Vec<usize>,
    widest_cell_width: usize,
    dimensions: Dimensions,
}

impl<T: AsRef<str>> Grid<T> {
    /// Creates a new grid view with the given cells and options
    pub fn new(cells: Vec<T>, options: GridOptions) -> Self {
        let widths: Vec<usize> = cells.iter().map(|c| display_width(c.as_ref())).collect();
        let widest_cell_width = widths.iter().copied().max().unwrap_or(0);
        let width = options.width;

        let mut grid = Self {
            options,
            cells,
            widths,
            widest_cell_width,
            dimensions: Dimensions {
                num_lines: 0,
                widths: Vec::new(),
            },
        };

        grid.dimensions = grid.width_dimensions(width).unwrap_or(Dimensions {
            num_lines: grid.cells.len(),
            widths: vec![widest_cell_width],
        });

        grid
    }

    /// The number of terminal columns this display takes up, based on the separator
    /// width and the number and width of the columns.
    pub fn width(&self) -> usize {
        self.dimensions.total_width(self.options.filling.width())
    }

    /// The number of rows this display takes up.
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

    fn column_widths(&self, num_lines: usize, num_columns: usize) -> Dimensions {
        let mut column_widths = vec![0; num_columns];
        for (index, cell_width) in self.widths.iter().copied().enumerate() {
            let index = match self.options.direction {
                Direction::LeftToRight => index % num_columns,
                Direction::TopToBottom => index / num_lines,
            };
            if cell_width > column_widths[index] {
                column_widths[index] = cell_width;
            }
        }

        Dimensions {
            num_lines,
            widths: column_widths,
        }
    }

    fn theoretical_max_num_lines(&self, maximum_width: usize) -> usize {
        // TODO: Make code readable / efficient.
        let mut widths = self.widths.clone();

        // Sort widths in reverse order
        widths.sort_unstable_by(|a, b| b.cmp(a));

        let mut col_total_width_so_far = 0;
        for (i, width) in widths.iter().enumerate() {
            if width + col_total_width_so_far <= maximum_width {
                col_total_width_so_far += self.options.filling.width() + width;
            } else {
                return div_ceil(self.cells.len(), i);
            }
        }

        // If we make it to this point, we have exhausted all cells before
        // reaching the maximum width; the theoretical max number of lines
        // needed to display all cells is 1.
        1
    }

    fn width_dimensions(&self, maximum_width: usize) -> Option<Dimensions> {
        if self.widest_cell_width > maximum_width {
            // Largest cell is wider than maximum width; it is impossible to fit.
            return None;
        }

        if self.cells.is_empty() {
            return Some(Dimensions {
                num_lines: 0,
                widths: Vec::new(),
            });
        }

        if self.cells.len() == 1 {
            let cell_widths = self.widths[0];
            return Some(Dimensions {
                num_lines: 1,
                widths: vec![cell_widths],
            });
        }

        let theoretical_max_num_lines = self.theoretical_max_num_lines(maximum_width);
        if theoretical_max_num_lines == 1 {
            // This if—statement is necessary for the function to work correctly
            // for small inputs.
            return Some(Dimensions {
                num_lines: 1,
                widths: self.widths.clone(),
            });
        }
        // Instead of numbers of columns, try to find the fewest number of *lines*
        // that the output will fit in.
        let mut smallest_dimensions_yet = None;
        for num_lines in (1..=theoretical_max_num_lines).rev() {
            // The number of columns is the number of cells divided by the number
            // of lines, *rounded up*.
            let num_columns = div_ceil(self.cells.len(), num_lines);

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
            if potential_dimensions.widths.iter().sum::<usize>() < adjusted_width {
                smallest_dimensions_yet = Some(potential_dimensions);
            } else {
                return smallest_dimensions_yet;
            }
        }

        None
    }
}

impl<T: AsRef<str>> fmt::Display for Grid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let separator = match &self.options.filling {
            Filling::Spaces(n) => " ".repeat(*n),
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
        let padding = " ".repeat(self.widest_cell_width);

        for y in 0..self.dimensions.num_lines {
            for x in 0..self.dimensions.widths.len() {
                let num = match self.options.direction {
                    Direction::LeftToRight => y * self.dimensions.widths.len() + x,
                    Direction::TopToBottom => y + self.dimensions.num_lines * x,
                };

                // Abandon a line mid-way through if that’s where the cells end
                if num >= self.cells.len() {
                    continue;
                }

                let contents = &self.cells[num];
                let width = self.widths[num];
                let last_in_row = x == self.dimensions.widths.len() - 1;

                let col_width = self.dimensions.widths[x];
                let padding_size = col_width - width;

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
                f.write_str(contents.as_ref())?;
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
// Can be removed on MSRV 1.73.
/// Division with upward rounding
pub const fn div_ceil(lhs: usize, rhs: usize) -> usize {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 && rhs > 0 {
        d + 1
    } else {
        d
    }
}
