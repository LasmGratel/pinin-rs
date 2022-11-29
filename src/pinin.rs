use crate::accelerator::{Accelerator, StringProvider};
use crate::dict_loader::DictLoader;
use crate::elements::{Character, Pinyin};
use crate::format::{number_format, PinyinFormat};
use crate::keyboard::{Keyboard, KEYBOARD_QUANPIN};
use std::borrow::{BorrowMut, Cow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct PinIn<'a> {
    pub(crate) chars: HashMap<char, Option<Character<'a>>>,

    pub keyboard: &'static Keyboard,
    pub fuzzy: FuzzySettings,
    pub format: PinyinFormat<'a>,
    pub accelerate: bool,
    pub accelerator: Option<Rc<RefCell<Accelerator>>>,

    pub(crate) pinyins: Rc<RefCell<HashMap<&'a str, Rc<Pinyin<'a>>>>>,

    total: AtomicUsize,
}

#[derive(Default, Debug)]
pub struct FuzzySettings {
    pub zh2z: bool,
    pub sh2s: bool,
    pub ch2c: bool,
    pub ang2an: bool,
    pub ing2in: bool,
    pub eng2en: bool,
    pub u2v: bool,
}

impl<'a> PinIn<'a> {
    pub fn new() -> PinIn<'a> {
        let mut p = PinIn {
            chars: HashMap::new(),
            keyboard: &KEYBOARD_QUANPIN,
            fuzzy: FuzzySettings::default(),
            format: Box::new(number_format),
            accelerate: false,
            accelerator: None,

            pinyins: Rc::new(RefCell::new(HashMap::new())),
            total: AtomicUsize::default(),
        };
        p.accelerator = Some(Rc::new(RefCell::new(Accelerator::new())));

        p
    }

    pub fn get_or_insert_pinyin(&self, x: &'a str) -> Rc<Pinyin<'a>> {
        self.pinyins
            .as_ref()
            .borrow_mut()
            .entry(x)
            .or_insert(Rc::new(Pinyin::new(
                x,
                &self.fuzzy,
                &self.keyboard,
                self.total.fetch_add(1, Ordering::SeqCst),
            )))
            .clone()
    }

    pub fn load_dict(&mut self, loader: Box<dyn DictLoader<'a>>) {
        loader.load_dict().into_iter().for_each(|(c, ss)| {
            if ss.is_empty() {
                self.chars.insert(c, None);
            } else {
                self.chars.insert(
                    c,
                    Some(Character::new(
                        c,
                        ss.iter().map(|s| self.get_or_insert_pinyin(s)).collect(),
                    )),
                );
            }
        });
    }

    pub fn get_character<'b>(&'b self, c: char) -> Cow<'b, Character> {
        self.chars
            .get(&c)
            .map(|x| x.as_ref().map(|y| Cow::Borrowed(y)))
            .flatten()
            .unwrap_or(Cow::Owned(Character::new(c, vec![])))
    }

    pub fn check(&self, s1: &str, start1: usize, s2: &str, start2: usize, partial: bool) -> bool {
        if start2 == s2.chars().count() {
            return partial || start1 == s1.chars().count();
        }

        let r = self.get_character(s1.chars().skip(start1).next().unwrap());
        let s = r.match_str(s2, start2, partial);

        if start1 == s1.chars().count() - 1 {
            let i = s2.chars().count() - start2;
            return s.get(i);
        }

        return s.traverse(|i| self.check(s1, start1 + 1, s2, start2 + i as usize, partial));
    }

    pub fn contains(&self, s1: &str, s2: &str) -> bool {
        if !self.accelerate {
            return if s1.trim().is_empty() {
                s1.contains(s2)
            } else {
                s1.chars()
                    .enumerate()
                    .any(|(i, _)| self.check(s1, i, s2, 0, true))
            };
        }

        let mut a = self.accelerator.as_ref().unwrap().as_ref().borrow_mut();
        a.provider = Some(Box::new(StringProvider::from(s1)));
        a.search(s2);
        a.contains(0, 0)
    }

    pub fn begins(&self, s1: &str, s2: &str) -> bool {
        if !self.accelerate {
            return if s1.trim().is_empty() {
                s1.starts_with(s2)
            } else {
                self.check(s1, 0, s2, 0, true)
            };
        }

        let mut a = self.accelerator.as_ref().unwrap().as_ref().borrow_mut();
        a.provider = Some(Box::new(StringProvider::from(s1)));
        a.search(s2);
        a.begins(0, 0)
    }

    pub fn matches(&self, s1: &str, s2: &str) -> bool {
        if !self.accelerate {
            return if s1.trim().is_empty() {
                s1 == s2
            } else {
                self.check(s1, 0, s2, 0, true)
            };
        }

        let mut a = self.accelerator.as_ref().unwrap().as_ref().borrow_mut();
        a.provider = Some(Box::new(StringProvider::from(s1)));
        a.search(s2);
        a.begins(0, 0)
    }
}
