use std::fs::File;
use std::io::Read;
use std::ops::AddAssign;
use std::ops::Sub;
use std::error::Error;
use std::result;
use std::fmt;
use std::collections::HashSet;

type Result<T> = result::Result<T, String>;

#[derive(Copy, Clone, Debug)]
enum Compass {
    North,
    West,
    South,
    East,
}

impl Compass {
    fn turn_left(self) -> Self {
        match self {
            Compass::North => Compass::West,
            Compass::West  => Compass::South,
            Compass::South => Compass::East,
            Compass::East  => Compass::North,
        }
    }

    fn turn_right(self) -> Self {
        match self {
            Compass::North => Compass::East,
            Compass::East  => Compass::South,
            Compass::South => Compass::West,
            Compass::West  => Compass::North,
        }
    }
}

impl Default for Compass {
    fn default() -> Compass { Compass::North }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
struct Position(i32, i32);

impl AddAssign for Position {
    fn add_assign(&mut self, other: Position) {
        *self = Position(self.0 + other.0, self.1 + other.1);
    }
}

impl Sub for Position {
    type Output = Position;
    fn sub(self, rhs: Position) -> Self::Output {
        Position(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Position {
    fn dist(self) -> i32 {
        self.0.abs() + self.1.abs()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[derive(Default)]
struct State {
    compass: Compass,
    position: Position,
}

impl State {
    fn process_cmd(&mut self, cmd: &str) -> Result<()> {
        let mut chars = cmd.chars();
        let cmd = if chars.next() == Some(' ') { chars.as_str() } else { cmd };

        let mut chars = cmd.chars();
        self.compass = match chars.next() {
                Some('L') => self.compass.turn_left(),
                Some('R') => self.compass.turn_right(),
                Some(c) => return Err(format!("invalid turn `{}`", c)),
                None    => return Err("unexpected end of command".into()),
            };

        let dist = match chars.as_str().parse::<i32>() {
                Ok(n)    => n,
                Err(err) => return Err(err.description().into()),
            };

        self.position += match self.compass {
                Compass::North => Position(dist, 0),
                Compass::West  => Position(0, -dist),
                Compass::South => Position(-dist, 0),
                Compass::East  => Position(0, dist),
            };

        Ok(())
    }
}

pub fn run() {
    let data = match import_data() {
            Ok(d)    => d,
            Err(err) => {
                println!("Error: {}", err);
                return;
            }
        };

    match solve(&data) {
        Ok(pos) =>
            println!("The map ends at {}, which {} units away.", pos, pos.dist()),
        Err(err) => println!("Error: {}", err),
    };

    match solve_hashmap(&data) {
        Ok(pos) =>
            println!("Our first point of intersection is at {}, \
                      which is {} units away", pos, pos.dist()),
        Err(err) => println!("Error: {}", err),
    }
}

fn import_data() -> Result<String> {
    let mut file = match File::open("data/day1.txt") {
            Err(err) => return Err(err.description().into()),
            Ok(n) => n,
        };

    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        return Err(e.description().into())
    };

    Ok(data)
}

fn solve(input: &str) -> Result<Position> {
    let cmds = input.split(',');
    let mut state = State::default();

    for cmd in cmds {
        state.process_cmd(cmd)?;
    }

    Ok(state.position)
}

fn solve_hashmap(input: &str) -> Result<Position> {
    let cmds = input.split(',');
    let mut state  = State::default();
    let mut crumbs: HashSet<Position> = HashSet::new();
    crumbs.insert(Position(0,0));

    for cmd in cmds {
        let mut current = state.position;
        state.process_cmd(cmd)?;

        // insert breadcrumbs into hashset.
        let delta = match state.compass {
                Compass::North => Position(1, 0),
                Compass::West  => Position(0, -1),
                Compass::South => Position(-1, 0),
                Compass::East  => Position(0, 1),
            };

        let dist = (current - state.position).dist().abs() as u32;

        // insert next crumb.  If there already is a crumb
        // we will return early and return the result.
        for _ in 0..dist {
            current += delta;
            if !crumbs.insert(current) { return Ok(current) }
        }
    }

    // If there is no intersection, we return the last position
    Ok(state.position)
}

#[cfg(test)]
mod test {
    use super::solve;

    #[test]
    fn examples() {
        // The two will be interleaved.
        assert_eq!(solve("R2, L3").unwrap(), 5);
        assert_eq!(solve("R2, R2, R2").unwrap(), 2);
        assert_eq!(solve("R5, L5, R5, R3").unwrap(), 12);
    }
}