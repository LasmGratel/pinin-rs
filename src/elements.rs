use std::borrow::Cow;
use std::cmp::min;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::rc::Rc;
use compact_str::CompactString;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use crate::compressed::IndexSet;
use crate::keyboard::Keyboard;
use crate::pinin::FuzzySettings;
use crate::unicode_utils::SegmentedStr;

const VOWEL_CHARS: [char; 6] = ['a', 'e', 'i', 'o', 'u', 'v'];

#[derive(Hash, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub enum Phoneme {
    Single(CompactString),
    Multiple(Vec<CompactString>),
}

impl Debug for Phoneme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Phoneme::Single(x) => {
                f.write_str(x)
            }
            Phoneme::Multiple(strings) => {
                f.debug_list().entries(strings.iter()).finish()
            }
        }
    }
}

impl Phoneme {
    pub fn new(s: &str, settings: &FuzzySettings, keyboard: &Keyboard) -> Self {
        let mut ret = FxHashSet::default();
        ret.insert(Cow::Borrowed(s));

        if let Some(c) = s.chars().next() {
            match c {
                'c' => {
                    if settings.ch2c {
                        ret.insert(Cow::Borrowed("c"));
                        ret.insert(Cow::Borrowed("ch"));
                    }
                }
                's' => {
                    if settings.sh2s {
                        ret.insert(Cow::Borrowed("s"));
                        ret.insert(Cow::Borrowed("sh"));
                    }
                }
                'z' => {
                    if settings.zh2z {
                        ret.insert(Cow::Borrowed("z"));
                        ret.insert(Cow::Borrowed("zh"));
                    }
                }
                'v' => {
                    if settings.u2v {
                        let mut str = String::from("u");
                        str.push_str(&s[1..s.len()]);
                        ret.insert(Cow::Owned(str));
                    }
                }
                _ => {}
            }
        }

        if (settings.ang2an && s.ends_with("ang"))
            || (settings.eng2en && s.ends_with("eng"))
            || (settings.ing2in && s.ends_with("ing"))
        {
            ret.insert(Cow::Borrowed(&s[0..s.len() - 1]));
        }

        if (settings.ang2an && s.ends_with("an"))
            || (settings.eng2en && s.ends_with("en"))
            || (settings.ing2in && s.ends_with("in"))
        {
            let mut str = s.to_string();
            str.push('g');
            ret.insert(Cow::Owned(str));
        }

        if ret.len() == 1 {
            Phoneme::Single(keyboard.keys(s).into())
        } else {
            Phoneme::Multiple(ret.into_iter().map(|x| keyboard.keys_cow(x).into()).collect())
        }
    }
/*
    pub fn new_cow(s: Cow<'a, str>, settings: &FuzzySettings, keyboard: &Keyboard) -> Self {
        let mut ret = HashSet::new();
        ret.insert(s.clone());

        if let Some(c) = s.chars().next() {
            match c {
                'c' => {
                    if settings.ch2c {
                        ret.insert(Cow::Borrowed("c"));
                        ret.insert(Cow::Borrowed("ch"));
                    }
                }
                's' => {
                    if settings.sh2s {
                        ret.insert(Cow::Borrowed("s"));
                        ret.insert(Cow::Borrowed("sh"));
                    }
                }
                'z' => {
                    if settings.zh2z {
                        ret.insert(Cow::Borrowed("z"));
                        ret.insert(Cow::Borrowed("zh"));
                    }
                }
                'v' => {
                    if settings.u2v {
                        let mut str = String::from("u");
                        str.push_str(&s[1..s.len()]);
                        ret.insert(Cow::Owned(str));
                    }
                }
                _ => {}
            }
        }

        if (settings.ang2an && s.ends_with("ang"))
            || (settings.eng2en && s.ends_with("eng"))
            || (settings.ing2in && s.ends_with("ing"))
        {
            ret.insert(Cow::Borrowed(&s[0..s.len() - 1]));
        }

        if (settings.ang2an && s.ends_with("an"))
            || (settings.eng2en && s.ends_with("en"))
            || (settings.ing2in && s.ends_with("in"))
        {
            let mut str = s.to_string();
            str.push('g');
            ret.insert(Cow::Owned(str));
        }

        Phoneme {
            strings: ret.into_iter().map(|x| keyboard.keys_cow(x)).collect(),
        }
    }
*/
    pub fn strcmp(a: &SegmentedStr, b: &SegmentedStr, a_start: usize) -> usize {
        let len = min(a.graphemes.len() - a_start, b.graphemes.len());
        for i in 0..len {
            if a.graphemes[i + a_start].1 != b.graphemes[i].1 {
                return i;
            }
        }
        len
    }

    pub fn match_string_idx(
        &self,
        source: &str,
        idx: IndexSet,
        start: usize,
        partial: bool,
    ) -> IndexSet {
        if self.is_empty() {
            return idx;
        }
        let mut ret = IndexSet::default();
        idx.for_each(|i| {
            let mut set = self.match_string(source, start + i as usize, partial);
            set.offset(i);
            ret.merge(set);
        });
        ret
    }

    pub fn is_empty(&self) -> bool {
        match &self {
            Phoneme::Single(s) => { s.is_empty() }
            Phoneme::Multiple(_) => { false }
        }
    }

    pub fn match_sequence(&self, c: char) -> bool {
        match &self {
            Phoneme::Single(s) => { s.chars().next().unwrap() == c }
            Phoneme::Multiple(strings) => { strings.iter().any(|s| s.chars().next().unwrap() == c) }
        }
    }

    pub fn match_string(&self, source: &str, start: usize, partial: bool) -> IndexSet {
        let mut ret = IndexSet::default();

        let source: SegmentedStr = source.into();
        match &self {
            Phoneme::Single(s) => {
                if s.trim().is_empty() {
                    return ret;
                }

                let s = s.as_str().into();
                let size = Self::strcmp(&source, &s, start);
                if (partial && start + size == source.graphemes.len()) || size == s.graphemes.len() {
                    ret.set(size);
                }
            }
            Phoneme::Multiple(strings) => {
                for s in strings.iter() {
                    let s = s.as_str().into();
                    let size = Self::strcmp(&source, &s, start);
                    if (partial && start + size == source.graphemes.len()) || size == s.graphemes.len() {
                        ret.set(size);
                    }
                }
            }
        }

        ret
    }
}

#[derive(Clone, Debug)]
pub struct Character {
    pub ch: char,
    pub pinyin: SmallVec<[Pinyin; 4]>,
}

impl Character {
    pub fn new(ch: char, pinyin: SmallVec<[Pinyin; 4]>) -> Self {
        Character { ch, pinyin }
    }

    pub fn match_str(&self, s: &str, start: usize, partial: bool) -> IndexSet {
        let mut ret = if s.chars().nth(start) == Some(self.ch) {
            IndexSet::one()
        } else {
            IndexSet::none()
        };
        self.pinyin
            .iter()
            .for_each(|p| ret.merge(p.match_string(s, start, partial)));
        ret
    }
}

#[derive(Debug, Clone)]
pub struct Pinyin {
    pub raw: CompactString,
    pub id: usize,
    pub duo: bool,
    pub sequence: bool,
    pub phonemes: SmallVec<[Phoneme; 4]>,
}

impl Pinyin {
    pub fn new(s: &str, settings: &FuzzySettings, keyboard: &Keyboard, id: usize) -> Pinyin {
        let split = keyboard.split(s);
        let phonemes: SmallVec<[Phoneme; 4]> = split
            .into_iter()
            .map(|x| Phoneme::new(&x, settings, keyboard))
            .collect();

        Pinyin {
            id,
            phonemes,
            raw: s.into(),
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
            // in other keyboards, match of precedent phoneme
            // is compulsory to match subsequent phonemes
            // for example, zhong1, z+h+ong+1 cannot match zong or zh1
            let mut active = IndexSet::zero();
            let mut ret = IndexSet::none();

            self.phonemes.iter().for_each(|phoneme| {
                active = phoneme.match_string_idx(s, active, start, partial);
                if active == IndexSet::none() {
                    return;
                }
                ret.merge(active);
            });

            if self.sequence
                && self.phonemes[0].match_sequence(s.chars().nth(start).unwrap())
            {
                ret.set(1);
            }
            ret
        }
    }

    pub fn has_initial(s: &str) -> bool {
        VOWEL_CHARS
            .iter()
            .all(|x| s.chars().next().map(|c| c != *x).unwrap_or(false))
    }
}
