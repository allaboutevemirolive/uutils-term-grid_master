use term_grid::{Cell, Direction, Filling, Grid, GridOptions};

fn main() {
    let mut grid = Grid::new(GridOptions {
        filling: Filling::Spaces(50),
        direction: Direction::LeftToRight,
    });

    for s in &[
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
        "twelve",
    ] {
        grid.add(Cell::from(*s));
    }

    println!("{}", grid.fit_into_width(250).unwrap());
}
