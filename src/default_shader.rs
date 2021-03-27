use crate::render::{RGB, Shade};
use crate::terrain::{Cell, Terrain};

pub struct DefaultShader;

impl Shade for DefaultShader {
    fn shade_cell(&self, cell: &Cell, terrain: &Terrain) -> RGB {
        if cell.depth() > 0.1 && cell.height() < 1.0 {
            let factor = (cell.depth() - 0.1) * 2.0 + 0.5;
            RGB {
                r: 0.2 / factor,
                g: 0.4 / factor,
                b: 1.0 / factor,
            }
        } else {
            let p_cell = [cell.x(), cell.y(), cell.height()];
            let mut v_light = [-1.0, 1.0, 1.0];
            vec3::norm_mut(&mut v_light);
            let mut lighting_sum = 0.0;
            let mut lighting_count = 0;
            for neighbor_dat in cell.neighbor_data_iter() {
                let neighbor = terrain.get_cell(neighbor_dat.index());
                let p_neighbor = [neighbor.x(), neighbor.y(), neighbor.height()];
                let mut v_neighbor = p_neighbor.clone();
                vec3::sub_mut(&mut v_neighbor, &p_cell);
                let mut v_normal = [v_neighbor[1], -v_neighbor[0], 0.0];
                vec3::cross_mut(&mut v_normal, &v_neighbor);
                vec3::norm_mut(&mut v_normal);
                lighting_sum += vec3::dot(&v_normal, &v_light);
                lighting_count += 1;
            }
            let lighting = 2.0 * lighting_sum / lighting_count as f64;
            RGB {
                r: 1.0 * lighting * lighting * lighting,
                g: 0.5 * lighting * lighting * lighting,
                b: 0.1 * lighting * lighting * lighting,
            }
        }
    }
}
