use super::{spawner, BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;

pub struct CorridorSpawner {}

impl MetaMapBuilder for CorridorSpawner {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CorridorSpawner {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(corridors) = &build_data.corridors {
            for c in corridors.iter() {
                spawner::spawn_region(rng, &c, build_data.map.depth, &mut build_data.spawn_list);
            }
        } else {
            panic!("Corridor Based Spawning only works after rooms have been created")
        }
    }
}
