use material::Material;

mod material;

struct Mtl {
    materials: Vec<Material>,
}

impl Mtl {
    pub fn parse(str: &str) -> Self {
        let mut materials: Vec<Material> = vec![];

        let lines = str.lines().filter(|line| !line.trim().is_empty());

        let mut material: Option<Material> = None;

        for line in lines.map(|line| line.trim()) {
            let mut words = line.split_whitespace();

            if let Some(word) = words.next() {
                // for each line
                match word {
                    "newmtl" => {
                        // extract current material and store it in materials
                        if let Some(prev_material) = material.take() {
                            materials.push(prev_material);
                        }
                        material = Some(Material::new(&words.collect::<Vec<&str>>().join(" ")));
                    }
                    material_word => {
                        if let Some(material) = material.as_mut() {
                            material.parse(line);
                        } else {
                            panic!("Material is None when expected to be Some");
                        }
                    }
                }
            }
        }

        Self { materials }
    }
}
