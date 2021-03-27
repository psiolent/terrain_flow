use crate::run::RunnerBuilder;

mod point;
mod point_gen;
mod terrain;
mod flow;
mod render;
mod run;
mod default_flow;
mod default_shader;

fn main() {
    let width = 1280_usize;
    let height = 720_usize;
    let density = 2_u32;
    let max_z = 36.0;

    let flow_rate = 0.9;
    let flow_erosion_rate = 1.0;
    let erosion_threshold = 0.2;
    let erosion_rate = 0.5;
    let precipitation_rate = 0.001;
    let precipitation_amount = 0.01;

    let render_step = 1.0;
    let frame_skip = 30;
    let frame_count = 20000;

    RunnerBuilder::new()
        .width(width)
        .height(height)
        .density(density)
        .max_z(max_z)
        .flow_rate(flow_rate)
        .flow_erosion_rate(flow_erosion_rate)
        .erosion_threshold(erosion_threshold)
        .erosion_rate(erosion_rate)
        .precipitation_rate(precipitation_rate)
        .precipitation_amount(precipitation_amount)
        .render_step(render_step)
        .frame_skip(frame_skip)
        .frame_count(frame_count)
        .data_path("./point_data")
        .render_path("./render")
        .build()
        .run();
}
