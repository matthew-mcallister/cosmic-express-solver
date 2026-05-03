use dedent::dedent;

use super::*;

macro_rules! assert_eq_grid {
    ($lhs:expr, $rhs:expr) => {{
        let lhs = $lhs.trim();
        let rhs = $rhs.trim();
        assert!(lhs == rhs, "got:\n{}\n\nexpected:\n{}\n", lhs, rhs);
    }}
}

#[test]
fn test_parse() {
    let input = dedent!(r"
        ......
        .b....
        e....x
        ...B..
        ......
    ");
    let expected = dedent!(r"
        ......
        .b....
        .....x
        ...B..
        ......
    ");
    let grid = parse_grid(input);
    assert_eq!(grid.entry, (2, 0));
    let result = grid.to_string();
    assert_eq_grid!(result, expected);
}

#[test]
fn test_local_partition() {
    let grid = parse_grid(r"
        /////
        /.../
        //e//
        /.../
        /////
    ");
    assert!(is_local_partitioned(&grid, 2, 2));

    let grid = parse_grid(r"
        /////
        /.../
        /.e./
        /.../
        /////
    ");
    assert!(!is_local_partitioned(&grid, 2, 2));

    let grid = parse_grid(r"
        /////
        /.../
        //e./
        /.../
        /////
    ");
    assert!(!is_local_partitioned(&grid, 2, 2));

    let grid = parse_grid(r"
        /////
        /.../
        /.e./
        //.//
        /////
    ");
    assert!(is_local_partitioned(&grid, 2, 2));
}

#[test]
fn test_solvable_reachable_targets_and_exit() {
    let grid = parse_grid(r"
        ....
        .bB.
        e..x
        .oO.
    ");
    let state = State::new(grid);
    assert!(solvable(&state, 2, 1));
}

#[test]
fn test_solvable_detects_blocked_exit_or_targets() {
    let grid = parse_grid(r"
        //////
        /e..//
        ////x/
        //////
    ");
    let state = State::new(grid);
    assert!(!solvable(&state, 1, 2));
}

#[test]
fn test_solvable_does_not_search_past_exit() {
    let grid = parse_grid(r"
        //////
        /e.xb/
        //////
    ");
    let state = State::new(grid);
    assert!(!solvable(&state, 1, 2));
}

#[test]
fn test_solve_easy() {
    // custom
    let grid = parse_grid(r"
        ......
        .b.o..
        e....x
        .O.B..
        ......
    ");
    let expected = dedent!(r"
        34567.
        2/./8.
        1.109x
        ./2/67
        ..345.
    ");
    let got = format!("{}", solve(grid));
    assert_eq_grid!(got, expected);
}

#[test]
fn test_solve_medium() {
    // andromeda 9
    let grid = parse_grid(r"
        ...........
        ..........x
        ..o.B.O....
        ........b..
        ..b..B.....
        ........b..
        ..o.B.O....
        ..........e
        ...........
    ");
    let expected = dedent!(r"
        .9012345678
        .8.6543298x
        .7/7/./1076
        .6.8901./.5
        .5/98/23..4
        .4.07654/.3
        .3/1/./...2
        .212345...1
        ..09876....
    ");
    let got = format!("{}", solve(grid));
    assert_eq_grid!(got, expected);
}

#[test]
fn test_solve_hard() {
    // andromeda 14
    let grid = parse_grid(r"
       ....b...b..
       ...........
       ...........
       ....B.O.B..
       ..o........
       e...O...o.x
       ..o........
       ....B.O.B..
       ...........
       ...........
       ....b...b..
    ");
    let expected = dedent!(r"
        67../.../90
        58.23..4581
        49.145.3672
        30.0/6/2/.3
        21/9.7.1..4
        12.8/890/.x
        .3/7654321.
        .412/././0.
        .503678569.
        .694509478.
        .78./123/..
    ");
    let got = format!("{}", solve(grid));
    assert_eq_grid!(got, expected);
}

#[test]
fn test_solve_passenger_conflict() {
    // vela 5
    let mut grid = parse_grid(r"
        .x.....e.
        .........
        ..B/./B..
        ../.../..
        .........
        B./....//
        B.b/.b.b.
        ..b......
        .........
    ");
    grid.num_cars = 2;
    let expected = dedent!(r"
        .x.98761.
        .2103452.
        ..//2//3.
        ../.10/4.
        .345.965.
        /2/6787//
        /1//./8/.
        .0/65.90.
        .9874321.
    ");
    let got = format!("{}", solve(grid));
    assert_eq_grid!(got, expected);
}

#[test]
fn test_solve_two_car() {
    // vela 8
    let mut grid = parse_grid(r"
        .e........
        ..........
        ..b.....B.
        ..b.../.B.
        x.b.....B.
        ..b./...B.
        ..b.....B.
        ..........
        ..........
    ");
    grid.num_cars = 2;
    let expected = dedent!(r"
        .12345678.
        3210987690
        4./90345/1
        56/812/./2
        x7/7.109/3
        18/6/278/4
        09/5436./5
        9012345096
        8765432187
    ");
    let got = format!("{}", solve(grid));
    assert_eq_grid!(got, expected);
}

#[test]
fn test_solve_wildcard() {
    // taurus 13
    let mut grid = parse_grid(r"
        ......b
        .......
        e...O..
        .......
        ......?
        oo.....
        ......B
        .......
        x...?..
        .......
        ......b
    ");
    grid.num_cars = 2;
    let expected = dedent!(r"
        343456/
        252..7.
        161./8.
        .70109.
        .89290/
        //4381.
        765672/
        8945.3.
        x03./4.
        412985.
        321076/
    ");
    let got = format!("{}", solve(grid));
    assert_eq_grid!(got, expected);
}

#[test]
fn test_solve_green() {
    // delphinus 9
    let mut grid = parse_grid(r"
        G.....O
        .......
        .......
        ..o.g..
        e.....x
        ..o.g..
        .......
        .......
        G.....O
    ");
    grid.num_cars = 2;
    let expected = dedent!(r"
        /45678/
        .30989.
        .21670.
        ../5/1.
        1234.2x
        .././30
        ..98549
        .107678
        /23456/
    ");
    let got = format!("{}", solve(grid));
    assert_eq_grid!(got, expected);
}
