#[derive(Clone)]
pub struct Group(String);

impl Group {
    pub fn parse(line: &str) -> Self {
        let mut words = line.split_whitespace();

        match words.next() {
            Some("o") => (),
            _ => panic!(),
        }

        Self(words.collect::<Vec<_>>().join(" "))
    }
}
