use crate::accelerator::Accelerator;
use crate::compressed::Compressor;
use crate::pinin::PinIn;
use std::collections::HashSet;

pub struct SimpleSearcher<'a, T> {
    context: &'a PinIn<'a>,

    objects: Vec<T>,
    accelerator: Accelerator,
    compressor: Compressor,
    logic: SearcherLogic,
}

impl<'a, T> SimpleSearcher<'a, T> {
    pub fn insert(&mut self, name: &str, id: T) {
        self.compressor.push(name);
        name.chars().for_each(|c| {
            self.context.get_character(c);
        });

        self.objects.push(id);
    }

    pub fn search(&mut self, name: &str) -> Vec<&T> {
        self.accelerator.search(name);
        let offsets = &self.compressor.offsets;
        offsets
            .iter()
            .enumerate()
            .filter(|(_i, s)| self.logic.test_accelerator(&self.accelerator, 0, **s))
            .map(|(i, _)| &self.objects[i])
            .collect()
    }

    pub fn reset(&mut self) {
        self.accelerator.reset();
    }
}

const BTREE_THRESHOLD: usize = 256;

pub struct TreeSearcher {}

pub struct NDense {
    data: Vec<usize>,
}

impl NDense {
    pub fn get_accelerator(
        &self,
        logic: &SearcherLogic,
        accelerator: &Accelerator,
        offset: usize,
        collection: &mut HashSet<usize>,
    ) {
        let full = logic == &SearcherLogic::Equal;
        if !full && accelerator.search_string.len() == offset {
            self.get(collection);
        } else {
            for i in 0..self.data.len() / 2 {
                let ch = self.data[i * 2];
                if (full && accelerator.matches(offset, ch)) || accelerator.begins(offset, ch) {
                    collection.insert(self.data[i * 2 + 1]);
                }
            }
        }
    }

    pub fn get(&self, collection: &mut HashSet<usize>) {
        for i in 0..self.data.len() / 2 {
            collection.insert(self.data[i * 2 + 1]);
        }
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum SearcherLogic {
    Begin,
    Contain,
    Equal,
}

impl SearcherLogic {
    pub fn test_accelerator(&self, a: &Accelerator, offset: usize, start: usize) -> bool {
        match *self {
            SearcherLogic::Begin => a.begins(offset, start),
            SearcherLogic::Contain => a.contains(offset, start),
            SearcherLogic::Equal => a.matches(offset, start),
        }
    }

    pub fn test_pinyin(&self, p: &PinIn, s1: &str, s2: &str) -> bool {
        match *self {
            SearcherLogic::Begin => p.begins(s1, s2),
            SearcherLogic::Contain => p.contains(s1, s2),
            SearcherLogic::Equal => p.matches(s1, s2),
        }
    }
}
