#![allow(dead_code)]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod accelerator;
pub mod cache;
pub mod compressed;
pub mod dict_loader;
pub mod elements;
pub mod format;
pub mod keyboard;
pub mod pinin;
pub mod searcher;
pub mod unicode_utils;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::rc::Rc;
    use crate::format::{number_format, phonetic_format, raw_format, unicode_format};
    use crate::keyboard::{KEYBOARD_DAQIAN, KEYBOARD_XIAOHE, KEYBOARD_ZIRANMA};
    use crate::pinin::PinIn;
    use pretty_assertions::assert_str_eq;
    use crate::accelerator::Accelerator;
    use crate::searcher::{Searcher, SearcherLogic, SimpleSearcher, TreeSearcher};

    #[test]
    fn quanpin() {
        let mut pinin = PinIn::new();
        pinin.load_dict(Box::new(include_str!("dict.txt")));
        pinin.accelerate = true;
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
        let py = &ch.pinyin[0];

        assert_str_eq!(number_format(py), "yuan2");
        assert_str_eq!(raw_format(py), "yuan");
        assert_str_eq!(unicode_format(py), "yuán");
        assert_str_eq!(phonetic_format(py), "ㄩㄢˊ");
    }

    #[test]
    pub fn full() {
        let mut pinin = PinIn::new();
        pinin.load_dict(Box::new(include_str!("dict.txt")));

        let mut ss: Vec<Box<dyn Searcher<i32>>> = vec![
            Box::new(TreeSearcher::new(SearcherLogic::Equal, Rc::new(Accelerator::new()))),
            Box::new(SimpleSearcher::new(SearcherLogic::Equal))
        ];

        ss.iter_mut().for_each(|searcher| {
            searcher.insert(&pinin, "测试文本", 1);
            searcher.insert(&pinin, "测试切分", 5);
            searcher.insert(&pinin, "测试切分文本", 6);
            searcher.insert(&pinin, "合金炉", 2);
            searcher.insert(&pinin, "洗矿场", 3);
            searcher.insert(&pinin, "流体", 4);
            searcher.insert(&pinin, "轰20", 7);
            searcher.insert(&pinin, "hong2", 8);
            searcher.insert(&pinin, "月球", 9);
            searcher.insert(&pinin, "汉化", 10);
            searcher.insert(&pinin, "喊话", 11);
        });

        ss.iter().for_each(|searcher| {

            let list = searcher.search(&pinin, "hong2");
            pretty_assertions::assert_eq!(list.len(), 1);
            assert!(list.contains(&&8));

            let list = searcher.search(&pinin, "hong20");
            pretty_assertions::assert_eq!(list.len(), 1);
            assert!(list.contains(&&7));

            let list = searcher.search(&pinin, "ceshqf");
            pretty_assertions::assert_eq!(list.len(), 1);
            assert!(list.contains(&&5));

            let list = searcher.search(&pinin, "ceshqfw");
            pretty_assertions::assert_eq!(list.len(), 0);

            let list = searcher.search(&pinin, "hh");
            pretty_assertions::assert_eq!(list.len(), 2);
            assert!(list.contains(&&10));
            assert!(list.contains(&&11));

            let list = searcher.search(&pinin, "hhu");
            pretty_assertions::assert_eq!(list.len(), 0);
        });
    }

    #[test]
    pub fn dataset() {

        const SMALL: &str = include_str!("../benches/small");
        const LARGE: &str = include_str!("../benches/small");

        let mut context = PinIn::new();
        {
            let time = std::time::Instant::now();
            context.load_dict(Box::new(include_str!("dict.txt")));
            println!("load dict took {:?}", std::time::Instant::now() - time);
        }

        let mut searcher = TreeSearcher::new(SearcherLogic::Begin, context.accelerator.clone().unwrap());

        let lines: Vec<_> = SMALL.lines().collect();

        {
            let time = std::time::Instant::now();
            lines.iter().enumerate().for_each(|(i, s)| {
                searcher.insert(&context, s, i);
            });
            println!("build tree {:?}", std::time::Instant::now() - time);
        }
    }

}
