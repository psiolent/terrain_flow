use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::default_flow::DefaultFlow;
use crate::default_shader::DefaultShader;
use crate::flow::FlowEngine;
use crate::point::Point;
use crate::point_gen::{Bounds, PointGenerator, PointsReader, PointsWriter};
use crate::render::Renderer;
use crate::terrain::Terrain;

pub struct Runner<'a> {
    width: usize,
    height: usize,
    density: u32,
    max_z: f64,

    flow_rate: f64,
    flow_erosion_rate: f64,
    erosion_threshold: f64,
    erosion_rate: f64,
    precipitation_rate: f64,
    precipitation_amount: f64,

    render_step: f64,
    frame_skip: u32,
    frame_count: u32,

    data_path: &'a str,
    render_path: &'a str,
}

pub struct RunnerBuilder<'a> {
    width: Option<usize>,
    height: Option<usize>,
    density: Option<u32>,
    max_z: Option<f64>,

    flow_rate: Option<f64>,
    flow_erosion_rate: Option<f64>,
    erosion_threshold: Option<f64>,
    erosion_rate: Option<f64>,
    precipitation_rate: Option<f64>,
    precipitation_amount: Option<f64>,

    render_step: Option<f64>,
    frame_skip: Option<u32>,
    frame_count: Option<u32>,

    data_path: Option<&'a str>,
    render_path: Option<&'a str>,
}

impl<'a> Runner<'a> {
    pub fn run(&mut self) {
        let points_file_path = format!(
            "{}/points_{}x{}x{}.dat",
            self.data_path,
            self.width,
            self.height,
            self.density,
        );

        if !Path::new(&points_file_path).exists() {
            let mut pw = PointsWriter::new(BufWriter::new(File::create(&points_file_path).unwrap()));
            println!("generating points");
            pw.write_points(PointGenerator::new(
                Bounds::new(0f64, self.width as f64),
                Bounds::new(0f64, self.height as f64),
                (self.density as f64).recip(),
            ));
        }

        println!("configuring flow engine");
        let height_at = |p: &Point| -> f64 {
            let x_term = -2.0 * p.x / self.width as f64 + 1.0;
            let y_term = -2.0 * p.y / self.height as f64 + 1.0;
            self.max_z * (-x_term * x_term + 1.0) * (-y_term * y_term + 1.0)
        };
        let depth_at = |p: &Point| -> f64 {
            let z = height_at(p);
            if z < 1.0 {
                1.0 - z
            } else {
                0.0
            }
        };
        let mut flow_engine = FlowEngine::new(
            Terrain::generate(
                PointsReader::new(BufReader::new(File::open(points_file_path).unwrap())),
                height_at,
                depth_at,
            ),
            DefaultFlow::new(
                self.flow_rate,
                self.flow_erosion_rate,
                self.erosion_threshold,
                self.erosion_rate,
                self.precipitation_rate,
                self.precipitation_amount,
            ),
        );

        let renderer = Renderer::new(
            self.width,
            self.height,
            DefaultShader {},
            self.render_path,
        );

        println!("rendering");

        for frame_num in 0..self.frame_count {
            println!("frame {} of {}", frame_num + 1, self.frame_count);
            renderer.render(flow_engine.terrain(), frame_num);
            for _ in 0..self.frame_skip {
                flow_engine.step(self.render_step);
            }
        }
    }
}

impl<'a> RunnerBuilder<'a> {
    pub fn new() -> RunnerBuilder<'a> {
        RunnerBuilder {
            width: None,
            height: None,
            density: None,
            max_z: None,
            flow_rate: None,
            flow_erosion_rate: None,
            erosion_threshold: None,
            erosion_rate: None,
            precipitation_rate: None,
            precipitation_amount: None,
            render_step: None,
            frame_skip: None,
            frame_count: None,
            data_path: None,
            render_path: None,
        }
    }

    pub fn width(&mut self, width: usize) -> &mut RunnerBuilder<'a> {
        assert!(width > 0);
        self.width = Some(width);
        self
    }

    pub fn height(&mut self, height: usize) -> &mut RunnerBuilder<'a> {
        assert!(height > 0);
        self.height = Some(height);
        self
    }

    pub fn density(&mut self, density: u32) -> &mut RunnerBuilder<'a> {
        assert!(density > 0);
        self.density = Some(density);
        self
    }

    pub fn max_z(&mut self, max_z: f64) -> &mut RunnerBuilder<'a> {
        assert!(max_z.is_finite());
        self.max_z = Some(max_z);
        self
    }

    pub fn flow_rate(&mut self, flow_rate: f64) -> &mut RunnerBuilder<'a> {
        assert!(flow_rate.is_finite());
        self.flow_rate = Some(flow_rate);
        self
    }

    pub fn flow_erosion_rate(&mut self, flow_erosion_rate: f64) -> &mut RunnerBuilder<'a> {
        assert!(flow_erosion_rate.is_finite());
        self.flow_erosion_rate = Some(flow_erosion_rate);
        self
    }

    pub fn erosion_threshold(&mut self, erosion_threshold: f64) -> &mut RunnerBuilder<'a> {
        assert!(erosion_threshold.is_finite());
        self.erosion_threshold = Some(erosion_threshold);
        self
    }

    pub fn erosion_rate(&mut self, erosion_rate: f64) -> &mut RunnerBuilder<'a> {
        assert!(erosion_rate.is_finite());
        self.erosion_rate = Some(erosion_rate);
        self
    }

    pub fn precipitation_rate(&mut self, precipitation_rate: f64) -> &mut RunnerBuilder<'a> {
        assert!(precipitation_rate.is_finite());
        assert!(precipitation_rate > 0_f64);
        assert!(precipitation_rate < 1_f64);
        self.precipitation_rate = Some(precipitation_rate);
        self
    }

    pub fn precipitation_amount(&mut self, precipitation_amount: f64) -> &mut RunnerBuilder<'a> {
        assert!(precipitation_amount.is_finite());
        self.precipitation_amount = Some(precipitation_amount);
        self
    }

    pub fn render_step(&mut self, render_step: f64) -> &mut RunnerBuilder<'a> {
        assert!(render_step.is_normal());
        assert!(render_step.is_sign_positive());
        self.render_step = Some(render_step);
        self
    }

    pub fn frame_skip(&mut self, frame_skip: u32) -> &mut RunnerBuilder<'a> {
        assert!(frame_skip > 0);
        self.frame_skip = Some(frame_skip);
        self
    }

    pub fn frame_count(&mut self, frame_count: u32) -> &mut RunnerBuilder<'a> {
        assert!(frame_count > 0);
        self.frame_count = Some(frame_count);
        self
    }

    pub fn data_path(&mut self, data_path: &'a str) -> &mut RunnerBuilder<'a> {
        assert!(Path::new(data_path).is_dir());
        self.data_path = Some(data_path);
        self
    }

    pub fn render_path(&mut self, render_path: &'a str) -> &mut RunnerBuilder<'a> {
        assert!(Path::new(render_path).is_dir());
        self.render_path = Some(render_path);
        self
    }

    pub fn build(&self) -> Runner {
        assert!(self.width.is_some());
        assert!(self.height.is_some());
        assert!(self.density.is_some());
        assert!(self.max_z.is_some());
        assert!(self.flow_rate.is_some());
        assert!(self.flow_erosion_rate.is_some());
        assert!(self.erosion_threshold.is_some());
        assert!(self.erosion_rate.is_some());
        assert!(self.precipitation_rate.is_some());
        assert!(self.precipitation_amount.is_some());
        assert!(self.render_step.is_some());
        assert!(self.frame_skip.is_some());
        assert!(self.frame_count.is_some());
        assert!(self.data_path.is_some());
        assert!(self.render_path.is_some());

        Runner {
            width: self.width.unwrap(),
            height: self.height.unwrap(),
            density: self.density.unwrap(),
            max_z: self.max_z.unwrap(),
            flow_rate: self.flow_rate.unwrap(),
            flow_erosion_rate: self.flow_erosion_rate.unwrap(),
            erosion_threshold: self.erosion_threshold.unwrap(),
            erosion_rate: self.erosion_rate.unwrap(),
            precipitation_rate: self.precipitation_rate.unwrap(),
            precipitation_amount: self.precipitation_amount.unwrap(),
            render_step: self.render_step.unwrap(),
            frame_skip: self.frame_skip.unwrap(),
            frame_count: self.frame_count.unwrap(),
            data_path: self.data_path.unwrap(),
            render_path: self.render_path.unwrap(),
        }
    }
}
