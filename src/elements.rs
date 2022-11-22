use std::borrow::Cow;
use std::cmp::min;
use std::collections::HashSet;
use std::hash::Hash;
use unicode_segmentation::UnicodeSegmentation;
use crate::compressed::IndexSet;
use crate::keyboard::{Keyboard, KEYBOARD_QUANPIN};
use crate::pinin::{FuzzySettings, PinIn};

const VOWEL_CHARS: [char; 6] = ['a', 'e', 'i', 'o', 'u', 'v'];

#[derive(Debug, Hash, PartialEq)]
pub struct Phoneme<'a> {
    strings: Vec<Cow<'a, str>>
}

impl<'a> Phoneme<'a> {
    pub fn new(s: &'a str, settings: &FuzzySettings, keyboard: &Keyboard) -> Self {
        let mut ret = HashSet::new();
        ret.insert(Cow::Borrowed(s));

        if let Some(c) = s.chars().next() {
            match c {
                'c' => if settings.ch2c {
                    ret.insert(Cow::Borrowed("c"));
                    ret.insert(Cow::Borrowed("ch"));
                },
                's' => if settings.sh2s {
                    ret.insert(Cow::Borrowed("s"));
                    ret.insert(Cow::Borrowed("sh"));
                }
                'z' => if settings.zh2z {
                    ret.insert(Cow::Borrowed("z"));
                    ret.insert(Cow::Borrowed("zh"));
                }
                'v' => if settings.u2v {
                    let mut str = String::from("u");
                    str.push_str(&s[1..s.len()]);
                    ret.insert(Cow::Owned(str));
                }
                _ => {}
            }
        }

        if (settings.ang2an && s.ends_with("ang")) ||
            (settings.eng2en && s.ends_with("eng")) ||
            (settings.ing2in && s.ends_with("ing")) {
            ret.insert(Cow::Borrowed(&s[0..s.len() - 1]));
        }

        if (settings.ang2an && s.ends_with("an")) ||
            (settings.eng2en && s.ends_with("en")) ||
            (settings.ing2in && s.ends_with("in")) {
            let mut str = s.to_string();
            str.push('g');
            ret.insert(Cow::Owned(str));
        }

        Phoneme {
            strings: ret.into_iter().map(|x| keyboard.keys_cow(x)).collect()
        }
    }

    pub fn new_cow(s: Cow<'a, str>, settings: &FuzzySettings, keyboard: &Keyboard) -> Self {
        let mut ret = HashSet::new();
        ret.insert(s.clone());

        if let Some(c) = s.chars().next() {
            match c {
                'c' => if settings.ch2c {
                    ret.insert(Cow::Borrowed("c"));
                    ret.insert(Cow::Borrowed("ch"));
                },
                's' => if settings.sh2s {
                    ret.insert(Cow::Borrowed("s"));
                    ret.insert(Cow::Borrowed("sh"));
                }
                'z' => if settings.zh2z {
                    ret.insert(Cow::Borrowed("z"));
                    ret.insert(Cow::Borrowed("zh"));
                }
                'v' => if settings.u2v {
                    let mut str = String::from("u");
                    str.push_str(&s[1..s.len()]);
                    ret.insert(Cow::Owned(str));
                }
                _ => {}
            }
        }

        if (settings.ang2an && s.ends_with("ang")) ||
            (settings.eng2en && s.ends_with("eng")) ||
            (settings.ing2in && s.ends_with("ing")) {
            ret.insert(Cow::Borrowed(&s[0..s.len() - 1]));
        }

        if (settings.ang2an && s.ends_with("an")) ||
            (settings.eng2en && s.ends_with("en")) ||
            (settings.ing2in && s.ends_with("in")) {
            let mut str = s.to_string();
            str.push('g');
            ret.insert(Cow::Owned(str));
        }

        Phoneme {
            strings: ret.into_iter().map(|x| keyboard.keys_cow(x)).collect()
        }
    }

    pub fn strcmp(a: &str, b: &str, a_start: usize) -> usize {
        let a_graphemes = a.graphemes(true).collect::<Vec<&str>>();
        let b_graphemes = b.graphemes(true).collect::<Vec<&str>>();
        let len = min(a_graphemes.len(), b_graphemes.len());
        for i in 0..len {
            if a_graphemes[i + a_start] != b_graphemes[i] {
                return i;
            }
        }
        len
    }

    pub fn match_string_idx(&self, source: &str, idx: IndexSet, start: usize, partial: bool) -> IndexSet {
        if self.is_empty() {
            return idx;
        }
        let mut ret = IndexSet::default();
        for i in idx.iter() {
            let mut set = self.match_string(source, start + i as usize, partial);
            set.offset(i);
            ret.merge(set);
        }
        ret
    }

    pub fn is_empty(&self) -> bool {
        self.strings.len() == 1 && self.strings[0].is_empty()
    }

    pub fn match_string(&self, source: &str, start: usize, partial: bool) -> IndexSet {
        let mut ret = IndexSet::default();
        if self.is_empty() {
            return ret;
        }
        for s in self.strings.iter() {
            let size = Self::strcmp(source, s, start);
            if (partial && start + size == source.len()) || size == s.len() {
                ret.set(size);
            }
        }
        ret
    }
}

#[derive(Clone)]
pub struct Character<'a> {
    pub ch: char,
    pub pinyin: Vec<&'a Pinyin<'a>>
}

impl<'a> Character<'a> {
    pub fn new(ch: char, pinyin: Vec<&'a Pinyin<'a>>) -> Self {
        Character {
            ch, pinyin
        }
    }

    pub fn match_str(&self, s: &str, start: usize, partial: bool) -> IndexSet {
        let mut ret = if s.chars().skip(start).next() == Some(self.ch) { IndexSet::one() } else { IndexSet::none() };
        self.pinyin.iter().for_each(|p| ret.merge(p.match_string(s, start, partial)));
        ret
    }
}

pub struct Pinyin<'a> {
    pub raw: &'a str,
    pub id: usize,
    pub duo: bool,
    pub sequence: bool,
    pub phonemes: Vec<Phoneme<'a>>
}

impl<'a> Pinyin<'a> {
    pub fn new(s: &'a str, settings: &FuzzySettings, keyboard: &Keyboard, id: usize) -> Pinyin<'a> {
        let split = keyboard.split(s);
        let phonemes: Vec<Phoneme> = split.into_iter().map(|x| Phoneme::new_cow(x, settings, keyboard)).collect();

        Pinyin {
            id,
            phonemes,
            raw: s,
            duo: keyboard.duo,
            sequence: keyboard.sequence,
        }
    }

    pub fn match_string(&self, s: &str, start: usize, partial: bool) -> IndexSet {
        if self.duo {
            let mut ret = IndexSet::zero();
            ret = self.phonemes[0].match_string_idx(s, ret, start, partial);
            ret = self.phonemes[1].match_string_idx(s, ret, start, partial);
            if self.phonemes.len() == 3 {
                ret.merge(self.phonemes[2].match_string_idx(s, ret, start, partial));
            }
            ret
        } else {
            let mut active = IndexSet::zero();
            let mut ret = IndexSet::none();

            self.phonemes.iter().for_each(|phoneme| {
                active = phoneme.match_string_idx(s, active, start, partial);
                if active == IndexSet::none() {
                    return;
                }
                ret.merge(active);
            });
            ret
        }
    }

    pub fn has_initial(s: &str) -> bool {
        VOWEL_CHARS.iter().all(|x| s.chars().next().map(|c| c != *x).unwrap_or(false))
    }
}