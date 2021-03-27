use crate::terrain::{Terrain, TerrainDelta};

pub struct FlowEngine<S: Flow> {
    terrain: Terrain,
    strategy: S,
}

impl<S: Flow> FlowEngine<S> {
    pub fn new(terrain: Terrain, strategy: S) -> FlowEngine<S> {
        FlowEngine { terrain, strategy }
    }

    pub fn step(&mut self, time_delta: f64) {
        let mut deltas = self.strategy.flow(&self.terrain);
        for delta in deltas.iter_mut() {
            delta.height_delta *= time_delta;
            delta.depth_delta *= time_delta;
        }
        self.terrain.apply_deltas(&deltas);
    }

    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }
}

pub trait Flow {
    fn flow(&self, terrain: &Terrain) -> Vec<TerrainDelta>;
}
