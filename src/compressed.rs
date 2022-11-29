use std::fmt::{Display, Formatter};
use std::ops::Index;
use smallvec::SmallVec;

use crate::accelerator::CharProvider;

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct IndexSet {
    value: i32,
}

impl Display for IndexSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        self.for_each(|i| s.push_str(&format!("{}, ", i)));

        if s.is_empty() {
            s.push('0');
        }
        f.write_str(&s)
    }
}

impl From<i32> for IndexSet {
    fn from(value: i32) -> Self {
        IndexSet::new(value)
    }
}

impl IndexSet {
    pub fn zero() -> Self {
        Self::from(0x1)
    }

    pub fn one() -> Self {
        Self::from(0x2)
    }

    pub fn none() -> Self {
        Self::from(0x0)
    }

    pub fn null() -> Self {
        Self::from(-1)
    }

    pub fn new(value: i32) -> Self {
        IndexSet { value }
    }

    pub fn set(&mut self, index: usize) {
        self.value |= 0x1 << index;
    }

    pub fn get(&self, index: usize) -> bool {
        self.value & (0x1 << index) != 0
    }

    pub fn for_each<F>(&self, mut c: F)
    where
        F: FnMut(i32),
    {
        let mut v = self.value;
        for i in 0..7 {
            if (v & 0x1) == 0x1 {
                c(i);
            } else if v == 0 {
                return;
            }
            v >>= 1;
        }
    }

    pub fn traverse<F>(&self, p: F) -> bool
    where
        F: Fn(i32) -> bool,
    {
        let mut v = self.value;
        for i in 0..7 {
            if (v & 0x1) == 0x1 && p(i) {
                return true;
            }
            if v == 0 {
                return false;
            }
            v >>= 1;
        }
        false
    }

    pub fn offset(&mut self, i: i32) {
        self.value <<= i;
    }

    pub fn merge(&mut self, s: Self) {
        if self.value == 0x1 {
            self.value = s.value;
        } else {
            self.value |= s.value;
        }
    }
}

pub struct IndexSetStorage {
    data: SmallVec<[i32; 16]>,
}

impl Default for IndexSetStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexSetStorage {
    pub fn new() -> Self {
        IndexSetStorage {
            data: SmallVec::from([0; 16]),
        }
    }

    pub fn set(&mut self, set: IndexSet, index: usize) {
        if index >= self.data.len() {
            let mut size = index;
            size |= size >> 1;
            size |= size >> 2;
            size |= size >> 4;
            size |= size >> 8;
            size |= size >> 16;
            self.data.resize(size + 1, 0);
        }
        self.data[index] = set.value + 1;
    }

    pub fn get(&self, index: usize) -> IndexSet {
        if let Some(ret) = self.data.get(index) {
            if *ret != 0 {
                IndexSet::from(ret - 1)
            } else {
                IndexSet::null()
            }
        } else {
            IndexSet::null()
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Compressor {
    pub chars: Vec<char>,
    pub offsets: Vec<usize>,
}

impl Index<usize> for Compressor {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chars[index]
    }
}

impl CharProvider for Compressor {
    fn end(&self, index: usize) -> bool {
        self.chars.get(index) == Some(&'\0')
    }
}

impl Compressor {
    pub fn push(&mut self, s: &str) -> usize {
        self.offsets.push(self.chars.len());
        s.chars().for_each(|c| self.chars.push(c));
        self.chars.push('\0');
        self.offsets.last().copied().unwrap_or(0)
    }
}
