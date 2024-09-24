use std::{fs::File, io::Read, path::Path};

use glam::Vec3;

use super::{obj_raw::face::VertexAttribute};

pub fn parse_vec3_or<'a>(words: &mut impl Iterator<Item = &'a str>, default: Option<Vec3>) -> Vec3 {
    let mut components: [f32; 3] = if let Some(default) = default {
        [default.x, default.y, default.z]
    } else {
        [0.0; 3]
    };

    for i in 0..3 {
        if let Some(value_str) = words.next() {
            components[i] = value_str.parse::<f32>().unwrap_or(components[i]);
        }
    }

    Vec3::new(components[0], components[1], components[2])
}

pub fn calculate_tri_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let edge_a = b - a;
    let edge_b = c - a;
    edge_a.cross(edge_b)
}
