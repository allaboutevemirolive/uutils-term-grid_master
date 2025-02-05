// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// spell-checker:ignore underflowed

use term_grid::{Direction, Filling, Grid, GridOptions};

#[test]
fn no_items() {
    let grid = Grid::new(
        Vec::<String>::new(),
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(2),
            width: 40,
        },
    );

    assert_eq!("", grid.to_string());
}

#[test]
fn one_item() {
    let grid = Grid::new(
        vec!["1"],
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(2),
            width: 40,
        },
    );
    assert_eq!("1\n", grid.to_string());
}

#[test]
fn one_item_exact_width() {
    let grid = Grid::new(
        vec!["1234567890"],
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(2),
            width: 10,
        },
    );

    assert_eq!("1234567890\n", grid.to_string());
}

#[test]
fn one_item_just_over() {
    let grid = Grid::new(
        vec!["1234567890!"],
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(2),
            width: 10,
        },
    );

    assert_eq!(grid.row_count(), 1);
}

#[test]
fn two_small_items() {
    let grid = Grid::new(
        vec!["1", "2"],
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(2),
            width: 40,
        },
    );

    assert_eq!(grid.width(), 1 + 2 + 1);
    assert_eq!("1  2\n", grid.to_string());
}

#[test]
fn two_medium_size_items() {
    let grid = Grid::new(
        vec!["hello there", "how are you today?"],
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(2),
            width: 40,
        },
    );

    assert_eq!(grid.width(), 11 + 2 + 18);
    assert_eq!("hello there  how are you today?\n", grid.to_string());
}

#[test]
fn two_big_items() {
    let grid = Grid::new(
        vec![
            "nuihuneihsoenhisenouiuteinhdauisdonhuisudoiosadiuohnteihaosdinhteuieudi",
            "oudisnuthasuouneohbueobaugceoduhbsauglcobeuhnaeouosbubaoecgueoubeohubeo",
        ],
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Spaces(2),
            width: 40,
        },
    );

    assert_eq!(grid.row_count(), 2);
}

#[test]
fn that_example_from_earlier() {
    let grid = Grid::new(
        vec![
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            "eleven", "twelve",
        ],
        GridOptions {
            filling: Filling::Spaces(1),
            direction: Direction::LeftToRight,
            width: 24,
        },
    );

    let bits = "one  two three  four\nfive six seven  eight\nnine ten eleven twelve\n";
    assert_eq!(grid.to_string(), bits);
    assert_eq!(grid.row_count(), 3);
}

#[test]
fn number_grid_with_pipe() {
    let grid = Grid::new(
        vec![
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            "eleven", "twelve",
        ],
        GridOptions {
            filling: Filling::Text("|".into()),
            direction: Direction::LeftToRight,
            width: 24,
        },
    );

    let bits = "one |two|three |four\nfive|six|seven |eight\nnine|ten|eleven|twelve\n";
    assert_eq!(grid.to_string(), bits);
    assert_eq!(grid.row_count(), 3);
}

#[test]
fn huge_separator() {
    let grid = Grid::new(
        vec!["a", "b"],
        GridOptions {
            filling: Filling::Spaces(100),
            direction: Direction::LeftToRight,
            width: 99,
        },
    );
    assert_eq!(grid.row_count(), 2);
}

#[test]
fn huge_yet_unused_separator() {
    let grid = Grid::new(
        vec!["abcd"],
        GridOptions {
            filling: Filling::Spaces(100),
            direction: Direction::LeftToRight,
            width: 99,
        },
    );

    assert_eq!(grid.width(), 4);
    assert_eq!("abcd\n", grid.to_string());
}

// Note: This behaviour is right or wrong depending on your terminal
// This test is mostly added so that we don't change our current
// behaviour, unless we explicitly want to do that.
#[test]
fn emoji() {
    let grid = Grid::new(
        vec!["🦀", "hello", "👩‍🔬", "hello"],
        GridOptions {
            direction: Direction::LeftToRight,
            filling: Filling::Spaces(2),
            width: 12,
        },
    );
    assert_eq!("🦀    hello\n👩‍🔬  hello\n", grid.to_string());
}

// This test once underflowed, which should never happen. The test is just
// checking that we do not get a panic.
#[test]
fn possible_underflow() {
    let cells: Vec<_> = (0..48).map(|i| 2_isize.pow(i).to_string()).collect();

    let grid = Grid::new(
        cells,
        GridOptions {
            direction: Direction::TopToBottom,
            filling: Filling::Text(" | ".into()),
            width: 15,
        },
    );

    println!("{}", grid);
}

// These test are based on the tests in uutils ls, to ensure we won't break
// it while editing this library.
mod uutils_ls {
    use super::*;

    #[test]
    fn different_widths() {
        for (width, expected) in [
            (
                100,
                "test-width-1  test-width-2  test-width-3  test-width-4\n",
            ),
            (
                50,
                "test-width-1  test-width-3\ntest-width-2  test-width-4\n",
            ),
            (
                25,
                "test-width-1\ntest-width-2\ntest-width-3\ntest-width-4\n",
            ),
        ] {
            let grid = Grid::new(
                vec![
                    "test-width-1",
                    "test-width-2",
                    "test-width-3",
                    "test-width-4",
                ],
                GridOptions {
                    direction: Direction::TopToBottom,
                    filling: Filling::Spaces(2),
                    width,
                },
            );
            assert_eq!(expected, grid.to_string());
        }
    }

    #[test]
    fn across_width_30() {
        let grid = Grid::new(
            vec![
                "test-across1",
                "test-across2",
                "test-across3",
                "test-across4",
            ],
            GridOptions {
                direction: Direction::LeftToRight,
                filling: Filling::Spaces(2),
                width: 30,
            },
        );

        assert_eq!(
            "test-across1  test-across2\ntest-across3  test-across4\n",
            grid.to_string()
        );
    }

    #[test]
    fn columns_width_30() {
        let grid = Grid::new(
            vec![
                "test-columns1",
                "test-columns2",
                "test-columns3",
                "test-columns4",
            ],
            GridOptions {
                direction: Direction::TopToBottom,
                filling: Filling::Spaces(2),
                width: 30,
            },
        );

        assert_eq!(
            "test-columns1  test-columns3\ntest-columns2  test-columns4\n",
            grid.to_string()
        );
    }

    #[test]
    fn three_short_one_long() {
        let grid = Grid::new(
            vec!["a", "b", "a-long-name", "z"],
            GridOptions {
                direction: Direction::TopToBottom,
                filling: Filling::Spaces(2),
                width: 15,
            },
        );

        assert_eq!("a  a-long-name\nb  z\n", grid.to_string());
    }
}
