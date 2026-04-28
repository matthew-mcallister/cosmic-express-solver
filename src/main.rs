use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Passenger {
    Blue,
    Orange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Content {
    None = 0,
    Rail = 1,
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

#[derive(Clone, Debug)]
struct Grid {
    width: u8,
    height: u8,
    cells: Vec<u8>,
    // Initial conditions
    entry: (u8, u8),
    passenger: Option<Passenger>,
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
    old_passenger: Option<Passenger>,
}

impl<'s> Stepper<'s> {
    fn new(state: &'s mut State) -> Self {
        if state.mutations.len() <= state.depth as usize {
            state.mutations.resize_with(state.depth as usize + 1, || Vec::with_capacity(32));
        }
        Self {
            homes_filled: 0,
            passengers_taken: 0,
            old_passenger: state.passenger,
            state,
        }
    }

    fn mutations(&mut self) -> &mut Vec<Mutation> {
        &mut self.state.mutations[self.state.depth as usize]
    }

    fn check_home(&mut self, i: u8, j: u8) {
        match self.state.grid.get(i, j) {
            Content::BlueHome if self.state.passenger == Some(Passenger::Blue) => {
                self.state.passenger = None;
                self.state.empty_homes -= 1;
                self.homes_filled += 1;
                *self.state.grid.get_mut(i, j) = Content::Obstacle as u8;
                self.mutations().push(Mutation { i, j, value: Content::BlueHome as u8 });
            },
            Content::OrangeHome if self.state.passenger == Some(Passenger::Orange) => {
                self.state.passenger = None;
                self.state.empty_homes -= 1;
                self.homes_filled += 1;
                *self.state.grid.get_mut(i, j) = Content::Obstacle as u8;
                self.mutations().push(Mutation { i, j, value: Content::OrangeHome as u8 });
            },
            _ => {},
        }
    }

    fn check_homes(&mut self) {
        if self.state.i > 0 {
            self.check_home(self.state.i - 1, self.state.j);
        }
        if self.state.i < self.state.grid.height - 1 {
            self.check_home(self.state.i + 1, self.state.j);
        }
        if self.state.j > 0 {
            self.check_home(self.state.i, self.state.j - 1);
        }
        if self.state.j < self.state.grid.width - 1 {
            self.check_home(self.state.i, self.state.j + 1);
        }
    }

    fn check_passenger(&mut self, i: u8, j: u8) {
        if self.state.passenger.is_some() { return; }
        match self.state.grid.get(i, j) {
            Content::BlueGuy => {
                self.state.passenger = Some(Passenger::Blue);
                self.passengers_taken += 1;
                self.state.waiting_passengers -= 1;
                *self.state.grid.get_mut(i, j) = Content::Obstacle as u8;
                self.mutations().push(Mutation { i, j, value: Content::BlueGuy as u8 });
            },
            Content::OrangeGuy => {
                self.state.passenger = Some(Passenger::Orange);
                self.passengers_taken += 1;
                self.state.waiting_passengers -= 1;
                *self.state.grid.get_mut(i, j) = Content::Obstacle as u8;
                self.mutations().push(Mutation { i, j, value: Content::OrangeGuy as u8 });
            },
            _ => {},
        }
    }

    fn check_passengers(&mut self) {
        if self.state.i > 0 {
            self.check_passenger(self.state.i - 1, self.state.j);
        }
        if self.state.i < self.state.grid.height - 1 {
            self.check_passenger(self.state.i + 1, self.state.j);
        }
        if self.state.j > 0 {
            self.check_passenger(self.state.i, self.state.j - 1);
        }
        if self.state.j < self.state.grid.width - 1 {
            self.check_passenger(self.state.i, self.state.j + 1);
        }
    }

    fn restore(&mut self) {
        while let Some(mutation) = self.mutations().pop() {
            *self.state.grid.get_mut(mutation.i, mutation.j) = mutation.value;
        }
        self.state.waiting_passengers += self.passengers_taken;
        self.state.empty_homes += self.homes_filled;
        self.state.passenger = self.old_passenger;
    }
}

#[derive(Clone, Debug)]
struct State {
    visited: u64,
    i: u8,
    j: u8,
    depth: u8,
    passenger: Option<Passenger>,
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
            passenger: grid.passenger,
            empty_homes: grid.cells.iter().filter(|&&cell| {
                let content = Content::from(cell);
                content == Content::BlueHome || content == Content::OrangeHome
            }).count() as u8,
            waiting_passengers: grid.cells.iter().filter(|&&cell| {
                let content = Content::from(cell);
                content == Content::BlueGuy || content == Content::OrangeGuy
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
            Content::BlueGuy | Content::OrangeGuy => {
                reachable_passengers += 1;
                continue;
            },
            Content::BlueHome | Content::OrangeHome => {
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
            if stepper.state.passenger.is_some() {
                stepper.check_homes();
                stepper.check_passengers();
            } else {
                stepper.check_passengers();
                stepper.check_homes();
            }

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
        Content::Rail
        | Content::BlueGuy
        | Content::BlueHome
        | Content::OrangeGuy
        | Content::OrangeHome
        | Content::Obstacle => Some(()),
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
fn parse_grid(grid: &str) -> Grid {
    let lines: Vec<&str> = grid.trim().lines().map(|line| line.trim()).collect();

    // Step 1: Fill grid
    let height = lines.len() as u8;
    let width = lines[0].len() as u8;
    let cells = vec![0; (width * height) as usize];
    let mut grid = Grid { entry: (0, 0), passenger: None, width, height, cells };
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
                    Content::Obstacle => '/',
                };
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[test]
fn test_parse() {
    let input = r"
......
.b....
e....x
...B..
......
";
    let expected = r"
......
.b....
.....x
...B..
......
";
    let grid = parse_grid(input);
    assert_eq!(grid.entry, (2, 0));
    let result = grid.to_string();
    assert_eq!(result.trim(), expected.trim());
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
    let expected = r"
34567.
2/./8.
1.109x
./2/67
..345.
";
    let got = format!("{}", solve(grid));
    assert_eq!(got.trim(), expected.trim());
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
    let expected = r"
.9012345678
.8.6543298x
.7/7/./1076
.6.8901./.5
.5/98/23..4
.4.07654/.3
.3/1/./...2
.212345...1
..09876....
";
    let got = format!("{}", solve(grid));
    assert_eq!(got.trim(), expected.trim());
}

fn main() {
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
    let soln = solve(grid);
    println!("{}", soln);
}
