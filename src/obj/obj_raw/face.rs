use glam::Vec3;

pub struct Face {
    pub material_name: Option<String>,
    pub vertex_attributes: Vec<VertexAttribute>,
}
impl Face {
    pub fn parse(line: &str, material_name: Option<String>) -> Self {
        let mut words = line.split_whitespace();

        match words.next() {
            Some("f") => (),
            _ => panic!(),
        }

        let vertex_attributes: Vec<&str> = words.collect();
        if vertex_attributes.len() < 3 {
            panic!();
        }

        Self {
            vertex_attributes: vertex_attributes
                .iter()
                .map(|word| VertexAttribute::parse(word))
                .collect(),
            material_name,
        }
    }
}

pub struct VertexAttribute {
    pub vertex_index: u32,
    pub vertex_texture_index: Option<u32>,
    pub vertex_normal_index: Option<u32>,
}
impl VertexAttribute {
    fn vertex_attribute_parse(str: &str) -> Option<u32> {
        if str.is_empty() {
            None
        } else {
            Some(str.parse::<u32>().unwrap())
        }
    }

    pub fn parse(str: &str) -> Self {
        let raw_vertex_attributes: Vec<&str> = str.split('/').collect();
        if raw_vertex_attributes.len() > 3 {
            panic!("{:?} {}", &raw_vertex_attributes, str);
        }

        let mut vertex_attributes = raw_vertex_attributes
            .iter()
            .map(|&v| Self::vertex_attribute_parse(v))
            .collect::<Vec<Option<u32>>>();

        vertex_attributes.resize(3, None);

        match vertex_attributes[..] {
            [Some(vertex_index), vertex_texture_index, vertex_normal_index] => VertexAttribute {
                vertex_index: vertex_index - 1,
                vertex_texture_index: vertex_texture_index
                    .and_then(|vertex_texture_index| Some(vertex_texture_index - 1)),
                vertex_normal_index: vertex_normal_index
                    .and_then(|vertex_normal_index| Some(vertex_normal_index - 1)),
            },
            _ => panic!("{:?}", raw_vertex_attributes),
        }
    }
}
