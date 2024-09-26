use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use material::Material;

mod material;

pub struct MaterialLib {
    filepath: PathBuf,
    materials: HashMap<String, Material>,
}

impl MaterialLib {
    pub fn parse(filepath: &Path, data: &str) -> Self {
        let lines = data.lines().filter(|line| !line.trim().is_empty());

        let mut materials: HashMap<String, Material> = HashMap::new();
        let mut current_material: Option<Material> = None;

        for line in lines.map(|line| line.trim()) {
            let mut words = line.split_whitespace();

            if let Some(word) = words.next() {
                // for each line
                match word {
                    "#" => {}
                    "newmtl" => {
                        // extract current material and insert it in hash
                        if let Some(prev_material) = current_material.take() {
                            materials.insert(prev_material.material_name.clone(), prev_material);
                        }
                        current_material =
                            Some(Material::new(&words.collect::<Vec<&str>>().join(" ")));
                    }
                    // all subsequent call are current_material related
                    material_word => {
                        let material = current_material.as_mut().unwrap();
                        material.parse(line);
                    }
                }
            }
        }

        // finally add last material
        if let Some(last_material) = current_material.take() {
            materials.insert(last_material.material_name.clone(), last_material);
        }

        Self {
            filepath: filepath.to_path_buf(),
            materials,
        }
    }

    pub fn load_from_file(filepath: &Path) -> Self {
        let mut file = File::open(&filepath).unwrap();

        let mut data = String::new();
        file.read_to_string(&mut data);

        Self::parse(filepath, &data)
    }
}
