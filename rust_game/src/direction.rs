use Direction::*;
#[derive(PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn describe(&self) -> String {
        match self {
            North => "north",
            East => "east",
            South => "south",
            West => "west"
        }.to_owned()
    }

    pub fn displacement(&self) -> (i32, i32) {
        match self {
            North => (0, 1),
            East => (1, 0),
            South => (0, -1),
            West => (-1, 0)
        }
    }

    pub fn go(&self, position: (i32, i32)) -> (i32, i32) {
        let d = self.displacement();
        (d.0 + position.0, d.1 + position.1)
    }

    pub fn reverse(&self) -> Self {
        match self {
            North => South,
            East => West,
            South => North,
            West => East
        }
    }
}
