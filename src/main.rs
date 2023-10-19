use term_grid::{Cell, Direction, Filling, Grid, GridOptions};

fn main() {
    // println!("Hello, Rust!");

    // let mut grid = Grid::new(GridOptions::default());

    let mut grid = Grid::new(GridOptions {
        filling: Filling::Spaces(10),
        direction: Direction::LeftToRight,
        // ..Default::default()
        tab_size: 0,
    });

    for s in &[
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
        "twelve",
    ] {
        grid.add(Cell::from(*s));
    }

    println!("{}", grid.fit_into_width(50).unwrap());
}
