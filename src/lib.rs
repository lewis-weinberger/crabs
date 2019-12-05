pub mod levels;

use std::cmp;
use std::collections::HashMap;
use std::env::Args;
use std::fs::File;
use std::io::{stdin, Write};
use std::{process, time};

use ron::de::from_reader;
use serde::Deserialize;
use termion::color;
use termion::event::Key;

// Target tick time which will be the minimum period between iterations of the game loop.
pub const TICK_TIME: time::Duration = time::Duration::from_millis(100);

// Terminal velocity
pub const VMAX: isize = 10;

pub fn process_args(mut args: Args, rate: &mut time::Duration) -> Vec<(Entities, Map)> {
    // Skip executable name
    args.next();

    match args.next() {
        Some(arg) => {
            match arg.as_str() {
                // Print help
                "--help" => {
                    println!("\ncrabs --help");
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
                    println!("Each level, type r to restart and q to quit.\n");
                    println!("Use a custom map saved in RON format:");
                    println!("\t$ crabs custom_level.ron");
                    println!("To adjust the crab speed:");
                    println!("\t$ crabs --tick-time N");
                    println!("where larger N makes the crabs slower! Default is 100ms\n");
                    process::exit(0);
                }

                // User adjusted rate
                "--tick-time" => match args.next() {
                    Some(new_rate) => {
                        *rate = match new_rate.parse::<usize>() {
                            Ok(r) => {
                                eprintln!("Using adjusted rate: {}", new_rate);
                                time::Duration::from_millis(r as u64)
                            }
                            Err(_) => {
                                eprintln!("{} not a valid rate!", new_rate);
                                TICK_TIME
                            }
                        };
                        levels::default_levels()
                    }
                    None => {
                        eprintln!("No rate provided...");
                        levels::default_levels()
                    }
                },

                // Load custom level
                path => load_level(&path),
            }
        }
        None => {
            // Load default level
            levels::default_levels()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoadedLevel {
    x: HashMap<u16, u16>,
    y: HashMap<u16, u16>,
    vx: HashMap<i16, i16>,
    vy: HashMap<i16, i16>,
    layout: String,
}

fn load_level(path: &str) -> Vec<(Entities, Map)> {
    match File::open(path) {
        Ok(file) => {
            // Decode RON format of configuration file
            let loaded: LoadedLevel = match from_reader(file) {
                Ok(x) => x,
                Err(err) => {
                    eprintln!("Unable to read custom level: {:?}", err);
                    return levels::default_levels();
                }
            };
            let entities = Entities::new(
                convert_to_vec(loaded.y, loaded.x),
                convert_to_vec(loaded.vy, loaded.vx),
            );
            let map = Map::new(&loaded.layout);
            vec![(entities, map)]
        }
        Err(err) => {
            eprintln!("Unable to read custom level file: {:?}", err.kind());
            levels::default_levels()
        }
    }
}

fn convert_to_vec<T: Copy, U: From<T>>(x: HashMap<T, T>, y: HashMap<T, T>) -> Vec<[U; 2]> {
    let mut store: Vec<[U; 2]> = Vec::new();
    for ((_, xi), (_, yi)) in x.iter().zip(y.iter()) {
        store.push([U::from(*xi), U::from(*yi)]);
    }
    eprintln!("Loaded {} crabs.", store.len());
    store
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
        let safe_x = self.advance_one_step_x(map, cmp::max(1, self.velocity[1].abs() as usize));

        // Evaluate y direction
        let safe_y = self.advance_one_step_y(map, cmp::max(1, self.velocity[0].abs() as usize));

        // Add new position to map
        map.update(&self.position, Scenery::StationaryCrab);

        // Did this crab make it to safety?
        safe_x || safe_y
    }

    fn advance_one_step_x(&mut self, map: &Map, steps: usize) -> bool {
        match steps {
            0 => (),
            n => {
                // Find next position along direction
                let mut next = self.position.clone();
                map.wrap(&mut next, [0, self.velocity[1].signum()]);

                // Determine if obstacles are present
                match map.layout[next[0]][next[1]] {
                    Scenery::Empty => {
                        // Move into empty space
                        self.position = next;
                        self.advance_one_step_x(map, n - 1);
                    }
                    Scenery::ForwardWedge => {
                        // Advance up wedge
                        if self.velocity[1] > 0 {
                            let tmp_vel = self.velocity[0];
                            let tmp_pos = self.position[0];
                            self.velocity[0] = -2; // overcome gravity
                            self.advance_one_step_y(map, 1);
                            self.velocity[0] = tmp_vel;
                            if self.position[0] == tmp_pos {
                                // Rebound
                                self.velocity[1] *= -1;
                            } else {
                                self.advance_one_step_x(map, n);
                            }
                        } else {
                            // Rebound
                            self.velocity[1] *= -1;
                        }
                    }
                    Scenery::BackwardWedge => {
                        // Advance up wedge
                        if self.velocity[1] < 0 {
                            let tmp_vel = self.velocity[0];
                            let tmp_pos = self.position[0];
                            self.velocity[0] = -2; // overcome gravity
                            self.advance_one_step_y(map, 1);
                            self.velocity[0] = tmp_vel;
                            if self.position[0] == tmp_pos {
                                // Rebound
                                self.velocity[1] *= -1;
                            } else {
                                self.advance_one_step_x(map, n);
                            }
                        } else {
                            // Rebound
                            self.velocity[1] *= -1;
                        }
                    }
                    //                    Scenery::ForwardBoost => {
                    //                        // Speed boost forward
                    //                        if self.velocity[1].abs() < VMAX {
                    //                            self.velocity[1] += 1;
                    //                        }
                    //                        self.position = next;
                    //                        self.advance_one_step_x(map, n - 1);
                    //                    }
                    //                    Scenery::BackwardBoost => {
                    //                        // Speed boost backward
                    //                        if self.velocity[1].abs() < VMAX {
                    //                            self.velocity[1] -= 1;
                    //                        }
                    //                        self.position = next;
                    //                        self.advance_one_step_x(map, n - 1);
                    //                    }
                    Scenery::Safety => {
                        // the crab made it to safety!
                        return true;
                    }
                    _ => {
                        // Rebound
                        self.velocity[1] *= -1;
                    }
                }
            }
        }
        false
    }

    fn advance_one_step_y(&mut self, map: &Map, steps: usize) -> bool {
        match steps {
            0 => (),
            n => {
                // Acceleration due to gravity
                if self.velocity[0] < VMAX {
                    self.velocity[0] += 1;
                }

                // Find next position along direction
                let mut next = self.position.clone();
                map.wrap(&mut next, [self.velocity[0].signum(), 0]);

                // Determine if obstacles are present
                match map.layout[next[0]][next[1]] {
                    // Move into empty space
                    Scenery::Empty => {
                        self.position = next;
                        self.advance_one_step_y(map, n - 1);
                    }
                    Scenery::Trampoline => {
                        self.velocity[0] = -VMAX;
                    }
                    Scenery::Safety => return true,
                    _ => {
                        if self.velocity[0] < 0 {
                            // Rebound above
                            self.velocity[0] *= -1;
                        } else {
                            // Stop below
                            self.velocity[0] = 0;
                        }
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub dimensions: [usize; 2],
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

        Map {
            dimensions,
            layout,
            index: [0, 0],
        }
    }

    pub fn instantaneous(&mut self, entities: &Entities) {
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
        for ((user_i, &dim_i), &change_i) in
            tmp_user.iter_mut().zip(tmp_dims.iter()).zip(change.iter())
        {
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
            buffer.push(ch);
            if x == self.dimensions[1] - 1 {
                buffer.push('\\');
                buffer.push('n');
            }
        }
        buffer
    }
}

impl Iterator for Map {
    type Item = (usize, usize, char);

    fn next(&mut self) -> Option<(usize, usize, char)> {
        if (self.index[1]*self.dimensions[0] + self.index[0]) < self.dimensions[0] * self.dimensions[1] {
            // Current position
            let y = self.index[0];
            let x = self.index[1];

            // Current character
            let ch = self.layout[y][x].to_char();

            // Advance position in map
            if self.index[1] + 1 == self.dimensions[1] {
                if self.index[0] + 1 < self.dimensions[0] {
                    self.index[0] += 1;
                    self.index[1] = 0;
                } else {
                    self.index = self.dimensions;
                }
            } else {
                self.index[1] += 1;
            }

            Some((y, x, ch))
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
    ForwardBoost,
    BackwardBoost,
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
            '>' => Self::ForwardBoost,
            '<' => Self::BackwardBoost,
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
            Self::ForwardBoost => '>',
            Self::BackwardBoost => '<',
            Self::Trampoline => '@',
            Self::Safety => 'X',
            Self::StationaryCrab => '.',
        }
    }
}

pub trait Colour {
    fn to_fg_colour(&self) -> String;
}

impl Colour for char {
    fn to_fg_colour(&self) -> String {
        match self {
            ' ' => format!("{}", color::Fg(color::Reset)),
            '#' => format!("{}", color::Fg(color::Red)),
            '/' => format!("{}", color::Fg(color::Yellow)),
            '\\' => format!("{}", color::Fg(color::Yellow)),
            '<' => format!("{}", color::Fg(color::Yellow)),
            '>' => format!("{}", color::Fg(color::Yellow)),
            '@' => format!("{}", color::Fg(color::Cyan)),
            'X' => format!("{}", color::Fg(color::Reset)),
            '.' => format!("{}", color::Fg(color::Reset)),
            _ => format!("{}", color::Fg(color::Reset)),
        }
    }
}

pub fn user_input(
    key: Key,
    user: &mut [usize; 2],
    complete: &mut bool,
    reset: &mut bool,
    map: &mut Map,
) {
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
        Key::Char('>') => {
            map.update(user, Scenery::ForwardBoost);
        }
        Key::Char('<') => {
            map.update(user, Scenery::BackwardBoost);
        }

        // Quit level
        Key::Char('q') => {
            *complete = true;
        }
        // Reset level
        Key::Char('r') => {
            *reset = true;
        }

        _ => (),
    }
}

pub fn check_resize(term_size: &mut (u16, u16)) -> bool {
    // Determine current terminal size
    match termion::terminal_size() {
        Ok(new) => {
            if *term_size == new {
                return false;
            } else {
                *term_size = new;
                return true;
            }
        }
        Err(err) => {
            eprintln!("Error determining terminal size: {:?}", err.kind());
            return true;
        }
    }
}

pub fn prompt_for_filename() -> Result<String, std::io::Error> {
    println!("\nPlease enter a filename for saving:\n");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    println!("Saving map to: {}\n", buffer);
    Ok(buffer)
}

pub fn prompt_for_positions() -> Result<Vec<[usize; 2]>, std::io::Error> {
    println!("\nPlease enter crab positions");
    println!("First provide a list of x coordinates (between 0 and 79):\n");
    let mut xbuffer = String::new();
    stdin().read_line(&mut xbuffer)?;

    println!("\nSecond provide a list of y coordinates (between 0 and 79):\n");
    let mut ybuffer = String::new();
    stdin().read_line(&mut ybuffer)?;

    eprintln!("\nYou have provided the following crab positions [y,x]:");
    let mut positions: Vec<[usize; 2]> = Vec::new();
    for (xi, yi) in xbuffer.chars().zip(ybuffer.chars()) {
        xi.to_digit(10).and_then(|xi| {
            yi.to_digit(10).map(|yi| {
                eprintln!("[{}, {}]", yi, xi);
                positions.push([yi as usize, xi as usize])
            })
        });
    }

    Ok(positions)
}

pub fn prompt_for_velocities() -> Result<Vec<[isize; 2]>, std::io::Error> {
    println!("\nPlease enter crab positions");
    println!("First provide a list of x coordinates (between 0 and 79):\n");
    let mut xbuffer = String::new();
    stdin().read_line(&mut xbuffer)?;

    println!("\nSecond provide a list of y coordinates (between 0 and 79):\n");
    let mut ybuffer = String::new();
    stdin().read_line(&mut ybuffer)?;

    eprintln!("\nYou have provided the following crab positions [y,x]:");
    let mut velocities: Vec<[isize; 2]> = Vec::new();
    for (xi, yi) in xbuffer.chars().zip(ybuffer.chars()) {
        // Note this breaks down for negative velocities!
        xi.to_digit(10).and_then(|xi| {
            yi.to_digit(10).map(|yi| {
                eprintln!("[{}, {}]", yi, xi);
                velocities.push([yi as isize, xi as isize])
            })
        });
    }

    Ok(velocities)
}

pub fn save_to_ron(
    filename: &str,
    map: &Map,
    positions: Vec<[usize; 2]>,
    velocities: Vec<[isize; 2]>,
) -> Result<(), std::io::Error> {
    // Open file
    let mut file = File::create(filename.trim())?;

    // Convert to appropriate format
    let mut x = String::from("{ ");
    let mut y = String::from("{ ");
    let mut vx = String::from("{ ");
    let mut vy = String::from("{ ");
    for (i, ([yi, xi], [vyi, vxi])) in positions.iter().zip(velocities.iter()).enumerate() {
        x.push_str(&format!("{}:{}, ", i, xi));
        y.push_str(&format!("{}:{}, ", i, yi));
        vx.push_str(&format!("{}:{}, ", i, vxi));
        vy.push_str(&format!("{}:{}, ", i, vyi));
    }
    x.push('}');
    y.push('}');
    vx.push('}');
    vy.push('}');

    // Output to file
    write!(
        file,
        "// Custom level\n(\n\tx: {},\n\ty: {},\n\tvx: {},\n\tvy: {},\n\tlayout: \"{}\",\n)\n",
        x,
        y,
        vx,
        vy,
        &map.to_string()
    )?;

    Ok(())
}
