use face::Face;

mod face;

pub struct ObjAsset {
    faces: Vec<Face>,
}

impl ObjAsset {
    pub fn parse(str: &str) -> Self {
        let lines = str.lines();

        Self {
            faces: lines.map(Face::parse).collect(),
        }
    }
}
