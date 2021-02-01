use std::collections::HashMap;
use std::cmp::Ordering;

use crate::direction::Direction;
use ExitStatus::*;
use RoomStatus::*;

#[derive(PartialEq)]
pub enum ExitStatus {
    Cleared,
    Blocked
}

impl ExitStatus {
    pub fn describe(&self) -> String {
        match self {
            Cleared => "",
            Blocked => " blocked"
        }.to_owned()
    }
}

#[derive(PartialEq)]
pub enum RoomStatus {
    Entrance,
    Empty,
    TreasureFilled
}

impl RoomStatus {
    pub fn describe(&self) -> String {
        match self {
            Entrance => "an entrance room",
            Empty => "a room",
            TreasureFilled => "a treasure-filled room"
        }.to_owned()
    }
}


pub struct Room {
    pub exits: HashMap<Direction, ExitStatus>,
    pub status: RoomStatus,
    pub infested: i8
}

impl Room {
    pub fn describe(&self) -> String {
        let out = format!("You find yourself in {}{}. {}", 
                self.status.describe(),
                self.describe_infestation(),
                self.describe_exits());
        out
    }

    pub fn describe_exits(&self) -> String {
        match self.exits.len() {
            0 => "There are no exits.".to_owned(),
            _ => {
                let mut temp = "There is ".to_owned();
                let mut iter = self.exits.iter();
                for i in 0..self.exits.len() {
                    let (direction, status) = iter.next().unwrap();
                    let formatted_str = match (i + 2).cmp(&self.exits.len()) {
                        Ordering::Less => {
                            format!("one{} exit to the {}, ", status.describe(), direction.describe())
                        },
                        Ordering::Equal => {
                            format!("one{} exit to the {} and ", status.describe(), direction.describe())
                        },
                        Ordering::Greater => {
                            format!("one{} exit to the {}.", status.describe(), direction.describe())
                        }
                    };
                    temp.push_str(&formatted_str);
                }
                temp
            }
        }
    }

    pub fn describe_infestation(&self) -> String {
        let lim = self.infested_deadly_limit();
        if self.infested <= 0 {
            ""
        } else if self.infested < lim - 2 {
            " where you can hear faint slithering noises"
        } else if self.infested <= lim {
            " with many snakes pouring in"
        } else if self.infested == lim {
            " where you are surrounded by snakes"
        } else {
            " that has echoes of the void"
        }.to_owned()
    }

    pub fn infested_deadly_limit(&self) -> i8 {
        12
    }

    pub fn is_infested(&self) -> bool {
        (1..(self.infested_deadly_limit() + 1)).contains(&self.infested)
    }

    pub fn infest(&mut self) {
        self.infested = std::cmp::min(self.infested + 1, self.infested_deadly_limit())
    }

    pub fn is_deadly(&self) -> bool {
        self.infested >= self.infested_deadly_limit()
    }
}
