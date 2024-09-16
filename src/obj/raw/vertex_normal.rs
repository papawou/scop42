pub struct VertexNormal {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl VertexNormal {
    pub fn parse(line: &str) -> Self {
        let mut words = line.split_whitespace();

        match words.next() {
            Some("vt") => (),
            _ => panic!(),
        }

        let raw_vertices: Vec<&str> = words.collect();
        if raw_vertices.len() != 3 {
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
        vertex.resize(3, None);

        match vertex[..] {
            [Some(x), Some(y), Some(z)] => VertexNormal { x, y, z },
            _ => panic!(),
        }
    }
}
