use std::io;
use std::collections::HashMap;

use Command::*;

mod room;
use room::*;
use room::ExitStatus::*;
use room::RoomStatus::*;

mod direction;
use direction::*;
use direction::Direction::*;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

#[derive(PartialEq)]
enum Command {
    Go(Direction),
    Clear(Direction),
    Take,
    Leave
}

fn can_go(
    position: &(i32, i32), 
    rooms: &HashMap<(i32, i32), room::Room>, 
    dir: &Direction
) -> bool {
    go(position, rooms, dir).0 != *position
}

fn go(
    position: &(i32, i32), 
    rooms: &HashMap<(i32, i32), Room>, 
    dir: &Direction
) -> ((i32, i32), Vec<String>) {
    // try to move in a direction
    let mut out_strs: Vec<String> = Vec::new();
    match rooms.get(&position) {
        None => {
            out_strs.push("You are in the void. You are trapped, forever.".to_string());
            return (*position, out_strs)
        },
        Some(room) => match room.exits.get(&dir) {
            None => {
                out_strs.push(format!("You cannot go {}. There is no exit!", dir.describe()));
                return (*position, out_strs)
            },
            Some(Blocked) => {
                out_strs.push(format!("You cannot go {}. The exit is blocked!", dir.describe()));
                return (*position, out_strs);
            },
            _ => ()
        }
    }
    let next_position = dir.go(*position);
    if let Some(next_room) = rooms.get(&next_position) {
        match next_room.exits.get(&dir.reverse()) {
            None => {
                out_strs.push(format!("You cannot go {}. There is no entrance to the new room.", 
                    dir.describe()));
                return (*position, out_strs);
            },
            Some(Blocked) => {
                out_strs.push(format!("You cannot go {}. The entrance to the new room is blocked!", 
                    dir.describe()));
                return (*position, out_strs);
            },
            Some(Cleared) => {
                out_strs.push(format!("Moving {}.", dir.describe()));
                return (next_position, out_strs);
            }
        }
    }
    out_strs.push(format!("You cannot go {}. There is no room connected to the exit!", 
                dir.describe()));
    (*position, out_strs)
}

fn clear(
    position: &(i32, i32), 
    rooms: &mut HashMap<(i32, i32), Room>, 
    dir: Direction
) -> ((i32, i32), Vec<String>) { 
    // try to clear an entrance in a direction
    let mut out_strs: Vec<String> = Vec::new();
    match rooms.get(&position) {
        None => {
            out_strs.push("You are in the void. There is nothing to clear.".to_string());
            return (*position, out_strs)
        },
        Some(Room {exits, .. }) => match exits.get(&dir) {
            None => {
                out_strs.push(format!("You cannot clear the {} exit. There is no exit!", dir.describe()));
                return (*position, out_strs)
            },
            Some(Blocked) => {
                out_strs.push(format!("You clear the {}ern exit.", dir.describe()));
                rooms.get_mut(&position).unwrap().exits.insert(dir, Cleared);
                return (*position, out_strs);
            },
            _ => ()
        }
    }
    let next_position = dir.go(*position);
    if let Some(next_room) = rooms.get(&next_position) {
        match next_room.exits.get(&dir.reverse()) {
            Some(Blocked) => {
                out_strs.push(format!("You try to clear the entrance to the {}ern room from this side, but you cannot.", 
                    dir.describe()));
                return (*position, out_strs);
            },
            None => {
                out_strs.push(format!("The {}ern exit is already clear, but there is no entrance to the new room.", dir.describe()));
                return (*position, out_strs);
            },
            _ => ()
        }
    }
    out_strs.push(format!("The {}ern exit is already clear.", dir.describe()));
    (*position, out_strs)
}

fn take(
    position: &(i32, i32), 
    rooms: &mut HashMap<(i32, i32), Room>
) -> ((i32, i32), Vec<String>) { 
    // try to take the treasure
    let mut out_strs: Vec<String> = Vec::new();
    match rooms.get(&position) {
        None => {
            out_strs.push("You are in the void. You are trapped, forever.".to_string());
        },
        Some(Room {status: TreasureFilled, ..}) => {
            out_strs.push("You take the treasure.".to_string());
            let room = rooms.get_mut(&position).unwrap();
            room.infested = std::cmp::max(1, room.infested);
            room.status = Empty;
        },
        _ => {
            out_strs.push("There is nothing to take.".to_string());
        }
    }
    (*position, out_strs)
}

fn leave(
    position: &(i32, i32), 
    rooms: &HashMap<(i32, i32), Room>, 
    finished_game: &mut bool
) -> ((i32, i32), Vec<String>) { 
    // try to leave the ruins
    let mut out_strs: Vec<String> = Vec::new();
    let at_entrance = rooms.get(&position).map_or(false, |room| room.status == Entrance);
    let taken_treasure = rooms.values().all(|room| room.status != TreasureFilled);
    if at_entrance && taken_treasure {
        *finished_game = true;
        out_strs.push("You have left with the treasure. Congratulations!".to_owned());
    } else if taken_treasure {
        out_strs.push("You need to get to an entrance to leave.".to_owned());
    } else {
        out_strs.push("You do not have all the treasure. You cannot leave.".to_owned());
    }
    (*position, out_strs)
}

fn update_rooms(rooms: &mut HashMap<(i32, i32), Room>) {
    fn has_infested_neighbor(position: &(i32, i32), rooms: &HashMap<(i32, i32), Room>) -> bool {
        vec![North, South, East, West]
            .iter()
            .any( |dir| can_go(position, rooms, dir) && 
                match rooms.get(&dir.go(*position)) {
                    Some(room) => room.is_infested(),
                    _ => false
                })
    }
    let positions_to_infest: Vec<_> = rooms
        .iter()
        .filter(|&(pos, room)| room.is_infested() || has_infested_neighbor(pos, rooms))
        .map(|(k, _)| *k)
        .collect();
    for pos in positions_to_infest {
        rooms.get_mut(&pos).unwrap().infest()
    }
}

fn get_command() -> Command {
    let mut command: Option<Command> = None;
    while command.is_none() {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        command = match input.trim() {
            "n" => Some(Go(North)),
            "s" => Some(Go(South)),
            "e" => Some(Go(East)),
            "w" => Some(Go(West)),
            "cn" => Some(Clear(North)),
            "cs" => Some(Clear(South)),
            "ce" => Some(Clear(East)),
            "cw" => Some(Clear(West)),
            "t" => Some(Take),
            "l" => Some(Leave),
            "help" => {
                println!("type n, s, e, w to move in those directions");
                println!("type 'c' in front to clear in those direction (i.e., 'cn')");
                println!("type 't' to take something in the room");
                println!("type 'l' to leave if you are at an entrance with all the treasure");
                None
            },
            _ => {
                println!("command not recognized, try again! Type help for help.");
                None
            }
        }
    }
    command.unwrap()
}

fn get_rooms() -> HashMap<(i32, i32), Room> {
    hashmap![
        (0, 0) => Room {
            exits: hashmap![North => Cleared],
            status: Entrance,
            infested: 0
        },
        (0, 1) => Room {
            exits: hashmap![South => Cleared, North => Blocked],
            status: TreasureFilled,
            infested: 0
        },
        (0, 2) => Room {
            exits: hashmap![South => Cleared, North => Cleared],
            status: Empty,
            infested: 0
        },
        (0, 3) => Room {
            exits: hashmap![South => Cleared, North => Cleared],
            status: Empty,
            infested: 0
        },
        (0, 4) => Room {
            exits: hashmap![South => Cleared, East => Cleared, North => Blocked],
            status: Empty,
            infested: 0
        },
        (1, 4) => Room {
            exits: hashmap![West => Cleared, East => Blocked],
            status: Empty,
            infested: 0
        },
        (2, 4) => Room {
            exits: hashmap![West => Cleared, North => Cleared],
            status: Empty,
            infested: 0
        },
        (2, 5) => Room {
            exits: hashmap![South => Cleared, West => Cleared],
            status: Empty,
            infested: 0
        },
        (1, 5) => Room {
            exits: hashmap![East => Cleared, North => Blocked, West => Blocked],
            status: Empty,
            infested: 0
        },
        (1, 6) => Room {
            exits: hashmap![South => Cleared],
            status: Empty,
            infested: 0
        },
        (0, 5) => Room {
            exits: hashmap![East => Cleared, West => Blocked, South => Blocked],
            status: Empty,
            infested: 0
        },
        (-1, 5) => Room {
            exits: hashmap![East => Cleared],
            status: TreasureFilled,
            infested: 0
        }
    ]
}

fn main() {
    println!("\n\n\n\n\n\n\n\n\n");
    println!("               *** The S P O O K Y ruins ***           \n");
    println!("Your job is to clear the ruins and leave with all the treasure.");
    println!("Good luck!");
    println!("\n\n");
    let mut rooms = get_rooms();
    let mut current_position = (0, 0);
    let mut finished_game = false;
    while rooms.contains_key(&current_position) && !finished_game {
        println!("{}", rooms.get(&current_position).unwrap().describe());
        update_rooms(&mut rooms);
        update_rooms(&mut rooms);
        let (new_position, outputs) = match get_command() {
            Go(dir) => go(&current_position, &rooms, &dir),
            Clear(dir) => clear(&current_position, &mut rooms, dir),
            Take => take(&current_position, &mut rooms),
            Leave => leave(&current_position, &rooms, &mut finished_game)
        };
        for s in outputs {
            println!("{}", s);
        }
        if let Some(room) = rooms.get(&current_position) {
            if room.is_deadly() {
                finished_game = true;
                println!("{}", rooms.get(&current_position).unwrap().describe());
            }
        }
        current_position = new_position;
    }
    if !rooms.contains_key(&current_position) {
        println!("Somehow, you have escaped into the void. You are safe... for now.");
    } else if rooms.get(&current_position).unwrap().is_deadly() {
        println!("You have fallen to the snakes. Game over!")
    } else {
        println!("You have escaped!");
    }
}