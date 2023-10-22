use term_grid::{Grid, GridOptions, Direction, Filling, Cell};

fn main() {
    let mut grid = Grid::new(GridOptions {
        filling:     Filling::Spaces(1),
        direction:   Direction::LeftToRight,
    });
    
    for s in &["one", "two", "three", "four", "five", "six", "seven",
               "eight", "nine", "ten", "eleven", "twelve"]
    {
        grid.add(Cell::from(*s));
    }
    
    println!("{}", grid.fit_into_width(24).unwrap());
}