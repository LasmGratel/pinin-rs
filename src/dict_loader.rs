use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

pub trait DictLoader<'a> {
    fn load_dict(&self) -> HashMap<char, Vec<&'a str>>;
}

impl<'a> DictLoader<'a> for &'a str {
    fn load_dict(&self) -> HashMap<char, Vec<&'a str>> {
        self.lines()
            .map(|line: &str| {
                let ch = line.chars().next().unwrap();
                let index = line.grapheme_indices(true).nth(3).unwrap().0;
                let records = line[index..].split(", ").collect::<Vec<&str>>();
                (ch, records)
            })
            .collect()
    }
}
