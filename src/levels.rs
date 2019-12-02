use crate::{Entities, Map};

/// Load in the default levels
pub fn default_levels() -> Vec<(Entities, Map)> {
    let mut default: Vec<(Entities, Map)> = Vec::new();
    let levels = [level_1, level_2, level_3];
    for level in levels.iter() {
        default.push(level())
    }
    default
}

/// Default level 1
pub fn level_1() -> (Entities, Map) {
    let layout: &str = "                                                                                \n                                                                                \n                         #                                                      \n########################### ####################################################\n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n               #        X           #                                           \n################################################################################\n                                                                                \n";
    let positions: Vec<[usize; 2]> = vec![[2, 1], [2, 10]];
    let velocities: Vec<[isize; 2]> = vec![[1, 1], [1, -1]];
    let crabs = Entities::new(positions, velocities);
    let map = Map::new(layout);

    (crabs, map)
}

/// Default level 2
pub fn level_2() -> (Entities, Map) {
    let layout: &str = "                                                                                \n                                                                                \n                         #                                                      \n########################### ####################################################\n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n               #        X           #                                           \n################################################################################\n                                                                                \n";
    let positions: Vec<[usize; 2]> = vec![[1, 1]];
    let velocities: Vec<[isize; 2]> = vec![[1, 1]];
    let crabs = Entities::new(positions, velocities);
    let map = Map::new(layout);

    (crabs, map)
}

/// Default level 2
pub fn level_3() -> (Entities, Map) {
    let layout: &str = "                                                                                \n                                                                                \n                         #                                                      \n########################### ####################################################\n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n               #        X           #                                           \n################################################################################\n                                                                                \n";
    let positions: Vec<[usize; 2]> = vec![[1, 1]];
    let velocities: Vec<[isize; 2]> = vec![[1, 1]];
    let crabs = Entities::new(positions, velocities);
    let map = Map::new(layout);

    (crabs, map)
}

/// Blank map
pub fn blank_map() -> Map {
    let layout: &str = "                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n                                                                                \n";
    let map = Map::new(layout);

    map
}
