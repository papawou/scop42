use glam::Vec4;

pub struct VertexPosition(Vec4);

impl VertexPosition {
    pub fn parse(line: &str) -> Self {
        let mut words = line.split_whitespace();

        match words.next() {
            Some("v") => (),
            _ => panic!(),
        }

        let raw_vertices: Vec<&str> = words.collect();
        if raw_vertices.len() < 3 && 4 < raw_vertices.len() {
            panic!();
        }

        let mut vertex = raw_vertices
            .iter()
            .map(|&v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.parse::<f32>().unwrap())
                }
            })
            .collect::<Vec<Option<f32>>>();
        vertex.resize(4, None);

        match vertex[..] {
            [Some(x), Some(y), Some(z), w] => Self(glam::vec4(x, y, z, w.unwrap_or(1f32))), // w?=1
            _ => panic!(),
        }
    }

    pub fn position(&self) -> Vec4 {
        self.0
    }
}
