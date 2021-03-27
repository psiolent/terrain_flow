use std::collections::HashMap;

use crossbeam;
use crossbeam::channel;
use rand::Rng;

use crate::flow::Flow;
use crate::terrain::{Cell, NeighborData, Terrain, TerrainDelta};

pub struct DefaultFlow {
    flow_rate: f64,
    flow_erosion_rate: f64,
    erosion_threshold: f64,
    erosion_rate: f64,
    precipitation_rate: f64,
    precipitation_amount: f64,
}

struct TransferWeight {
    weight: f64,
    available: f64,
}

impl DefaultFlow {
    pub fn new(
        flow_rate: f64,
        flow_erosion_rate: f64,
        erosion_threshold: f64,
        erosion_rate: f64,
        precipitation_rate: f64,
        precipitation_amount: f64,
    ) -> DefaultFlow {
        DefaultFlow {
            flow_rate,
            flow_erosion_rate,
            erosion_threshold,
            erosion_rate,
            precipitation_rate,
            precipitation_amount,
        }
    }
}

impl DefaultFlow {
    fn do_flow(&self, terrain: &Terrain) -> Vec<TerrainDelta> {
        let (tx_work, rx_work) = channel::bounded(1);
        let (tx_result, rx_result) = channel::bounded(1);

        crossbeam::scope(|s| {
            // send all cell indices as "work" units
            s.spawn(|_| {
                for cell_index in 0..terrain.cells_len() {
                    tx_work.send(cell_index).unwrap();
                }
                drop(tx_work);
            });

            // process each cell in parallel threads
            for _ in 0..num_cpus::get() {
                let (tx, rx) = (tx_result.clone(), rx_work.clone());
                s.spawn(move |_| {
                    for cell_index in rx.iter() {
                        let cell = terrain.get_cell(cell_index);
                        for delta in self.calc_flow_deltas(cell_index, cell, terrain) {
                            tx.send(delta).unwrap();
                        }
                        for delta in self.calc_sink_deltas(cell_index, cell) {
                            tx.send(delta).unwrap();
                        }
                    }
                });
            }

            drop(tx_result);

            rx_result.iter().collect()
        }).unwrap()
    }

    fn calc_sink_deltas(&self, cell_index: usize, cell: &Cell) -> Vec<TerrainDelta> {
        let mut height_delta = 0.0;
        let mut depth_delta = 0.0;
        if cell.height() < 0.0 {
            height_delta = -0.5 * cell.height();
        }
        if cell.depth() > 1.0 {
            depth_delta = -0.5 * (cell.depth() - 1.0);
        }
        vec!(TerrainDelta { cell_index, height_delta, depth_delta })
    }

    fn calc_flow_deltas(&self, cell_index: usize, cell: &Cell, terrain: &Terrain) -> Vec<TerrainDelta> {
        let flow_weights = self.calc_flow_weights(terrain, cell);
        let flow_agg = aggregate_transfer_weights(flow_weights.values());

        let erosion_weights = self.calc_erosion_weights(terrain, cell);
        let erosion_agg = aggregate_transfer_weights(erosion_weights.values());

        let mut self_delta: Option<TerrainDelta> = None;
        let mut neighbor_deltas: HashMap<usize, TerrainDelta> = HashMap::new();

        for (&neighbor_index, flow_weight) in flow_weights.iter() {
            let depth_delta = (flow_weight.weight / flow_agg.weight) * flow_agg.available * self.flow_rate;
            let height_delta = depth_delta * self.flow_erosion_rate;

            if depth_delta > 0.0 {
                let neighbor_delta = neighbor_deltas
                    .entry(neighbor_index)
                    .or_insert(TerrainDelta::new(neighbor_index));
                neighbor_delta.depth_delta += depth_delta;
                //neighbor_delta.height_delta += height_delta;

                let self_delta = self_delta
                    .get_or_insert(TerrainDelta::new(cell_index));
                self_delta.depth_delta -= depth_delta;
                self_delta.height_delta -= height_delta;
            }
        }

        for (&neighbor_index, erosion_weight) in erosion_weights.iter() {
            let height_delta = (erosion_weight.weight / erosion_agg.weight) * erosion_agg.available * self.erosion_rate;

            if height_delta > 0.0 {
                let neighbor_delta = neighbor_deltas
                    .entry(neighbor_index)
                    .or_insert(TerrainDelta::new(neighbor_index));
                neighbor_delta.height_delta += height_delta;

                let self_delta = self_delta
                    .get_or_insert(TerrainDelta::new(cell_index));
                self_delta.height_delta -= height_delta;
            }
        }

        if let Some(precipitation_amount) = self.calc_precipitation() {
            let self_delta = self_delta
                .get_or_insert(TerrainDelta::new(cell_index));
            self_delta.depth_delta += precipitation_amount;
        }

        let mut deltas: Vec<_> = neighbor_deltas
            .drain()
            .map(|(_, delta)| { delta })
            .collect();
        if let Some(self_delta) = self_delta {
            deltas.push(self_delta);
        }

        deltas
    }

    fn calc_flow_weights(&self, terrain: &Terrain, cell: &Cell) -> HashMap<usize, TransferWeight> {
        calc_transfer_weights_with(terrain, cell, |cell, neighbor, n_data| {
            self.calc_flow_weight(cell, neighbor, n_data.distance())
        })
    }

    fn calc_erosion_weights(&self, terrain: &Terrain, cell: &Cell) -> HashMap<usize, TransferWeight> {
        calc_transfer_weights_with(terrain, cell, |cell, neighbor, n_data| {
            self.calc_erosion_weight(cell, neighbor, n_data.distance())
        })
    }

    fn calc_flow_weight(&self, cell: &Cell, neighbor: &Cell, distance: f64) -> Option<TransferWeight> {
        let diff = (cell.height() + cell.depth()) - (neighbor.height() + neighbor.depth());
        let slope = diff / distance;
        let available = cell.depth().min(diff / 2.0);
        if slope > 0.0 {
            Some(TransferWeight { weight: slope * slope * slope, available })
        } else {
            None
        }
    }

    fn calc_erosion_weight(&self, cell: &Cell, neighbor: &Cell, distance: f64) -> Option<TransferWeight> {
        let diff = cell.height() - neighbor.height();
        let slope = diff / distance;
        let available = diff / 2.0;
        if slope > self.erosion_threshold {
            Some(TransferWeight { weight: slope, available })
        } else {
            None
        }
    }

    fn calc_precipitation(&self) -> Option<f64> {
        if rand::thread_rng().gen::<f64>() < self.precipitation_rate {
            Some(self.precipitation_amount)
        } else {
            None
        }
    }
}

impl Flow for DefaultFlow {
    fn flow(&self, terrain: &Terrain) -> Vec<TerrainDelta> {
        self.do_flow(terrain)
    }
}

fn calc_transfer_weights_with<F>(terrain: &Terrain, cell: &Cell, calc: F) -> HashMap<usize, TransferWeight>
    where
        F: Fn(&Cell, &Cell, &NeighborData) -> Option<TransferWeight>
{
    cell.neighbor_data_iter()
        .filter_map(|nd| {
            let neighbor = terrain.get_cell(nd.index());
            if let Some(transfer_weight) = calc(cell, neighbor, nd) {
                Some((nd.index(), transfer_weight))
            } else {
                None
            }
        })
        .collect()
}

fn aggregate_transfer_weights<'a>(iter: impl Iterator<Item=&'a TransferWeight>) -> TransferWeight {
    iter.fold(
        TransferWeight { weight: 0.0, available: f64::MAX },
        |acc: TransferWeight, cur: &TransferWeight| {
            TransferWeight {
                weight: acc.weight + cur.weight,
                available: acc.available.min(cur.available),
            }
        },
    )
}
