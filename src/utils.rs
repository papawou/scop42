use std::io::Read;

use anyhow;

pub fn i8_to_str(bytes: &[i8]) -> anyhow::Result<String> {
    Ok(String::from_utf8(bytes.iter().map(|&b| b as u8).collect())?)
}

pub fn read_file<P: AsRef<std::path::Path>>(filename: P) -> anyhow::Result<Vec<u8>> {
    let file = std::fs::File::open(filename)?;
    let mut buf_reader = std::io::BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents)?;

    Ok(contents)
}

pub fn from_u8_to_u32(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("Chunk should be 4 bytes")))
        .collect()
}
