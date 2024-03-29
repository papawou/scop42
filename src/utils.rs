use anyhow;

pub fn i8_to_str(bytes: &[i8]) -> anyhow::Result<String> {
    Ok(String::from_utf8(bytes.iter().map(|&b| b as u8).collect())?)
}
