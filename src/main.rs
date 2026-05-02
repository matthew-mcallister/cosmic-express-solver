use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Passenger {
    Blue,
    Orange,
    Green,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Content {
    None = 0,
    Rail = 1,
    GreenGuy = 247,
    GreenHome = 248,
    WildcardHome = 249,
    Exit = 250,
    BlueGuy = 251,
    BlueHome = 252,
    OrangeGuy = 253,
    OrangeHome = 254,
    Obstacle = 255,
}

impl Default for Content {
    fn default() -> Self {
        Content::None
    }
}

impl From<u8> for Content {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            247 => Self::GreenGuy,
            248 => Self::GreenHome,
            249 => Self::WildcardHome,
            250 => Self::Exit,
            251 => Self::BlueGuy,
            252 => Self::BlueHome,
            253 => Self::OrangeGuy,
            254 => Self::OrangeHome,
            255 => Self::Obstacle,
            _ => Self::Rail,
        }
    }
}

impl Content {
    fn passenger(self) -> Option<Passenger> {
        match self {
            Content::BlueGuy => Some(Passenger::Blue),
            Content::OrangeGuy => Some(Passenger::Orange),
            Content::GreenGuy => Some(Passenger::Green),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
struct Grid {
    width: u8,
    height: u8,
    cells: Vec<u8>,
    // Initial conditions
    entry: (u8, u8),
    num_cars: u8,
}

impl Grid {
    fn get(&self, i: u8, j: u8) -> Content {
        Content::from(self.get_u8(i, j))
    }

    fn get_u8(&self, i: u8, j: u8) -> u8 {
        let index = (i as usize) * (self.width as usize) + (j as usize);
        self.cells[index]
    }

    fn get_signed(&self, i: i8, j: i8) -> Content {
        if i < 0 || j < 0 {
            return Content::Obstacle;
        }
        let i = i as u8;
        let j = j as u8;
        if i >= self.height || j >= self.width {
            return Content::Obstacle;
        }
        self.get(i, j)
    }

    fn get_mut(&mut self, i: u8, j: u8) -> &mut u8 {
        let index = (i as usize) * (self.width as usize) + (j as usize);
        &mut self.cells[index]
    }
}

fn is_local_partitioned(grid: &Grid, i: u8, j: u8) -> bool {
    let i = i as i8;
    let j = j as i8;
    let ring = [
        grid.get_signed(i - 1, j - 1),
        grid.get_signed(i - 1, j),
        grid.get_signed(i - 1, j + 1),
        grid.get_signed(i, j + 1),
        grid.get_signed(i + 1, j + 1),
        grid.get_signed(i + 1, j),
        grid.get_signed(i + 1, j - 1),
        grid.get_signed(i, j - 1),
    ];

    let mut open_runs = 0;
    let mut was_open = ring[ring.len() - 1] == Content::None;
    for &cell in &ring {
        let is_open = cell == Content::None;
        if is_open && !was_open {
            open_runs += 1;
        }
        was_open = is_open;
    }

    open_runs > 1
}

/// Allows restoring the original state of the grid by backtracking
#[derive(Clone, Debug)]
struct Mutation {
    i: u8,
    j: u8,
    value: u8,
}

#[derive(Debug)]
struct Stepper<'s> {
    state: &'s mut State,
    homes_filled: u8,
    passengers_taken: u8,
    old_cars: [Car; 3],
}

impl<'s> Stepper<'s> {
    fn new(state: &'s mut State) -> Self {
        if state.mutations.len() <= state.depth as usize {
            state.mutations.resize_with(state.depth as usize + 1, || Vec::with_capacity(32));
        }
        Self {
            homes_filled: 0,
            passengers_taken: 0,
            old_cars: state.cars,
            state,
        }
    }

    fn mutations(&mut self) -> &mut Vec<Mutation> {
        &mut self.state.mutations[self.state.depth as usize]
    }

    fn check_home(&mut self, car: usize, i: u8, j: u8) {
        let content = self.state.grid.get(i, j);
        match content {
            Content::BlueHome if self.state.cars[car].passenger == Some(Passenger::Blue) => {},
            Content::OrangeHome if self.state.cars[car].passenger == Some(Passenger::Orange) => {},
            Content::GreenHome if self.state.cars[car].passenger == Some(Passenger::Green) => {},
            Content::WildcardHome if self.state.cars[car].passenger.is_some() => {},
            _ => return,
        }

        self.state.cars[car].passenger = None;
        self.state.empty_homes -= 1;
        self.homes_filled += 1;
        *self.state.grid.get_mut(i, j) = Content::Obstacle as u8;
        self.mutations().push(Mutation { i, j, value: content as u8 });
    }

    fn check_homes(&mut self, car_idx: usize) {
        let car = self.state.cars[car_idx];
        if car.passenger.is_none() { return; }
        let (i, j) = (car.i, car.j);
        if i > 0 {
            self.check_home(car_idx, i - 1, j);
        }
        if i < self.state.grid.height - 1 {
            self.check_home(car_idx, i + 1, j);
        }
        if j > 0 {
            self.check_home(car_idx, i, j - 1);
        }
        if j < self.state.grid.width - 1 {
            self.check_home(car_idx, i, j + 1);
        }
    }

    fn check_passenger(&mut self, car: usize, i: u8, j: u8) {
        // Cannot pick up if car full
        if self.state.cars[car].passenger.is_some() { return; }

        let content = self.state.grid.get(i, j);
        let passenger = match content {
            Content::BlueGuy => Passenger::Blue,
            Content::OrangeGuy => Passenger::Orange,
            Content::GreenGuy => Passenger::Green,
            _ => return,
        };

        // Cannot pick up if car stinks
        if passenger != Passenger::Green && self.state.cars[car].stinks {
            return;
        }

        // Cannot pick up if there is another passenger waiting on the exact
        // opposite side of the car (must have a unique choice of passenger)
        let v = (i as i8 - self.state.cars[car].i as i8, j as i8 - self.state.cars[car].j as i8);
        let flip = (-v.0, -v.1);
        let opposite = (self.state.cars[car].i as i8 + flip.0, self.state.cars[car].j as i8 + flip.1);
        if let Some(other) = self.state.grid.get_signed(opposite.0, opposite.1).passenger() {
            // Exception: If the car stinks and the other passenger is not
            // green, they will not attempt to board
            if !(self.state.cars[car].stinks && other != Passenger::Green) {
                return;
            }
        }

        self.state.cars[car].passenger = Some(passenger);
        if passenger == Passenger::Green {
            self.state.cars[car].stinks = true;
        }
        self.passengers_taken += 1;
        self.state.waiting_passengers -= 1;
        *self.state.grid.get_mut(i, j) = Content::Obstacle as u8;
        self.mutations().push(Mutation { i, j, value: content as u8 });
    }

    fn check_passengers(&mut self, car_idx: usize) {
        let car = &self.state.cars[car_idx];
        if car.passenger.is_some() { return; }
        let (i, j) = (car.i, car.j);
        if i > 0 {
            self.check_passenger(car_idx, i - 1, j);
        }
        if i < self.state.grid.height - 1 {
            self.check_passenger(car_idx, i + 1, j);
        }
        if j > 0 {
            self.check_passenger(car_idx, i, j - 1);
        }
        if j < self.state.grid.width - 1 {
            self.check_passenger(car_idx, i, j + 1);
        }
    }

    fn update_cars(&mut self) {
        self.state.cars[0].i = self.state.i;
        self.state.cars[0].j = self.state.j;
        for car in 1..self.state.num_cars as usize {
            self.state.cars[car].i = self.old_cars[car - 1].i;
            self.state.cars[car].j = self.old_cars[car - 1].j;
        }
        for car in 0..self.state.num_cars as usize {
            // Sometimes we can take two actions on the same turn, so need to
            // handle both load -> unload and unload -> load orders.
            self.check_homes(car);
            self.check_passengers(car);
            self.check_homes(car);
        }
    }

    fn restore(&mut self) {
        while let Some(mutation) = self.mutations().pop() {
            *self.state.grid.get_mut(mutation.i, mutation.j) = mutation.value;
        }
        self.state.waiting_passengers += self.passengers_taken;
        self.state.empty_homes += self.homes_filled;
        self.state.cars = self.old_cars;
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Car {
    i: u8,
    j: u8,
    passenger: Option<Passenger>,
    stinks: bool,
}

#[derive(Clone, Debug)]
struct State {
    visited: u64,
    i: u8,
    j: u8,
    depth: u8,
    num_cars: u8,
    cars: [Car; 3],
    waiting_passengers: u8,
    empty_homes: u8,
    grid: Grid,
    // Manual memory management
    mutations: Vec<Vec<Mutation>>,
}

impl State {
    fn new(grid: Grid) -> Self {
        Self {
            visited: 0,
            i: grid.entry.0,
            j: grid.entry.1,
            depth: 0,
            num_cars: grid.num_cars,
            cars: [Car {
                i: grid.entry.0,
                j: grid.entry.0,
                passenger: None,
                stinks: false,
            }; 3],
            empty_homes: grid.cells.iter().filter(|&&cell| {
                let content = Content::from(cell);
                matches!(content, Content::BlueHome | Content::OrangeHome | Content::GreenHome | Content::WildcardHome)
            }).count() as u8,
            waiting_passengers: grid.cells.iter().filter(|&&cell| {
                let content = Content::from(cell);
                matches!(content, Content::BlueGuy | Content::OrangeGuy | Content::GreenGuy)
            }).count() as u8,
            grid,
            mutations: Vec::with_capacity(100),
        }
    }
}

// Does DFS to check that all passengers/homes are reachable from this
// location. This is slightly expensive but can trim a massive amount of
// search space.
fn solvable(state: &State, i: u8, j: u8) -> bool {
    let width = state.grid.width as usize;
    let height = state.grid.height as usize;
    let mut seen = vec![0_u8; width * height];
    let mut stack = Vec::with_capacity(width * height);
    let mut reachable_passengers = 0_u8;
    let mut reachable_homes = 0_u8;
    let mut exit_found = false;

    stack.push((i, j));

    while let Some((row, col)) = stack.pop() {
        let index = (row as usize) * width + (col as usize);
        if seen[index] != 0 {
            continue;
        }
        seen[index] = 1;

        match state.grid.get(row, col) {
            Content::None => {},
            Content::Exit => {
                exit_found = true;
                continue;
            },
            Content::BlueGuy | Content::OrangeGuy | Content::GreenGuy => {
                reachable_passengers += 1;
                continue;
            },
            Content::BlueHome
            | Content::OrangeHome
            | Content::GreenHome
            | Content::WildcardHome => {
                reachable_homes += 1;
                continue;
            },
            Content::Rail | Content::Obstacle => {
                continue;
            },
        }

        if row > 0 {
            stack.push((row - 1, col));
        }
        if row + 1 < state.grid.height {
            stack.push((row + 1, col));
        }
        if col > 0 {
            stack.push((row, col - 1));
        }
        if col + 1 < state.grid.width {
            stack.push((row, col + 1));
        }
    }

    reachable_passengers == state.waiting_passengers
        && reachable_homes == state.empty_homes
        && exit_found
}

// Return None when solution is found to short circuit the search
fn dfs(state: &mut State, i: u8, j: u8, depth: u8, check_reachability: bool) -> Option<()> {
    match state.grid.get(i, j) {
        Content::None => {
            if check_reachability && !solvable(state, i, j) {
                return Some(());
            }

            state.visited += 1;
            if state.visited % 10_000_000 == 0 {
                println!("{}", state.grid);
            }

            // Update state
            let (old_x, old_y) = (state.i, state.j);
            state.i = i;
            state.j = j;
            state.depth += 1;
            *state.grid.get_mut(i, j) = state.depth;
            let mut stepper = Stepper::new(state);
            stepper.mutations().push(Mutation { i, j, value: Content::None as u8 });
            stepper.update_cars();

            // Detect whether local neighborhood (3x3) is partitioned
            let is_partitioned = is_local_partitioned(&stepper.state.grid, i, j);

            // Recursion
            if i > 0 {
                dfs(stepper.state, i - 1, j, depth, is_partitioned)?;
            }
            if i < stepper.state.grid.height - 1 {
                dfs(stepper.state, i + 1, j, depth, is_partitioned)?;
            }
            if j > 0 {
                dfs(stepper.state, i, j - 1, depth, is_partitioned)?;
            }
            if j < stepper.state.grid.width - 1 {
                dfs(stepper.state, i, j + 1, depth, is_partitioned)?;
            }

            // Restore state
            stepper.restore();
            state.i = old_x;
            state.j = old_y;
            state.depth -= 1;

            Some(())
        },
        Content::Exit => {
            if state.empty_homes == 0 {
                None
            } else {
                Some(())
            }
        },
        _ => Some(()),
    }
}

fn solve(grid: Grid) -> Grid {
    let entry = grid.entry;
    let mut state = State::new(grid);
    dfs(&mut state, entry.0, entry.1, 0, false);
    state.grid
}

// . = nothing
// / = obstacle
// x = exit
// b = blue guy
// B = blue home
// o = orange guy
// O = orange home
// ? = wildcard home
fn parse_grid(grid: &str) -> Grid {
    let lines: Vec<&str> = grid.trim().lines().map(|line| line.trim()).collect();

    // Step 1: Fill grid
    let height = lines.len() as u8;
    let width = lines[0].len() as u8;
    let cells = vec![0; (width * height) as usize];
    let mut grid = Grid { entry: (0, 0), num_cars: 1, width, height, cells };
    for (i, line) in lines.into_iter().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            let cell = match ch {
                '.' => Content::None,
                '#' => Content::Rail,
                '/' => Content::Obstacle,
                'e' => {
                    grid.entry = (i as u8, j as u8);
                    Content::None
                },
                'x' => Content::Exit,
                'b' => Content::BlueGuy,
                'B' => Content::BlueHome,
                'o' => Content::OrangeGuy,
                'O' => Content::OrangeHome,
                'g' => Content::GreenGuy,
                'G' => Content::GreenHome,
                '?' => Content::WildcardHome,
                _ => panic!("Invalid character in grid: {}", ch),
            };
            *grid.get_mut(i as u8, j as u8) = cell as u8;
        }
    }

    grid
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.height {
            for j in 0..self.width {
                let ch = match self.get(i, j) {
                    Content::None => '.',
                    Content::Rail => ('0' as u8 + self.get_u8(i, j) % 10) as char,
                    Content::Exit => 'x',
                    Content::BlueGuy => 'b',
                    Content::BlueHome => 'B',
                    Content::OrangeGuy => 'o',
                    Content::OrangeHome => 'O',
                    Content::GreenGuy => 'g',
                    Content::GreenHome => 'G',
                    Content::Obstacle => '/',
                    Content::WildcardHome => '?',
                };
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
            //.//
            /.../
            /////
        ");
        assert!(is_local_partitioned(&grid, 2, 2));

        let grid = parse_grid(r"
            /////
            /.../
            /.../
            /.../
            /////
        ");
        assert!(!is_local_partitioned(&grid, 2, 2));

        let grid = parse_grid(r"
            /////
            /.../
            //../
            /.../
            /////
        ");
        assert!(!is_local_partitioned(&grid, 2, 2));

        let grid = parse_grid(r"
            /////
            /.../
            /.../
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
        let mut grid = parse_grid(r"
            ...........
            ...........
            .......?...
            .o.b../?.b.
            .o.....?.b.
            ...........
            ..........x
            ...........
        ");
        grid.num_cars = 2;
        let expected = dedent!(r"
            1.7834..12.
            2.6925..03.
            3.5016./945
            4/4b.7//8/6
            5/3..8./7/7
            612..9236.8
            70...0145.x
            89.........
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
}

fn main() {
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
    let soln = solve(grid);
    println!("{}", soln);
}
