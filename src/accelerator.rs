use crate::compressed::{IndexSet, IndexSetStorage};
use crate::elements::Pinyin;
use crate::pinin::PinIn;
use std::cell::{Cell, RefCell};
use std::ops::Index;
use std::rc::Rc;
use compact_str::CompactString;

pub trait CharProvider: Index<usize, Output = char> {
    fn end(&self, index: usize) -> bool;
}

#[derive(Default)]
pub struct StringProvider {
    s: Vec<char>,
}

impl Index<usize> for StringProvider {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.s[index]
    }
}

impl From<&str> for StringProvider {
    fn from(s: &str) -> Self {
        StringProvider {
            s: s.chars().collect(),
        }
    }
}

impl CharProvider for StringProvider {
    fn end(&self, index: usize) -> bool {
        index >= self.s.len()
    }
}

pub struct Accelerator {
    cache: Rc<RefCell<Vec<IndexSetStorage>>>,

    search_chars: RefCell<Vec<char>>,
    pub search_string: RefCell<CompactString>,
    pub provider: RefCell<Option<Rc<RefCell<dyn CharProvider>>>>,

    partial: Cell<bool>,
}

impl Default for Accelerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Accelerator {
    pub fn new() -> Self {
        Accelerator {
            cache: Rc::new(RefCell::new(Default::default())),
            search_chars: RefCell::new(Default::default()),
            search_string: RefCell::new("".into()),
            provider: RefCell::new(None),
            partial: Cell::new(false),
        }
    }

    pub fn matches(&self, context: &PinIn, offset: usize, start: usize) -> bool {
        if self.partial.get() {
            self.partial.set(false);
            self.reset();
        }
        self.check(context, offset, start)
    }

    pub fn begins(&self, context: &PinIn, offset: usize, start: usize) -> bool {
        if !self.partial.get() {
            self.partial.set(true);
            self.reset();
        }
        self.check(context, offset, start)
    }

    pub fn contains(&self, context: &PinIn, offset: usize, start: usize) -> bool {
        if !self.partial.get() {
            self.partial.set(true);
            self.reset();
        }
        if let Some(provider) = self.provider.borrow().as_ref() {
            let provider = provider.borrow();
            let mut i = start;
            while !provider.end(i) {
                if self.check(context, offset, i) {
                    return true;
                }

                i += 1;
            }
        }
        false
    }

    pub fn common(&self, s1: usize, s2: usize, max: usize) -> usize {
        if let Some(provider) = self.provider.borrow().as_ref() {
            let provider = provider.borrow();
            let mut i = 0;
            loop {
                if i >= max {
                    return max;
                }
                let a = provider[s1 + i];
                let b = provider[s2 + i];
                if a != b || a == '\0' {
                    return i;
                }

                i += 1;
            }
        }

        0
    }

    pub fn search(&self, s: &str) {
        if self.search_string.borrow().as_str() != s {
            *self.search_string.borrow_mut() = s.into();
            *self.search_chars.borrow_mut() = s.chars().collect();
            self.reset();
        }
    }

    pub fn reset(&self) {
        self.cache.borrow_mut().clear();
    }

    pub fn get(&self, context: &PinIn, ch: char, offset: usize) -> IndexSet {
        let c = context.get_character(ch);
        let mut ret = if self.search_chars.borrow()[offset] == ch {
            IndexSet::one()
        } else {
            IndexSet::none()
        };
        c.pinyin
            .iter()
            .for_each(|x| ret.merge(self.get_pinyin(x, offset)));
        ret
    }

    pub fn get_pinyin(&self, p: &Pinyin, offset: usize) -> IndexSet {
        let mut cache = self.cache.borrow_mut();
        cache.resize_with(offset + 1, IndexSetStorage::new);
        let data = &mut cache[offset];
        let ret = data.get(p.id);
        if ret != IndexSet::null() {
            return ret;
        }

        let set = p.match_string(self.search_string.borrow().as_str(), offset, self.partial.get());
        data.set(set, p.id);
        set
    }

    pub fn check(&self, context: &PinIn, offset: usize, start: usize) -> bool {
        if let Some(provider) = self.provider.borrow().as_ref() {
            let provider = provider.borrow();
            if offset == self.search_string.borrow().chars().count() {
                return self.partial.get() || provider.end(start);
            }

            if provider.end(start) {
                return false;
            }

            let s = self.get(context, provider[start], offset);

            return if provider.end(start + 1) {
                let i = self.search_string.borrow().chars().count() - offset;
                s.get(i)
            } else {
                s.traverse(|i| self.check(context, offset + i as usize, start + 1))
            }
        }

        false
    }
}
