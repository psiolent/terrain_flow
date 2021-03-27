use std::f64::consts::TAU;
use std::io::{Read, Write};

use kdtree::{distance, KdTree};
use rand::Rng;

use crate::point::Point;

const GEN_SEARCH_STEP_COUNT: i32 = 100;

pub struct PointGenerator {
    x_bounds: Bounds,
    y_bounds: Bounds,
    min_spacing: f64,
    min_spacing_sq: f64,
    point_queue: Vec<Point>,
    kd_tree: KdTree<f64, (), [f64; 2]>,
    init: bool,
}

pub struct PointsWriter<W: Write> {
    writer: W,
}

pub struct PointsReader<R: Read> {
    reader: R,
}

pub struct Bounds {
    min_inc: f64,
    max_exc: f64,
}

impl PointGenerator {
    pub fn new(x_bounds: Bounds, y_bounds: Bounds, min_spacing: f64) -> PointGenerator {
        let point_queue = Vec::new();
        let kd_tree = KdTree::new(2);
        let min_spacing_sq = min_spacing * min_spacing;

        PointGenerator { x_bounds, y_bounds, min_spacing, min_spacing_sq, point_queue, kd_tree, init: false }
    }

    fn next_point(&mut self) -> Option<Point> {
        if !self.init {
            Some(self.gen_init_point())
        } else {
            self.gen_next_point()
        }
    }

    fn gen_init_point(&mut self) -> Point {
        assert_eq!(self.init, false);

        self.init = true;

        let init_point = Point {
            x: (self.x_bounds.max_exc - self.x_bounds.min_inc) / 2f64,
            y: (self.y_bounds.max_exc - self.y_bounds.min_inc) / 2f64,
        };

        self.point_queue.push(init_point.clone());
        self.kd_tree.add([init_point.x, init_point.y], ()).unwrap();

        init_point
    }

    fn gen_next_point(&mut self) -> Option<Point> {
        let mut point = None;

        while point.is_none() && !self.point_queue.is_empty() {
            let anchor_point = self.point_queue.pop().unwrap();

            if let Some(neighbor_point) = self.gen_neighbor_point(&anchor_point) {
                self.point_queue.push(anchor_point);
                self.point_queue.push(neighbor_point.clone());
                self.kd_tree.add([neighbor_point.x, neighbor_point.y], ()).unwrap();
                point = Some(neighbor_point);
            }
        }

        point
    }

    fn gen_neighbor_point(&self, anchor_point: &Point) -> Option<Point> {
        let base_angle = rand::thread_rng().gen::<f64>() * TAU;
        let step_dir = if rand::thread_rng().gen::<bool>() { 1.0 } else { -1.0 };
        let step_delta_angle = step_dir * TAU / (GEN_SEARCH_STEP_COUNT as f64);

        let mut step_num = 0;
        let mut point = None;
        while point.is_none() && step_num < GEN_SEARCH_STEP_COUNT {
            let delta_x = (base_angle + step_delta_angle * step_num as f64).cos() * self.min_spacing;
            let delta_y = (base_angle + step_delta_angle * step_num as f64).sin() * self.min_spacing;

            let candidate_point = Point {
                x: anchor_point.x + delta_x,
                y: anchor_point.y + delta_y,
            };

            if self.point_in_bounds(&candidate_point) && self.point_has_space(&candidate_point) {
                point = Some(candidate_point);
            }

            step_num += 1;
        }

        point
    }

    fn point_in_bounds(&self, point: &Point) -> bool {
        self.x_bounds.contains(point.x) && self.y_bounds.contains(point.y)
    }

    fn point_has_space(&self, point: &Point) -> bool {
        self.kd_tree.within(
            &[point.x, point.y],
            self.min_spacing_sq,
            &distance::squared_euclidean,
        ).unwrap().is_empty()
    }
}

impl Iterator for PointGenerator {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        self.next_point()
    }
}

impl<W: Write> PointsWriter<W> {
    pub fn new(writer: W) -> PointsWriter<W> {
        PointsWriter { writer }
    }

    pub fn write_points(&mut self, points: impl Iterator<Item=Point>) {
        for point in points {
            self.writer.write(&point.x.to_le_bytes()).unwrap();
            self.writer.write(&point.y.to_le_bytes()).unwrap();
        }
    }
}

impl<R: Read> PointsReader<R> {
    pub fn new(reader: R) -> PointsReader<R> {
        PointsReader { reader }
    }
}

impl<R: Read> Iterator for PointsReader<R> {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        let mut x_buffer = [0; 8];
        let mut y_buffer = [0; 8];
        let x_num = self.reader.read(&mut x_buffer).unwrap();
        let y_num = self.reader.read(&mut y_buffer).unwrap();
        if x_num == 8 && y_num == 8 {
            Some(Point {
                x: f64::from_le_bytes(x_buffer),
                y: f64::from_le_bytes(y_buffer),
            })
        } else {
            None
        }
    }
}

impl Bounds {
    pub fn new(min_inc: f64, max_exc: f64) -> Bounds {
        assert!(min_inc < max_exc);
        Bounds { min_inc, max_exc }
    }

    pub fn contains(&self, val: f64) -> bool {
        self.min_inc <= val && val < self.max_exc
    }
}
