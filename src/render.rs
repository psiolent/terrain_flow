use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::terrain::{Cell, Terrain};

pub struct Renderer<'a, S: Shade> {
    width: usize,
    height: usize,
    shader: S,
    render_path: &'a str,
}

pub trait Shade {
    fn shade_cell(&self, cell: &Cell, terrain: &Terrain) -> RGB;
}

struct Pixels {
    width: usize,
    height: usize,
    pixels: Vec<Pixel>,
}

#[derive(Clone)]
struct Pixel {
    total_rgb: RGB,
    total_weight: f64,
}

#[derive(Clone)]
pub struct RGB {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl<'a, S: Shade> Renderer<'a, S> {
    pub fn new(width: usize, height: usize, shader: S, render_path: &'a str) -> Renderer<'a, S> {
        Renderer { width, height, shader, render_path }
    }

    pub fn render(&self, terrain: &Terrain, frame_num: u32) {
        let mut pixels = Pixels::new(self.width, self.height);
        for cell in terrain.cells_iter() {
            pixels.add_color(cell.x(), cell.y(), &self.shader.shade_cell(cell, terrain));
        }
        self.save_image(frame_num, &pixels.to_data());
    }

    fn save_image(&self, frame_num: u32, pixel_data: &Vec<u8>) {
        let path_string = format!("{}/frame_{:06}.png", self.render_path, frame_num);
        let path = Path::new(&path_string);
        let file = File::create(path).unwrap();
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixel_data).unwrap();
    }
}

impl Pixels {
    fn new(width: usize, height: usize) -> Pixels {
        let mut pixels = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            pixels.push(Pixel {
                total_rgb: RGB { r: 0.0, g: 0.0, b: 0.0 },
                total_weight: 0.0,
            });
        }
        Pixels { width, height, pixels }
    }

    fn add_color(&mut self, x: f64, y: f64, color: &RGB) {
        let px0 = (x - 0.5).floor() as i32;
        let py0 = (y - 0.5).floor() as i32;
        for px in px0..px0 + 2 {
            for py in py0..py0 + 2 {
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let wx = 1.0 - (px as f64 + 0.5 - x).abs();
                    let wy = 1.0 - (py as f64 + 0.5 - y).abs();
                    let pw = wx * wy;
                    let index = (self.height - py as usize - 1) * self.width + px as usize;
                    self.pixels[index].total_rgb.r += color.r * pw;
                    self.pixels[index].total_rgb.g += color.g * pw;
                    self.pixels[index].total_rgb.b += color.b * pw;
                    self.pixels[index].total_weight += pw;
                }
            }
        }
    }

    fn to_data(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(self.width * self.height * 3);
        for pixel in self.pixels.iter() {
            data.extend_from_slice(&pixel.render().to_data());
        }
        data
    }
}

impl Pixel {
    fn render(&self) -> RGB {
        let mut c = self.total_rgb.clone();
        if self.total_weight > 0.0 {
            c.r /= self.total_weight;
            c.g /= self.total_weight;
            c.b /= self.total_weight;
        }
        c
    }
}

impl RGB {
    fn to_data(&self) -> [u8; 3] {
        [
            RGB::normalize(self.r),
            RGB::normalize(self.g),
            RGB::normalize(self.b)
        ]
    }

    fn normalize(n: f64) -> u8 {
        let mut n = (n * 256.0).floor();
        if n < 0.0 { n = 0.0; }
        if n > 255.0 { n = 255.0; }
        n as u8
    }
}
