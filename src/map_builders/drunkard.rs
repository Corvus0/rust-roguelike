use super::{common::*, BuilderMap, InitialMapBuilder, MetaMapBuilder, Position, TileType};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Clone, Copy)]
enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub drunken_lifetime: i32,
    pub floor_percent: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry,
}

pub struct DrunkardsWalkBuilder {
    settings: DrunkardSettings,
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl MetaMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DrunkardsWalkBuilder {
    fn new(settings: DrunkardSettings) -> Self {
        Self { settings }
    }

    pub fn open_area() -> Box<Self> {
        Box::new(Self::new(DrunkardSettings {
            spawn_mode: DrunkSpawnMode::StartingPoint,
            drunken_lifetime: 400,
            floor_percent: 0.5,
            brush_size: 1,
            symmetry: Symmetry::None,
        }))
    }

    pub fn open_halls() -> Box<Self> {
        Box::new(Self::new(DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            drunken_lifetime: 400,
            floor_percent: 0.5,
            brush_size: 1,
            symmetry: Symmetry::None,
        }))
    }

    pub fn winding_passages() -> Box<Self> {
        Box::new(Self::new(DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            drunken_lifetime: 100,
            floor_percent: 0.4,
            brush_size: 1,
            symmetry: Symmetry::None,
        }))
    }

    pub fn fat_passages() -> Box<Self> {
        Box::new(Self::new(DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            drunken_lifetime: 100,
            floor_percent: 0.4,
            brush_size: 2,
            symmetry: Symmetry::None,
        }))
    }

    pub fn fearful_symmetry() -> Box<Self> {
        Box::new(Self::new(DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            drunken_lifetime: 100,
            floor_percent: 0.4,
            brush_size: 1,
            symmetry: Symmetry::Both,
        }))
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Set a central starting point
        let starting_position = Position {
            x: build_data.map.width / 2,
            y: build_data.map.height / 2,
        };
        let start_idx = build_data
            .map
            .xy_idx(starting_position.x, starting_position.y);
        build_data.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut first_digger = true;

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_x;
            let mut drunk_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = starting_position.x;
                    drunk_y = starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if first_digger {
                        drunk_x = starting_position.x;
                        drunk_y = starting_position.y;
                        first_digger = false;
                    } else {
                        drunk_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    }
                }
            }
            let drunk_life = self.settings.drunken_lifetime;

            for _ in 0..drunk_life {
                let drunk_idx = build_data.map.xy_idx(drunk_x, drunk_y);
                if build_data.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(
                    &mut build_data.map,
                    self.settings.symmetry,
                    self.settings.brush_size,
                    drunk_x,
                    drunk_y,
                );
                build_data.map.tiles[drunk_idx] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => {
                        if drunk_x > 2 {
                            drunk_x -= 1;
                        }
                    }
                    2 => {
                        if drunk_x < build_data.map.width - 2 {
                            drunk_x += 1;
                        }
                    }
                    3 => {
                        if drunk_y > 2 {
                            drunk_y -= 1;
                        }
                    }
                    _ => {
                        if drunk_y < build_data.map.height - 2 {
                            drunk_y += 1;
                        }
                    }
                }
            }
            if did_something {
                build_data.take_snapshot();
            }

            for t in build_data.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = build_data
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }
    }
}
