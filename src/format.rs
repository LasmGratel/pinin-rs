use crate::elements::Pinyin;
use crate::unicode_utils::UnicodeUtils;
use lazy_static::lazy_static;
use std::borrow::Cow;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

const OFFSET: &[&str] = &[
    "ui", "iu", "uan", "uang", "ian", "iang", "ua", "ie", "uo", "iong", "iao", "ve", "ia",
];

pub type PinyinFormat<'a> = Box<dyn Fn(&'a Pinyin) -> Cow<'a, str>>;

lazy_static! {
    static ref NONE: HashMap<char, char> = HashMap::from([
        ('a', 'a'),
        ('o', 'o'),
        ('e', 'e'),
        ('i', 'i'),
        ('u', 'u'),
        ('v', 'ü')
    ]);
    static ref FIRST: HashMap<char, char> = HashMap::from([
        ('a', 'ā'),
        ('o', 'ō'),
        ('e', 'ē'),
        ('i', 'ī'),
        ('u', 'ū'),
        ('v', 'ǖ')
    ]);
    static ref SECOND: HashMap<char, char> = HashMap::from([
        ('a', 'á'),
        ('o', 'ó'),
        ('e', 'é'),
        ('i', 'í'),
        ('u', 'ú'),
        ('v', 'ǘ')
    ]);
    static ref THIRD: HashMap<char, char> = HashMap::from([
        ('a', 'ǎ'),
        ('o', 'ǒ'),
        ('e', 'ě'),
        ('i', 'ǐ'),
        ('u', 'ǔ'),
        ('v', 'ǚ')
    ]);
    static ref FOURTH: HashMap<char, char> = HashMap::from([
        ('a', 'à'),
        ('o', 'ò'),
        ('e', 'è'),
        ('i', 'ì'),
        ('u', 'ù'),
        ('v', 'ǜ')
    ]);
    static ref TONES: Vec<&'static HashMap<char, char>> =
        Vec::from([&*NONE, &*FIRST, &*SECOND, &*THIRD, &*FOURTH]);
    static ref LOCAL: HashMap<&'static str, &'static str> = HashMap::from([
        ("yi", "i"),
        ("you", "iu"),
        ("yin", "in"),
        ("ye", "ie"),
        ("ying", "ing"),
        ("wu", "u"),
        ("wen", "un"),
        ("yu", "v"),
        ("yue", "ve"),
        ("yuan", "van"),
        ("yun", "vn"),
        ("ju", "jv"),
        ("jue", "jve"),
        ("juan", "jvan"),
        ("jun", "jvn"),
        ("qu", "qv"),
        ("que", "qve"),
        ("quan", "qvan"),
        ("qun", "qvn"),
        ("xu", "xv"),
        ("xue", "xve"),
        ("xuan", "xvan"),
        ("xun", "xvn"),
        ("shi", "sh"),
        ("si", "s"),
        ("chi", "ch"),
        ("ci", "c"),
        ("zhi", "zh"),
        ("zi", "z"),
        ("ri", "r")
    ]);
    static ref SYMBOLS: HashMap<&'static str, &'static str> = HashMap::from([
        ("a", "ㄚ"),
        ("o", "ㄛ"),
        ("e", "ㄜ"),
        ("er", "ㄦ"),
        ("ai", "ㄞ"),
        ("ei", "ㄟ"),
        ("ao", "ㄠ"),
        ("ou", "ㄡ"),
        ("an", "ㄢ"),
        ("en", "ㄣ"),
        ("ang", "ㄤ"),
        ("eng", "ㄥ"),
        ("ong", "ㄨㄥ"),
        ("i", "ㄧ"),
        ("ia", "ㄧㄚ"),
        ("iao", "ㄧㄠ"),
        ("ie", "ㄧㄝ"),
        ("iu", "ㄧㄡ"),
        ("ian", "ㄧㄢ"),
        ("in", "ㄧㄣ"),
        ("iang", "ㄧㄤ"),
        ("ing", "ㄧㄥ"),
        ("iong", "ㄩㄥ"),
        ("u", "ㄨ"),
        ("ua", "ㄨㄚ"),
        ("uo", "ㄨㄛ"),
        ("uai", "ㄨㄞ"),
        ("ui", "ㄨㄟ"),
        ("uan", "ㄨㄢ"),
        ("un", "ㄨㄣ"),
        ("uang", "ㄨㄤ"),
        ("ueng", "ㄨㄥ"),
        ("uen", "ㄩㄣ"),
        ("v", "ㄩ"),
        ("ve", "ㄩㄝ"),
        ("van", "ㄩㄢ"),
        ("vang", "ㄩㄤ"),
        ("vn", "ㄩㄣ"),
        ("b", "ㄅ"),
        ("p", "ㄆ"),
        ("m", "ㄇ"),
        ("f", "ㄈ"),
        ("d", "ㄉ"),
        ("t", "ㄊ"),
        ("n", "ㄋ"),
        ("l", "ㄌ"),
        ("g", "ㄍ"),
        ("k", "ㄎ"),
        ("h", "ㄏ"),
        ("j", "ㄐ"),
        ("q", "ㄑ"),
        ("x", "ㄒ"),
        ("zh", "ㄓ"),
        ("ch", "ㄔ"),
        ("sh", "ㄕ"),
        ("r", "ㄖ"),
        ("z", "ㄗ"),
        ("c", "ㄘ"),
        ("s", "ㄙ"),
        ("w", "ㄨ"),
        ("y", "ㄧ"),
        ("1", ""),
        ("2", "ˊ"),
        ("3", "ˇ"),
        ("4", "ˋ"),
        ("0", "˙"),
        ("", "")
    ]);
}

pub fn raw_format<'a>(p: &'a Pinyin) -> Cow<'a, str> {
    Cow::Borrowed(p.raw.remove_last_grapheme())
}

pub fn number_format<'a>(p: &'a Pinyin) -> Cow<'a, str> {
    Cow::Borrowed(p.raw)
}

pub fn phonetic_format<'a>(p: &'a Pinyin) -> Cow<'a, str> {
    let mut ret = String::new();

    let mut s = p.raw.to_string();
    if let Some(str) = LOCAL.get(s.remove_last_grapheme()) {
        let c = s.chars().last();
        s = str.to_string();
        if let Some(c) = c {
            s.push(c);
        }
    }
    let len = s.graphemes(true).count();
    let split = if Pinyin::has_initial(&s) {
        ["", s.remove_last_grapheme(), s.last_grapheme()]
    } else {
        let i = if len > 2 && s.starts_with('h') {
            2
        } else {
            1
        };
        [
            s.substring(0, i),
            s.substring(i, len - i - 1),
            s.substring(len - 1, 1),
        ]
    };

    let weak = split[2] == "0";
    if weak {
        ret.push_str(SYMBOLS[split[2]]);
    }
    ret.push_str(SYMBOLS[split[0]]);
    ret.push_str(SYMBOLS[split[1]]);
    if !weak {
        ret.push_str(SYMBOLS[split[2]]);
    }

    Cow::Owned(ret)
}

pub fn unicode_format<'a>(p: &'a Pinyin) -> Cow<'a, str> {
    let s = p.raw;
    let len = s.graphemes(true).count();
    let mut ret = String::new();

    let finale = if Pinyin::has_initial(s) {
        let i = if s.len() > 2 && s.chars().nth(1) == Some('h') {
            2
        } else {
            1
        };
        ret.push_str(s.substring(0, i));
        s.substring(i, len - i - 1)
    } else {
        s.remove_last_grapheme()
    };

    let offset = if OFFSET.contains(&finale) { 1 } else { 0 };
    if offset == 1 {
        ret.push_str(finale.first_grapheme());
    }
    let group = TONES[s
        .chars()
        .last()
        .and_then(|c| c.to_digit(10))
        .map(|x| x as usize)
        .unwrap_or(0)];
    if let Some(c) = finale.chars().nth(offset) {
        if let Some(tone) = group.get(&c) {
            ret.push(*tone);
        }
    }
    let finale_len = finale.graphemes(true).count();
    if finale_len > offset + 1 {
        ret.push_str(finale.substring(offset + 1, finale_len));
    }

    Cow::Owned(ret)
}
