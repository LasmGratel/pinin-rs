use crate::elements::Pinyin;
use crate::unicode_utils::UnicodeUtils;
use lazy_static::lazy_static;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

lazy_static! {
    static ref DAQIAN_KEYS: HashMap<&'static str, &'static str> = HashMap::from([
        ("", ""),
        ("0", ""),
        ("1", " "),
        ("2", "6"),
        ("3", "3"),
        ("4", "4"),
        ("a", "8"),
        ("ai", "9"),
        ("an", "0"),
        ("ang", ";"),
        ("ao", "l"),
        ("b", "1"),
        ("c", "h"),
        ("ch", "t"),
        ("d", "2"),
        ("e", "k"),
        ("ei", "o"),
        ("en", "p"),
        ("eng", "/"),
        ("er", "-"),
        ("f", "z"),
        ("g", "e"),
        ("h", "c"),
        ("i", "u"),
        ("ia", "u8"),
        ("ian", "u0"),
        ("iang", "u;"),
        ("iao", "ul"),
        ("ie", "u,"),
        ("in", "up"),
        ("ing", "u/"),
        ("iong", "m/"),
        ("iu", "u."),
        ("j", "r"),
        ("k", "d"),
        ("l", "x"),
        ("m", "a"),
        ("n", "s"),
        ("o", "i"),
        ("ong", "j/"),
        ("ou", "."),
        ("p", "q"),
        ("q", "f"),
        ("r", "b"),
        ("s", "n"),
        ("sh", "g"),
        ("t", "w"),
        ("u", "j"),
        ("ua", "j8"),
        ("uai", "j9"),
        ("uan", "j0"),
        ("uang", "j;"),
        ("uen", "mp"),
        ("ueng", "j/"),
        ("ui", "jo"),
        ("un", "jp"),
        ("uo", "ji"),
        ("v", "m"),
        ("van", "m0"),
        ("vang", "m;"),
        ("ve", "m,"),
        ("vn", "mp"),
        ("w", "j"),
        ("x", "v"),
        ("y", "u")
    ]);
    static ref XIAOHE_KEYS: HashMap<&'static str, &'static str> = HashMap::from([
        ("ai", "d"),
        ("an", "j"),
        ("ang", "h"),
        ("ao", "c"),
        ("ch", "i"),
        ("ei", "w"),
        ("en", "f"),
        ("eng", "g"),
        ("ia", "x"),
        ("ian", "m"),
        ("iang", "l"),
        ("iao", "n"),
        ("ie", "p"),
        ("in", "b"),
        ("ing", "k"),
        ("iong", "s"),
        ("iu", "q"),
        ("ong", "s"),
        ("ou", "z"),
        ("sh", "u"),
        ("ua", "x"),
        ("uai", "k"),
        ("uan", "r"),
        ("uang", "l"),
        ("ui", "v"),
        ("un", "y"),
        ("uo", "o"),
        ("ve", "t"),
        ("ue", "t"),
        ("vn", "y")
    ]);
    static ref ZIRANMA_KEYS: HashMap<&'static str, &'static str> = HashMap::from([
        ("ai", "l"),
        ("an", "j"),
        ("ang", "h"),
        ("ao", "k"),
        ("ch", "i"),
        ("ei", "z"),
        ("en", "f"),
        ("eng", "g"),
        ("ia", "w"),
        ("ian", "m"),
        ("iang", "d"),
        ("iao", "c"),
        ("ie", "x"),
        ("in", "n"),
        ("ing", "y"),
        ("iong", "s"),
        ("iu", "q"),
        ("ong", "s"),
        ("ou", "b"),
        ("sh", "u"),
        ("ua", "w"),
        ("uai", "y"),
        ("uan", "r"),
        ("uang", "d"),
        ("ui", "v"),
        ("un", "p"),
        ("uo", "o"),
        ("ve", "t"),
        ("ue", "t"),
        ("vn", "p"),
        ("zh", "v")
    ]);
    static ref PHONETIC_LOCAL_KEYS: HashMap<&'static str, &'static str> = HashMap::from([
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
    pub static ref KEYBOARD_QUANPIN: Keyboard = Keyboard {
        local: None,
        keys: None,
        cutter: standard_cutter,
        duo: false,
        sequence: true,
    };
    pub static ref KEYBOARD_DAQIAN: Keyboard = Keyboard {
        local: Some(&PHONETIC_LOCAL_KEYS),
        keys: Some(&DAQIAN_KEYS),
        cutter: standard_cutter,
        duo: false,
        sequence: false,
    };
    pub static ref KEYBOARD_XIAOHE: Keyboard = Keyboard {
        local: None,
        keys: Some(&XIAOHE_KEYS),
        cutter: zero_cutter,
        duo: true,
        sequence: false,
    };
    pub static ref KEYBOARD_ZIRANMA: Keyboard = Keyboard {
        local: None,
        keys: Some(&ZIRANMA_KEYS),
        cutter: zero_cutter,
        duo: true,
        sequence: false,
    };
}

pub struct Keyboard {
    local: Option<&'static HashMap<&'static str, &'static str>>,
    keys: Option<&'static HashMap<&'static str, &'static str>>,
    cutter: fn(&str) -> SmallVec<[&str; 4]>,
    pub duo: bool,
    pub sequence: bool,
}

impl Keyboard {
    pub fn keys<'a>(&self, s: &'a str) -> &'a str {
        self.keys.and_then(|keys| keys.get(s)).unwrap_or(&s)
    }

    pub fn keys_cow<'a>(&self, s: Cow<'a, str>) -> Cow<'static, str> {
        self.keys
            .and_then(|keys| keys.get(s.as_ref()))
            .map(|x| Cow::Borrowed(*x))
            .unwrap_or_else(|| Cow::Owned(s.into_owned()))
    }

    pub fn split<'a, 'b>(&'a self, s: &'b str) -> SmallVec<[Cow<'b, str>; 4]> {
        if let Some(local) = self.local {
            let s = s;

            let cut = s.remove_last_grapheme();
            if let Some(alt) = local.get(cut) {
                let mut sx = alt.to_string();
                sx.push_str(s.last_grapheme());
                return (self.cutter)(&sx)
                    .into_iter()
                    .map(|x| Cow::Owned(x.to_string()))
                    .collect();
            }
            (self.cutter)(s)
                .into_iter()
                .map(Cow::Borrowed)
                .collect()
        } else {
            (self.cutter)(s)
                .into_iter()
                .map(Cow::Borrowed)
                .collect()
        }
    }
}

fn standard_cutter<'a>(s: &'a str) -> SmallVec<[&'a str; 4]> {
    let mut cursor = 0usize;
    let mut ret = SmallVec::new();
    let graphemes: SmallVec<[(usize, &'a str); 7]> = s.grapheme_indices(true).collect();
    if Pinyin::has_initial(s) {
        cursor = if graphemes.len() > 2 && graphemes[1].1 == "h" {
            2
        } else {
            1
        };
        ret.push(s.substring(0, cursor));
    }

    if graphemes.len() != cursor + 1 {
        ret.push(s.substring(cursor, graphemes.len() - cursor - 1));
    }

    ret.push(s.last_grapheme());

    ret
}

fn zero_cutter(s: &str) -> SmallVec<[&str; 4]> {
    let mut ss = standard_cutter(s);
    if ss.len() != 2 {
        return ss;
    }
    let finale = ss[0];
    ss[0] = &finale[0..1];
    ss[1] = if finale.len() == 2 {
        &finale[1..2]
    } else {
        finale
    };

    ss
}
