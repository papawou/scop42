use std::collections::HashMap;

use material::Material;

mod material;

struct Mtl {
    materials: HashMap<String, Material>,
}

impl Mtl {
    pub fn parse(str: &str) -> Self {
        let mut materials: HashMap<String, Material> = HashMap::new();

        let lines = str.lines().filter(|line| !line.trim().is_empty());

        let mut current_material: Option<Material> = None;

        for line in lines.map(|line| line.trim()) {
            let mut words = line.split_whitespace();

            if let Some(word) = words.next() {
                // for each line
                match word {
                    "newmtl" => {
                        // extract current material and insert it in hash
                        if let Some(prev_material) = current_material.take() {
                            materials.insert(prev_material.group.clone(), prev_material);
                        }
                        current_material =
                            Some(Material::new(&words.collect::<Vec<&str>>().join(" ")));
                    }
                    // all subsequent call are current_material related
                    material_word => {
                        if let Some(material) = current_material.as_mut() {
                            material.parse(line);
                        } else {
                            panic!("Material is None when expected to be Some");
                        }
                    }
                }
            }
        }

        // finally add last material
        if let Some(last_material) = current_material.take() {
            materials.insert(last_material.group.clone(), last_material);
        }

        Self { materials }
    }
}
