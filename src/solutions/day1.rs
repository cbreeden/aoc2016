use std::fs::File;
use std::io::BufReader;
use std::result;

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

enum Turn {
    Left,
    Right,
}

fn run() {

}

fn solve(input: &str) -> Result<u32> {
    let data = BufReader::new(File::open("../day1.txt")
        .or(Err("unable to open data file '../day1.txt".to_string())?);

    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn examples() {
        // The two will be interleaved.
        assert_eq!(solve("R2, L3"), 5);
        assert_eq!(solve("R2, R2, R2"), 2);
        assert_eq!(solve("R5, L5, R5, R3"), 12);
    }
}