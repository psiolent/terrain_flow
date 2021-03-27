use delaunator::{Point as DelPoint, triangulate};

use crate::point::Point;

pub struct Terrain {
    cells: Vec<Cell>,
}

pub struct Cell {
    location: Point,
    height: f64,
    depth: f64,
    neighbor_data: Vec<NeighborData>,
}

pub struct TerrainDelta {
    pub cell_index: usize,
    pub height_delta: f64,
    pub depth_delta: f64,
}

pub struct NeighborData {
    index: usize,
    distance: f64,
}

impl Terrain {
    pub fn generate(points: impl Iterator<Item=Point>, height_at: impl Fn(&Point) -> f64, depth_at: impl Fn(&Point) -> f64) -> Terrain {
        let mut cells: Vec<Cell> = points
            .map(|point| -> Cell {
                let height = height_at(&point);
                let depth = depth_at(&point);
                Cell::new(point, height, depth)
            })
            .collect();

        Terrain::calculate_neighbors(&mut cells);

        Terrain { cells }
    }

    pub fn apply_delta(&mut self, delta: &TerrainDelta) {
        let cell = &mut self.cells[delta.cell_index];
        cell.apply_delta(delta.height_delta, delta.depth_delta);
    }

    pub fn apply_deltas(&mut self, deltas: &[TerrainDelta]) {
        for delta in deltas {
            self.apply_delta(delta);
        }
    }

    pub fn cells_len(&self) -> usize {
        self.cells.len()
    }

    pub fn get_cell(&self, index: usize) -> &Cell {
        &self.cells[index]
    }

    pub fn cells_iter(&self) -> impl Iterator<Item=&Cell> {
        self.cells.iter()
    }

    fn calculate_neighbors(cells: &mut [Cell]) {
        let del_points: Vec<DelPoint> = cells.iter()
            .map(|point| -> DelPoint {
                DelPoint { x: point.x(), y: point.y() }
            })
            .collect();
        let triangulation = triangulate(&del_points).unwrap();
        for i in (0..triangulation.triangles.len()).step_by(3) {
            for cell_vertex in 0..3 {
                for neighbor_vertex in 0..3 {
                    if cell_vertex != neighbor_vertex {
                        let cell_index = triangulation.triangles[cell_vertex + i];
                        let neighbor_index = triangulation.triangles[neighbor_vertex + i];
                        let neighbor = &cells[neighbor_index];
                        let neighbor_location = Point {
                            x: neighbor.x(),
                            y: neighbor.y(),
                        };
                        cells[cell_index].add_neighbor(neighbor_index, &neighbor_location);
                    }
                }
            }
        }
    }
}

impl Cell {
    pub fn new(location: Point, height: f64, depth: f64) -> Cell {
        Cell {
            location,
            height,
            depth,
            neighbor_data: Vec::new(),
        }
    }

    pub fn x(&self) -> f64 {
        self.location.x
    }

    pub fn y(&self) -> f64 {
        self.location.y
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn depth(&self) -> f64 {
        self.depth
    }

    pub fn neighbor_data_iter(&self) -> impl Iterator<Item=&NeighborData> {
        self.neighbor_data.iter()
    }

    fn apply_delta(&mut self, height_delta: f64, level_delta: f64) {
        self.height += height_delta;
        self.depth += level_delta;
    }

    fn add_neighbor(&mut self, index: usize, neighbor_location: &Point) {
        if self.neighbor_data.iter().find(|&nd| -> bool { nd.index == index }).is_none() {
            let x_dist = self.x() - neighbor_location.x;
            let y_dist = self.y() - neighbor_location.y;
            let distance = (x_dist * x_dist + y_dist * y_dist).sqrt();
            self.neighbor_data.push(NeighborData { index, distance });
        }
    }
}

impl TerrainDelta {
    pub fn new(cell_index: usize) -> TerrainDelta {
        TerrainDelta {
            cell_index,
            height_delta: 0.0,
            depth_delta: 0.0,
        }
    }
}

impl NeighborData {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }
}