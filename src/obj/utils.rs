use glam::Vec3;

pub fn parse_vec3_with_default<'a>(
    words: &mut impl Iterator<Item = &'a str>,
    default: Option<Vec3>,
) -> Vec3 {
    let mut components: [f32; 3] = if let Some(default) = default {
        [default.x, default.y, default.z]
    } else {
        [0.0; 3]
    };

    for i in 0..3 {
        if let Some(value_str) = words.next() {
            components[i] = value_str.parse::<f32>().unwrap_or(components[i]);
        }
    }

    Vec3::new(components[0], components[1], components[2])
}
