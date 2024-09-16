pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vertex {
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
            [Some(x), Some(y), Some(z), w] => Vertex {
                x,
                y,
                z,
                w: w.unwrap_or(1f32),
            },
            _ => panic!(),
        }
    }
}
