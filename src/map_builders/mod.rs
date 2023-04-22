use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
mod area_starting_point;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod cull_unreachable;
mod distant_exit;
mod dla;
mod door_placement;
mod drunkard;
mod maze;
mod prefab_builder;
mod room_based_spawner;
mod room_based_stairs;
mod room_based_starting_position;
mod room_corner_rounding;
mod room_corridor_spawner;
mod room_draw;
mod room_exploder;
mod room_sorter;
mod rooms_corridors_bsp;
mod rooms_corridors_dogleg;
mod rooms_corridors_lines;
mod rooms_corridors_nearest;
mod simple_map;
mod voronoi;
mod voronoi_spawning;
mod waveform_collapse;
use area_starting_point::*;
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
pub use common::*;
use cull_unreachable::CullUnreachable;
use distant_exit::DistantExit;
use dla::DLABuilder;
use door_placement::DoorPlacement;
use drunkard::DrunkardsWalkBuilder;
use maze::MazeBuilder;
use prefab_builder::PrefabBuilder;
use rltk::RandomNumberGenerator;
use room_based_spawner::RoomBasedSpawner;
use room_based_stairs::RoomBasedStairs;
use room_based_starting_position::RoomBasedStartingPosition;
use room_corner_rounding::RoomCornerRounder;
use room_corridor_spawner::CorridorSpawner;
use room_draw::RoomDrawer;
use room_exploder::RoomExploder;
use room_sorter::*;
use rooms_corridors_bsp::BspCorridors;
use rooms_corridors_dogleg::DoglegCorridors;
use rooms_corridors_lines::StraightLineCorridors;
use rooms_corridors_nearest::NearestCorridors;
use simple_map::SimpleMapBuilder;
use specs::prelude::*;
use voronoi::VoronoiCellBuilder;
use voronoi_spawning::VoronoiSpawning;
use waveform_collapse::WaveformCollapseBuilder;

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub width: i32,
    pub height: i32,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(new_depth: i32, width: i32, height: i32) -> Self {
        Self {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width,
                height,
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder"),
        }
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => starter.build_map(rng, &mut self.build_data), // Build the starting map
        }

        // Build the additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

fn random_start_position(rng: &mut RandomNumberGenerator) -> (XStart, YStart) {
    let x = match rng.roll_dice(1, 3) {
        1 => XStart::LEFT,
        2 => XStart::CENTER,
        _ => XStart::RIGHT,
    };

    let y = match rng.roll_dice(1, 3) {
        1 => YStart::TOP,
        2 => YStart::CENTER,
        _ => YStart::BOTTOM,
    };

    (x, y)
}

fn random_room_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    let starter: Box<dyn InitialMapBuilder> = match build_roll {
        1 => SimpleMapBuilder::new(),
        2 => BspDungeonBuilder::new(),
        _ => BspInteriorBuilder::new(),
    };
    builder.start_with(starter);

    // BSP Interior still makes holes in the walls
    if build_roll != 3 {
        // Sort by one of the 5 available algorithms
        let sort = match rng.roll_dice(1, 5) {
            1 => RoomSort::LEFTMOST,
            2 => RoomSort::RIGHTMOST,
            3 => RoomSort::TOPMOST,
            4 => RoomSort::BOTTOMMOST,
            _ => RoomSort::CENTRAL,
        };
        builder.with(RoomSorter::new(sort));

        let corridors: Box<dyn MetaMapBuilder> = match rng.roll_dice(1, 4) {
            1 => DoglegCorridors::new(),
            2 => NearestCorridors::new(),
            3 => StraightLineCorridors::new(),
            _ => BspCorridors::new(),
        };
        builder.with(corridors);

        if rng.roll_dice(1, 2) == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => (),
        };
    }

    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    builder.with(RoomDrawer::new());

    let exit_roll = rng.roll_dice(1, 2);
    match exit_roll {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new()),
    }

    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new()),
    }
}

fn random_shape_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    let starter: Box<dyn InitialMapBuilder> = match rng.roll_dice(1, 16) {
        1 => CellularAutomataBuilder::new(),
        2 => DrunkardsWalkBuilder::open_area(),
        3 => DrunkardsWalkBuilder::open_halls(),
        4 => DrunkardsWalkBuilder::winding_passages(),
        5 => DrunkardsWalkBuilder::fat_passages(),
        6 => DrunkardsWalkBuilder::fearful_symmetry(),
        7 => MazeBuilder::new(),
        8 => DLABuilder::walk_inwards(),
        9 => DLABuilder::walk_outwards(),
        10 => DLABuilder::central_attractor(),
        11 => DLABuilder::insectoid(),
        12 => DLABuilder::heavy_erosion(),
        13 => VoronoiCellBuilder::pythagoras(),
        14 => VoronoiCellBuilder::manhattan(),
        16 => VoronoiCellBuilder::chebyshev(),
        _ => PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED),
    };
    builder.start_with(starter);

    // Set the start to the center and cull
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());

    // Now set the start to a random starting area
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // Setup an exit and spawn mobs
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}

pub fn random_builder(
    new_depth: i32,
    rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height);
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder),
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());

        // Now set the start to a random starting area
        let (start_x, start_y) = random_start_position(rng);
        builder.with(AreaStartingPosition::new(start_x, start_y));

        // Setup an exit and spawn mobs
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(
            prefab_builder::prefab_sections::UNDERGROUND_FORT,
        ));
    }

    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());

    builder
}
