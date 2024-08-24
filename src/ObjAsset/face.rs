pub struct Face {
    vertex_attributes: Vec<VertexAttribute>,
}

struct VertexAttribute {
    vertex_index: u32,
    vertex_texture_index: Option<u32>,
    vertex_normal_index: Option<u32>,
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
        {
            let test: Vec<&str> = str.split('/').collect();
            dbg!(test);
        }

        let mut raw_vertex_attributes: Vec<Option<u32>> =
            str.split('/').map(Self::vertex_attribute_parse).collect();

        if raw_vertex_attributes.len() > 3 {
            panic!("{:?}", &raw_vertex_attributes);
        }

        raw_vertex_attributes.resize(3, None);
        let (vertex_index, vertex_texture_index, vertex_normal_index) =
            match raw_vertex_attributes[..] {
                [Some(v_index), tex_index, norm_index] => (v_index, tex_index, norm_index),
                _ => panic!("{:?}", raw_vertex_attributes),
            };

        Self {
            vertex_index,
            vertex_texture_index,
            vertex_normal_index,
        }
    }
}

impl Face {
    pub fn parse(line: &str) -> Self {
        let words = line.split_whitespace().collect::<Vec<&str>>();

        if (words[0] != "f") {
            panic!();
        }

        let vertex_attributes = &words[1..];
        if vertex_attributes.len() < 3 {
            panic!();
        }

        Self {
            vertex_attributes: vertex_attributes
                .iter()
                .map(|word| VertexAttribute::parse(word))
                .collect(),
        }
    }
}
