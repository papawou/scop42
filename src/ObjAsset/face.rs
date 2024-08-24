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
        println!("{}", str);
        if str.is_empty() {
            None
        } else {
            Some(str.parse::<u32>().unwrap())
        }
    }

    pub fn parse(str: &str) -> Self {
        let mut data = str.split('/');

        let vertex_index = data.next().and_then(Self::vertex_attribute_parse).unwrap();
        let vertex_texture_index = data.next().and_then(Self::vertex_attribute_parse);
        let vertex_normal_index = data.next().and_then(Self::vertex_attribute_parse);

        if (data.next().is_some()) {
            panic!();
        }

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
