use std::fs::File;
use std::io::Read;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::Add;
use std::error::Error;
use std::result;
use std::fmt;
use std::collections::HashSet;
use rayon::prelude::*;

type Result<T> = result::Result<T, String>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    fn as_u8(self) -> u8 {
        match self {
            Compass::North => 0,
            Compass::West  => 1,
            Compass::South => 2,
            Compass::East  => 3,
        }
    }

    fn from_u8(n: u8) -> Compass {
        match n {
            0 => Compass::North,
            1 => Compass::West,
            2 => Compass::South,
            3 => Compass::East,
            _ => unreachable!(),
        }
    }
}

impl Add for Compass {
    type Output = Compass;
    fn add(self, rhs: Compass) -> Compass {
        Compass::from_u8( (self.as_u8() + rhs.as_u8()) % 4 )
    }
}

impl Default for Compass {
    fn default() -> Compass { Compass::North }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Position(i32, i32);

impl AddAssign for Position {
    fn add_assign(&mut self, other: Position) {
        *self = Position(self.0 + other.0, self.1 + other.1);
    }
}

impl Add for Position {
    type Output = Position;
    fn add(self, rhs: Position) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
struct State {
    compass: Compass,
    position: Position,
}

impl State {
    fn process_cmd(&mut self, cmd: &str) -> Result<()> {
        let mut chars = cmd.trim_left().chars();

        self.compass = match chars.next() {
                Some('L') => self.compass.turn_left(),
                Some('R') => self.compass.turn_right(),
                Some(c) => return Err(format!("invalid turn `{}`", c)),
                None    => return Ok(()),
            };

        let dist = match chars.as_str().parse::<i32>() {
                Ok(n)    => n,
                Err(err) => return Err(err.description().into()),
            };

        self.position += match self.compass {
                Compass::North => Position(0, dist),
                Compass::West  => Position(-dist, 0),
                Compass::South => Position(0, -dist),
                Compass::East  => Position(dist, 0),
            };

        Ok(())
    }
}

impl Add for State {
    type Output = State;
    fn add(self, rhs: State) -> State {
        macro_rules! state {
            ($s:ident, $r:ident, $pos:expr) => ({
                State {
                    compass: $s.compass + $r.compass,
                    position: $s.position + $pos,
                }
            })
        }

        match self.compass {
            Compass::North => state!(self, rhs, rhs.position),
            Compass::West =>  state!(self, rhs, Position(-rhs.position.1,  rhs.position.0)),  //(x,y) -> (-y, x)
            Compass::South => state!(self, rhs, Position(-rhs.position.0, -rhs.position.1)), //(x,y) -> (-x, -y)
            Compass::East =>  state!(self, rhs, Position( rhs.position.1, -rhs.position.0)),  //(x,y) -> (y, -x)
        }
    }
}

pub fn run() {
    let data = match import_data("data/day1.txt") {
            Ok(d)    => d,
            Err(err) => {
                println!("Error: {}", err);
                return;
            }
        };

    match solve(&data) {
        Ok(s) =>
            println!("The map ends at {}, which {} units away.",
                s.position, s.position.dist()),
        Err(err) => println!("Error: {}", err),
    };

    match solve_hashmap(&data) {
        Ok(s) =>
            println!("Our first point of intersection is at {}, \
                      which is {} units away",
                      s, s.dist()),
        Err(err) => println!("Error: {}", err),
    }

    match solve_par(&data) {
        Ok(pos) =>
            println!("Par solution ends at {}, which is {} units away",
                     pos, pos.dist()),
        Err(err) => println!("Error: {}", err),
    }
}

fn import_data(f: &str) -> Result<String> {
    let mut file = match File::open(f) {
            Err(err) => return Err(err.description().into()),
            Ok(n) => n,
        };

    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        return Err(e.description().into())
    };

    Ok(data)
}

fn solve(input: &str) -> Result<State> {
    if input.is_empty() {
        return Ok(State::default())
    }

    let cmds = input.split(',');
    let mut state = State::default();

    for cmd in cmds {
        state.process_cmd(cmd)?;
    }

    Ok(state)
}

fn split(input: &str) -> (&str, &str) {
    let mut cut = input.len() / 2;

    while !input.is_char_boundary(cut) { cut += 1 }
    for c in input[cut..].chars() {
        if c == 'L' || c == 'R' { break; }
        cut += c.len_utf8();
    }

    input.split_at(cut)
}

fn solve_par(input: &str) -> Result<Position> {
    let mut cluster = [""; 32];

    fn recurse_assign<'a>(cluster: &mut [&'a str], left: &'a str, right: &'a str) {
        if cluster.len() == 2 {
            cluster[0] = left;
            cluster[1] = right;
        } else {
            let n = cluster.len() / 2;
            let (lcluster, rcluster) = cluster.split_at_mut(n);
            let (lleft, rleft)   = split(left);
            let (lright, rright) = split(right);

            recurse_assign(lcluster, lleft, rleft);
            recurse_assign(rcluster, lright, rright);
        }
    }

    let (left, right) = split(input);
    recurse_assign(&mut cluster, left, right);

    let end = cluster.into_par_iter()
        .map(|p| solve(p).unwrap())
        .reduce(|| State::default(), |a, b| a + b);

    Ok(end.position)
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
                Compass::North => Position(0, 1),
                Compass::West  => Position(-1, 0),
                Compass::South => Position(0, -1),
                Compass::East  => Position(1, 0),
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
    use super::{ State, Compass, Position, solve_par, solve, import_data };
    use test::Bencher;

    #[test]
    fn state_add() {
        let r = State {
            compass: Compass::East,
            position: Position(1, 0),
        };

        let l = State {
            compass: Compass::West,
            position: Position(-1, 0),
        };

        // Assert that binary addition works on L/R:
        assert_eq!(r + r, State { compass: Compass::South, position: Position(1, -1) });
        assert_eq!(r + l, State { compass: Compass::North, position: Position(1, 1) });
        assert_eq!(l + r, State { compass: Compass::North, position: Position(-1, 1) });
        assert_eq!(l + l, State { compass: Compass::South, position: Position(-1, -1) });

        println!("");
        println!("Addition trait:");
        println!("r + r: {:?}", r + r);
        println!("l + l: {:?}", l + l);
        println!("r + l: {:?}", r + l);
        println!("l + r: {:?}", l + r);

        println!("");
        println!("Current Solution:");
        println!("r + r: {:?}", solve("R1, R1").unwrap());
        println!("l + l: {:?}", solve("L1, L1").unwrap());
        println!("r + l: {:?}", solve("R1, L1").unwrap());
        println!("l + r: {:?}", solve("L1, R1").unwrap());
    }

    #[bench]
    fn bench_seq_large(b: &mut Bencher) {
        let data = import_data("data/day1_2.txt").unwrap();
        b.iter(|| {
            let _ = solve(&data);
        })
    }

    #[bench]
    fn bench_par_large(b: &mut Bencher) {
        let data = import_data("data/day1_2.txt").unwrap();
        b.iter(|| {
            let _ = solve_par(&data);
        })
    }

    #[bench]
    fn bench_seq_small(b: &mut Bencher) {
        let data = import_data("data/day1.txt").unwrap();
        b.iter(|| {
            let _ = solve(&data);
        })
    }

    #[bench]
    fn bench_par_small(b: &mut Bencher) {
        let data = import_data("data/day1.txt").unwrap();
        b.iter(|| {
            let _ = solve_par(&data);
        })
    }
}