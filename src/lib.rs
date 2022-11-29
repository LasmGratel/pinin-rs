mod accelerator;
mod cache;
mod compressed;
pub mod dict_loader;
mod elements;
mod format;
mod keyboard;
mod pinin;
mod searcher;
mod unicode_utils;

#[cfg(test)]
mod tests {

    use crate::format::{number_format, phonetic_format, raw_format, unicode_format};
    use crate::keyboard::{KEYBOARD_DAQIAN, KEYBOARD_XIAOHE, KEYBOARD_ZIRANMA};
    use crate::pinin::PinIn;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn quanpin() {
        let mut pinin = PinIn::new();
        pinin.load_dict(Box::new(include_str!("dict.txt")));

        assert!(pinin.contains("测试文本", "ceshiwenben"));
        assert!(pinin.contains("测试文本", "ceshiwenbe"));
        assert!(pinin.contains("测试文本", "ceshiwben"));
        assert!(pinin.contains("测试文本", "ce4shi4wb"));
        assert!(!pinin.contains("测试文本", "ce2shi4wb"));
        assert!(pinin.contains("合金炉", "hejinlu"));
        assert!(pinin.contains("洗矿场", "xikuangchang"));
        assert!(pinin.contains("流体", "liuti"));
        assert!(pinin.contains("轰20", "hong2"));
        assert!(pinin.contains("hong2", "hong2"));
        assert!(!pinin.begins("测", "ce4a"));
        assert!(!pinin.begins("", "a"));
        assert!(pinin.contains("石头", "stou"));
        assert!(pinin.contains("安全", "aquan"));
        assert!(pinin.contains("昂扬", "ayang"));
        assert!(!pinin.contains("昂扬", "anyang"));
        assert!(pinin.contains("昂扬", "angyang"));
    }

    #[test]
    fn xiaohe() {
        let mut pinin = PinIn::new();
        pinin.keyboard = &KEYBOARD_XIAOHE;
        pinin.load_dict(Box::new(include_str!("dict.txt")));

        assert!(pinin.contains("测试文本", "ceuiwfbf"));
        assert!(pinin.contains("测试文本", "ceuiwf2"));
        assert!(!pinin.contains("测试文本", "ceuiw2"));
        assert!(pinin.contains("合金炉", "hej"));
        assert!(pinin.contains("洗矿场", "xikl4"));
        assert!(pinin.contains("月球", "ytqq"));
    }

    #[test]
    fn ziranma() {
        let mut pinin = PinIn::new();
        pinin.keyboard = &KEYBOARD_ZIRANMA;
        pinin.load_dict(Box::new(include_str!("dict.txt")));

        assert!(pinin.contains("测试文本", "ceuiwfbf"));
        assert!(pinin.contains("测试文本", "ceuiwf2"));
        assert!(!pinin.contains("测试文本", "ceuiw2"));
        assert!(pinin.contains("合金炉", "hej"));
        assert!(pinin.contains("洗矿场", "xikd4"));
        assert!(pinin.contains("月球", "ytqq"));
        assert!(pinin.contains("安全", "anqr"));
    }

    #[test]
    fn daqian() {
        let mut pinin = PinIn::new();
        pinin.keyboard = &KEYBOARD_DAQIAN;
        pinin.load_dict(Box::new(include_str!("dict.txt")));

        assert!(pinin.contains("测试文本", "hk4g4jp61p3"));
        assert!(pinin.contains("测试文本", "hkgjp1"));
        assert!(pinin.contains("錫", "vu6"));
        // FIXME
        // Bug still presents in PininSharp
        //assert!(pinin.contains("鑽石", "yj0"));
        //assert!(pinin.contains("物質", "j456"));
        assert!(pinin.contains("腳手架", "rul3g.3ru84"));
        assert!(pinin.contains("鵝", "k6"));
        assert!(pinin.contains("葉", "u,4"));
        assert!(pinin.contains("共同", "ej/wj/"));
    }

    #[test]
    pub fn format() {
        let mut pinin = PinIn::new();
        pinin.load_dict(Box::new(include_str!("dict.txt")));

        let ch = pinin.chars[&'圆'].as_ref().unwrap();
        let py = &*ch.pinyin[0];

        assert_str_eq!(number_format(py), "yuan2");
        assert_str_eq!(raw_format(py), "yuan");
        assert_str_eq!(unicode_format(py), "yuán");
        assert_str_eq!(phonetic_format(py), "ㄩㄢˊ");

        pinin.format = Box::new(phonetic_format);
        let _temp = pinin.get_or_insert_pinyin("le0");

        //assert_str_eq!((pinin.format)(&*temp), "˙ㄌㄜ");
    }
}
