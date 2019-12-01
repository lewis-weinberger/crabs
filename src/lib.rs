pub mod levels;
use termion::event::Key;
use std::process;
use std::env::Args;
use std::fs::File;
use std::io::{Write, stdin};

// Number of game loops to wait in between updating crab positions
pub const RATE: usize = 10;
// Note: increase RATE to slow down the crabs!

pub fn process_args(mut args: Args) -> Vec<(Entities, Map)> {
    // Skip executable name
    args.next();

    match args.next() {
        Some(arg) => {
            match arg.as_str() {
                // Print help
                "--help" => {
                    println!("crabs --help");
                    println!("------------");
                    println!("Guide the crabs to safety:");
                    println!("\t. crab");
                    println!("\tX safety");
                    println!("Use the arrow keys to move the cursor:");
                    println!("\t+ cursor");
                    println!("Insert scenery by typing the appropriate key:");
                    println!("\t# block");
                    println!("\t/ forward ramp");
                    println!("\t\\ backward ramp");
                    println!("\t@ trampoline");
                    println!("Type q to quit level.\n");
                    println!("Use a custom map saved in RON format:");
                    println!("\t$ crabs custom_level.ron");
                    process::exit(0);
                },
                
                // Load custom level
                path => {
                    load_level(&path)
                },
            }
        },
        None => {
            // Load default level
            levels::default_levels()
        }
    }
}

fn load_level(path: &str) -> Vec<(Entities, Map)> {
    unimplemented!()
}

#[derive(Debug, Clone)]
pub struct Entities {
    collection: Vec<Crab>,
}

impl Entities {
    pub fn new(positions: Vec<[usize; 2]>, velocities: Vec<[isize; 2]>) -> Self {
        let mut collection: Vec<Crab> = Vec::new();
        for (position, velocity) in positions.iter().zip(velocities.iter()) {
            collection.push(Crab::new(*position, *velocity));
        }
        Entities { collection }
    }
    
    pub fn evolve(&mut self, map: &mut Map, complete: &mut bool) {
        // Add positions of crab to map
        map.instantaneous(&self);

        let mut remove: Vec<usize> = Vec::new();
        for (index, entity) in self.collection.iter_mut().enumerate() {
            // Advance each crab one after the other
            if entity.advance(map) {
                remove.push(index);
            }
        }

        // Remove any crabs that made it to safety
        for index in remove.iter() {
            self.collection.remove(*index);
        }

        // When all crabs are safe, game is complete
        if self.collection.len() == 0 {
            *complete = true;
        }
        
    }
}

#[derive(Debug, Clone)]
struct Crab {
    position: [usize; 2],
    velocity: [isize; 2],
}

impl Crab {
    fn new(position: [usize; 2], velocity: [isize; 2]) -> Self {
        Crab { position, velocity }
    }
    
    fn advance(&mut self, map: &mut Map) -> bool {
        // Remove previous position from map
        map.overide(&self.position, Scenery::Empty);
        
        // Evaluate x direction first (no diagonal motion!)
        let safe_x = self.advance_one_step_x(map,
                                             self.velocity[1].abs() as usize);

        // Evaluate y direction
        let safe_y = self.advance_one_step_y(map,
                                             self.velocity[0].abs() as usize);

        // Add new position to map
        map.update(&self.position, Scenery::StationaryCrab);

        // Did this crab make it to safety?
        safe_x || safe_y
    }

    fn advance_one_step_x(&mut self, map: &Map, steps: usize) -> bool {
        match steps {
            0 => false,
            n => {
                // Find next position along direction
                let mut next = self.position.clone();
                map.wrap(&mut next, [0, self.velocity[1]]);

                // Determine if obstacles are present
                match map.layout[next[0]][next[1]] {
                    Scenery::Empty => {
                        // Move into empty space
                        self.position = next;
                        self.advance_one_step_x(map, n - 1);
                        false
                    },
                    Scenery::ForwardWedge => {
                        // Advance up wedge
                        if self.velocity[1] > 0 {
                            let tmp = self.velocity[0];
                            self.velocity[0] = -1;
                            self.advance_one_step_y(map, 1);
                            self.velocity[0] = tmp;
                            self.advance_one_step_x(map, n);
                        } else {
                            // Rebound
                            self.velocity[1] *= -1;
                        }
                        false
                    },
                    Scenery::BackwardWedge => {
                        // Advance up wedge
                        if self.velocity[1] < 0 {
                            let tmp = self.velocity[0];
                            self.velocity[0] = -1;
                            self.advance_one_step_y(map, 1);
                            self.velocity[0] = tmp;
                            self.advance_one_step_x(map, n);
                        } else {
                            // Rebound
                            self.velocity[1] *= -1;
                        }
                        false
                    },
                    Scenery::Safety => {
                        // the crab made it to safety!
                        true
                    },
                    _ => {
                        // Rebound
                        self.velocity[1] *= -1;
                        false
                    },
                }
            }
        }
    }

    fn advance_one_step_y(&mut self, map: &Map, steps: usize) -> bool {
        match steps {
            0 => false,
            n => {
                // Find next position along direction
                let mut next = self.position.clone();
                map.wrap(&mut next, [self.velocity[0], 0]);

                // Determine if obstacles are present
                match map.layout[next[0]][next[1]] {
                    // Move into empty space
                    Scenery::Empty => {
                        self.position = next;
                        self.advance_one_step_y(map, n - 1);
                        false
                    },
                    Scenery::Trampoline => {
                        self.velocity[0] *= -1;
                        false
                    },
                    Scenery::Safety => {
                        true
                    },
                    _ => {
                        if self.velocity[0] < 0 {
                            // Rebound above
                            self.velocity[0] *= -1;
                        }
                        false
                    },
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    dimensions: [usize; 2],
    layout: Vec<Vec<Scenery>>,
    index: [usize; 2],
}

impl Map {
    pub fn new(cmap: &str) -> Self {
        // Determine size of layout
        let y_size = cmap.lines().count();
        let x_size = cmap.lines().nth(0).unwrap().chars().count();
        eprintln!("Level dimensions: (y, x) = ({}, {})", y_size, x_size);
        let dimensions = [y_size, x_size];

        // Allocate vector to store layout
        let mut layout = vec![vec![Scenery::Empty; x_size]; y_size];
        eprintln!("Map allocated: ({},{})", layout.len(), layout[0].len());

        // Fill in scenery
        for (yvec, line) in layout.iter_mut().zip(cmap.lines()) {
            for (cell, ch) in yvec.iter_mut().zip(line.chars()) {
                *cell = Scenery::new(ch);
            }
        }
        
        Map { dimensions, layout, index: [0, 0] }
    }

    fn instantaneous(&mut self, entities: &Entities) {
        // Fill in positions of crabs
        for entity in entities.collection.iter() {
            let y: usize = entity.position[0];
            let x: usize = entity.position[1];

            if self.layout[y][x] == Scenery::Empty {
                self.layout[y][x] = Scenery::StationaryCrab;
            } else {
                // Shouldn't ever reach here!
                panic!("Oh no, we've lost a crab!");
            }
        }
    }

    pub fn decrab(&mut self) {
        // Remove crabs from map
        for yvec in self.layout.iter_mut() {
            for cell in yvec.iter_mut() {
                if *cell == Scenery::StationaryCrab {
                    *cell = Scenery::Empty;
                }
            }
        }
    }

    pub fn update(&mut self, user: &[usize; 2], scenery: Scenery) {
        let [y, x] = *user;
        // Add new scenery at desired location (if empty)
        if self.layout[y][x] == Scenery::Empty {
            self.layout[y][x] = scenery;
        } else {
            eprintln!("Can't place {:?} as something is already there!", scenery);
        }
    }

    fn overide(&mut self, user: &[usize; 2], scenery: Scenery) {
        let [y, x] = *user;
        // Add new scenery at desired location
        self.layout[y][x] = scenery;
    }

    fn wrap(&self, user: &mut [usize; 2], change: [isize; 2]) {
        // Cast to signed integeter to avoid overflows
        let mut tmp_user = [user[0] as isize, user[1] as isize];
        let tmp_dims = [self.dimensions[0] as isize, self.dimensions[1] as isize];

        // Update coordinates with periodic boundaries
        for ((user_i, &dim_i), &change_i) in tmp_user.iter_mut().zip(tmp_dims.iter()).zip(change.iter()) {
            *user_i = if *user_i + change_i >= dim_i {
                // Wrap above
                *user_i + change_i - dim_i
            } else if *user_i + change_i < 0 {
                // Wrap below
                *user_i + change_i + dim_i
            } else {
                // Otherwise change as normal
                *user_i + change_i
            };
        }

        // Recast back to unsigned index
        *user = [tmp_user[0] as usize, tmp_user[1] as usize];
    }

    fn to_string(&self) -> String {
        let mut buffer = String::new();
        for (_, x, ch) in self.clone() {
            if x == self.dimensions[0] {
                buffer.push('\n');
            }
            buffer.push(ch);
        }
        buffer
    }
}

impl Iterator for Map {
    type Item = (usize, usize, char);

    fn next(&mut self) -> Option<(usize, usize, char)> {
        if self.index[0] + 1 < self.dimensions[0] {
            // Advance position in map
            if self.index[1] + 1 == self.dimensions[1] {
                self.index[0] += 1;
                self.index[1] = 0;
            } else {
                self.index[1] += 1;
            }

            // Find next character
            let ch = self.layout[self.index[0]][self.index[1]].to_char();
        
            Some((self.index[0], self.index[1], ch))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scenery {
    Empty,
    Block,
    ForwardWedge,
    BackwardWedge,
    Trampoline,
    Safety,
    StationaryCrab,
}

impl Scenery {
    pub fn new(scenery: char) -> Self {
        match scenery {
            '#' => Self::Block,
            '/' => Self::ForwardWedge,
            '\\' => Self::BackwardWedge,
            '@' => Self::Trampoline,
            'X' => Self::Safety,
            _ => Self::Empty,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Block => '#',
            Self::ForwardWedge => '/',
            Self::BackwardWedge => '\\',
            Self::Trampoline => '@',
            Self::Safety => 'X',
            Self::StationaryCrab => '.',
        }
    }
}

pub fn user_input(key: Key, user: &mut [usize; 2], complete: &mut bool, map: &mut Map) {
    match key {
        // Move cursor position
        Key::Left => {
            map.wrap(user, [0, -1]);
        }
        Key::Right => {
            map.wrap(user, [0, 1]);
        }
        Key::Up => {
            map.wrap(user, [-1, 0]);
        }
        Key::Down => {
            map.wrap(user, [1, 0]);
        }

        // Insert new scenery
        Key::Char('/') => {
            map.update(user, Scenery::ForwardWedge);
        }
        Key::Char('\\') => {
            map.update(user, Scenery::BackwardWedge);
        }
        Key::Char('#') => {
            map.update(user, Scenery::Block);
        }
        Key::Char('@') => {
            map.update(user, Scenery::Trampoline);
        }

        // Quit game
        Key::Char('q') => {
            *complete = true;
        }
        _ => ()
    }
}

pub fn check_resize(term_size: &mut (u16, u16)) -> bool {
    // Determine current terminal size
    match termion::terminal_size() {
        Ok(new) => {
            if *term_size == new {
                return false
            } else {
                *term_size = new;
                return true
            }
        }
        Err(err) => {
            eprintln!("Error determining terminal size: {:?}", err.kind());
            return true
        }
    }
}

pub fn prompt_for_filename() -> String {
    println!("Please enter a filename for saving:\n");
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Unable to read filename: {:?}", err.kind());
        }
    };
    println!("Saving map to: {}", buffer);
    // TODO: solve the bug here
    buffer
}

pub fn save_to_ron(filename: &str, map: &Map) {
    // Open file
    let mut file = match File::create(filename.trim()) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Unable to open save file: {:?}", err.kind());
            return ()
        },
    };

    // Output to file
    match write!(file, "// Custom level\n(\n\tpositions: [],\n\tvelocities: [],\n\tlayout: \"{}\",\n)\n", &map.to_string()) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Unable to write to save file: {:?}", err.kind());
            return ()
        },
    };
}
