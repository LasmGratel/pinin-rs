use std::cell::{Cell, RefCell};
use crate::accelerator::{Accelerator, CharProvider};
use crate::compressed::{Compressor, IndexSet};
use crate::pinin::PinIn;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use smallvec::{Array, SmallVec};

use crate::elements::{Phoneme, Pinyin};

pub trait Searcher<T: 'static> {
    fn insert(&mut self, context: &PinIn, name: &str, id: T);
    fn search(&self, context: &PinIn, s: &str) -> Vec<&T>;
    fn reset(&mut self, context: &PinIn);
}

pub trait Collection<T> {
    fn insert(&mut self, data: T);
}

impl<T> Collection<T> for HashSet<T> where T: Eq + Hash {
    fn insert(&mut self, data: T) {
        self.insert(data);
    }
}

impl<T> Collection<T> for BTreeSet<T> where T: Eq + Ord {
    fn insert(&mut self, data: T) {
        self.insert(data);
    }
}

impl<T> Collection<T> for Vec<T> {
    fn insert(&mut self, data: T) {
        self.push(data);
    }
}

impl<A> Collection<A::Item> for SmallVec<A> where A: Array {
    fn insert(&mut self, data: A::Item) {
        self.push(data);
    }
}



pub struct SimpleSearcher<T> {
    objects: Vec<T>,
    accelerator: Accelerator,
    compressor: Rc<RefCell<Compressor>>,
    logic: SearcherLogic,
}

impl<T> Searcher<T> for SimpleSearcher<T> where T: 'static {
    fn insert(&mut self, context: &PinIn, name: &str, id: T) {
        self.compressor.borrow_mut().push(name);
        name.chars().for_each(|c| {
            context.get_character(c);
        });

        self.objects.push(id);
    }

    fn search(&self, context: &PinIn, s: &str) -> Vec<&T> {
        self.accelerator.search(s);
        let offsets = &self.compressor.borrow().offsets;
        offsets
            .iter()
            .enumerate()
            .filter(|(_i, s)| self.logic.test_accelerator(&self.accelerator, context, 0, **s))
            .map(|(i, _)| &self.objects[i])
            .collect()
    }

    fn reset(&mut self, _context: &PinIn) {
        self.accelerator.reset();
    }
}

impl<T> SimpleSearcher<T> {
    pub fn new(logic: SearcherLogic) -> Self {
        let compressor = Rc::new(RefCell::new(Compressor::default()));
        let accelerator = Accelerator::new();
        *accelerator.provider.borrow_mut() = Some(compressor.clone() as Rc<RefCell<dyn CharProvider>>);
        SimpleSearcher {
            objects: Vec::new(),
            accelerator,
            compressor,
            logic
        }
    }

}

const BTREE_THRESHOLD: usize = 256;

pub trait Node<T> where T: 'static {
    fn get_offset(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>, offset: usize);

    fn get(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>);

    fn put(self: Rc<Self>, context: &PinIn, p: &TreeSearcher<T>, name: usize, id: usize) -> Rc<dyn Node<T>>;
}

pub struct TreeSearcher<T> where T: 'static {
    root: Rc<dyn Node<T>>,

    objects: Vec<T>,
    naccs: RefCell<Vec<Rc<NAcc<T>>>>,

    accelerator: Rc<Accelerator>,
    pub(crate) compressor: Rc<RefCell<Compressor>>,
    logic: SearcherLogic
}

impl<T> Searcher<T> for TreeSearcher<T> where T: 'static {
    fn reset(&mut self, context: &PinIn) {
        self.naccs.borrow().iter().for_each(|i| i.reload(context));
        self.accelerator.reset();
    }

    fn insert(&mut self, context: &PinIn, name: &str, id: T) {
        let pos = self.compressor.borrow_mut().push(name);
        let end = if self.logic == SearcherLogic::Contain { name.chars().count() } else { 1 };
        for i in 0..end {
            self.root = self.root.clone().put(context, self, pos + i, self.objects.len());
        }

        self.objects.push(id);
    }

    fn search(&self, context: &PinIn, s: &str) -> Vec<&T> {
        self.accelerator.search(s);
        let mut ret: SmallVec<[usize; 16]> = SmallVec::new();
        self.root.get_offset(context, self, &mut ret, 0);
        ret.into_iter().collect::<HashSet<_>>().into_iter().map(|i| &self.objects[i]).collect()
    }
}

impl<T> TreeSearcher<T> where T: 'static {
    pub fn new(logic: SearcherLogic, accelerator: Rc<Accelerator>) -> Self {
        let compressor = Rc::new(RefCell::new(Compressor::default()));
        let _ = accelerator.provider.borrow_mut().insert(compressor.clone());
        TreeSearcher {
            logic,
            root: Rc::new(NDense::new()),
            objects: Vec::new(),
            naccs: RefCell::new(Vec::new()),
            accelerator,
            compressor,
        }
    }

}

pub struct NMap<T> where T: 'static {

    #[allow(clippy::type_complexity)]
    children: RefCell<Option<HashMap<char, Rc<dyn Node<T>>>>>,

    leaves: RefCell<HashSet<usize>>,
}

impl<T> Default for NMap<T> {
    fn default() -> Self {
        NMap::new()
    }
}

impl<T> NMap<T> where T: 'static {
    pub fn new() -> Self {
        NMap {
            children: RefCell::new(None),
            leaves: RefCell::new(Default::default()),
        }
    }

    pub fn init(&self) {
        self.children.borrow_mut().get_or_insert_with(Default::default);
    }

    pub fn put_char(&self, ch: char, node: Rc<dyn Node<T>>) {
        self.init();
        self.children.borrow_mut().as_mut().unwrap().insert(ch, node);
    }
}

impl<T> Node<T> for NMap<T> {
    fn get_offset(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>, offset: usize) {
        if p.accelerator.search_string.borrow().chars().count() == offset {
            if p.logic == SearcherLogic::Equal {
                self.leaves.borrow().iter().copied().for_each(|x| { ret.insert(x); });
            }
        } else if let Some(children) = &*self.children.borrow() {
            children.iter().for_each(|(key, value)| {
                p.accelerator.get(context, *key, offset)
                    .for_each(|i| value.get_offset(context, p, ret, offset + i as usize));
            });
        }
    }

    fn get(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>) {
        self.leaves.borrow().iter().copied().for_each(|leaf| { ret.insert(leaf); });

        if let Some(children) = &*self.children.borrow() {
            children.values().for_each(|node| node.get(context, p, ret));
        }
    }

    fn put(self: Rc<Self>, context: &PinIn, p: &TreeSearcher<T>, name: usize, id: usize) -> Rc<dyn Node<T>> {
        if p.compressor.borrow().chars[name] == '\0' {
            // TODO Check and replace to BTree
            if self.leaves.borrow().len() >= BTREE_THRESHOLD {

            }

            self.leaves.borrow_mut().insert(id);
        } else {
            self.init();

            let ch = p.compressor.borrow().chars[name];
            if !self.children.borrow().as_ref().unwrap().contains_key(&ch) {
                self.put_char(ch, Rc::new(NDense::new()));
            }

            {
                let mut map = self.children.borrow_mut();

                let node = map.as_mut().unwrap().get_mut(&ch).unwrap();
                *node = node.clone().put(context, p, name + 1, id);
            }
        }

        if self.children.borrow().as_ref().map(|x| x.len() > 32).unwrap_or_default() {
            NAcc::new(context, p, self)
        } else {
            self
        }
    }
}

pub struct NAcc<T> where T: 'static {
    map: Rc<NMap<T>>,
    index: RefCell<HashMap<Phoneme, HashSet<char>>>,
}

impl<T> NAcc<T> where T: 'static {
    pub fn new(context: &PinIn, searcher: &TreeSearcher<T>, map: Rc<NMap<T>>) -> Rc<Self> {
        let acc = Rc::new(NAcc {
            map,
            index: RefCell::new(Default::default())
        });

        acc.reload(context);

        searcher.naccs.borrow_mut().push(acc.clone());
        acc
    }

    fn index(&self, context: &PinIn, c: char) {
        let ch = context.get_character(c);

        ch.pinyin.iter().for_each(|py: &Rc<Pinyin>| {
            let key = &py.phonemes[0];
            let mut index = self.index.borrow_mut();
            if let Some(_value) = index.get(key) {
                //if value.len() >= BTREE_THRESHOLD && !value.contains(&c) {
                    // _index[key] = new HashSet<char>(value); // Should be CharOpenHashSet
                //}
            } else {
                index.insert(key.clone(), HashSet::new());
            }

            index.get_mut(key).unwrap().insert(c);
        });
    }

    pub fn reload(&self, context: &PinIn) {
        self.index.borrow_mut().clear();
        self.map.children.borrow_mut().get_or_insert_with(Default::default);
        self.map.children.borrow().as_ref().unwrap().keys().copied().for_each(|i| {
            self.index(context, i);
        });
    }
}

impl<T: 'static> Node<T> for NAcc<T> {
    fn get_offset(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>, offset: usize) {
        if p.accelerator.search_string.borrow().chars().count() == offset {
            if p.logic == SearcherLogic::Equal {
                self.map.leaves.borrow().iter().copied().for_each(|x| { ret.insert(x); });
            } else {
                self.get(context, p, ret);
            }
        } else {
            if let Some(children) = self.map.children.borrow().as_ref() {
                if let Some(node) = children.get(&p.accelerator.search_string.borrow().chars().nth(offset).unwrap()) {
                    node.get_offset(context, p, ret, offset + 1);
                }
            }

            self.index.borrow().iter()
                .filter(|(key, _value)| key.match_string(p.accelerator.search_string.borrow().as_str(), offset, true) != IndexSet::none())
                .flat_map(|(_, value)| value)
                .copied()
                .for_each(|c| {
                    p.accelerator.get(context, c, offset)
                        .for_each(|j| {
                            if let Some(children) = self.map.children.borrow().as_ref() {
                                children[&c].get_offset(context, p, ret, offset + j as usize);
                            }
                        })
                });
        }
    }

    fn get(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>) {
        self.map.leaves.borrow().iter().copied().for_each(|leaf| { ret.insert(leaf); });

        if let Some(children) = &*self.map.children.borrow() {
            children.values().for_each(|node| node.get(context, p, ret));
        }
    }

    fn put(self: Rc<Self>, context: &PinIn, p: &TreeSearcher<T>, name: usize, id: usize) -> Rc<dyn Node<T>> {
        let _ = self.map.clone().put(context, p, name, id);
        self.index(context, p.compressor.borrow().chars[name]);

        self
    }
}

#[derive(Debug)]
pub struct NDense<T> {
    data: RefCell<Vec<usize>>,
    phantom: PhantomData<T>,
}

impl<T> Default for NDense<T> {
    fn default() -> Self {
        NDense::new()
    }
}

impl<T> NDense<T> {
    pub fn new() -> Self {
        NDense {
            data: RefCell::new(Default::default()),
            phantom: PhantomData::default(),
        }
    }
}


impl<T> Node<T> for NDense<T> where T: 'static {
    fn get_offset(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>, offset: usize) {
        let full = p.logic == SearcherLogic::Equal;
        if full && p.accelerator.search_string.borrow().chars().count() == offset {
            self.get(context, p, ret);
        } else {
            for i in 0..self.data.borrow().len() / 2 {
                let ch = self.data.borrow()[i * 2];
                if full {
                    if p.accelerator.matches(context, offset, ch) {
                        ret.insert(self.data.borrow()[i * 2 + 1]);
                    }
                } else if p.accelerator.begins(context, offset, ch) {
                    ret.insert(self.data.borrow()[i * 2 + 1]);
                }
            }
        }
    }

    fn get(&self, _context: &PinIn, _p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>) {
        for i in 0..self.data.borrow().len() / 2 {
            ret.insert(self.data.borrow()[i * 2 + 1]);
        }
    }

    fn put(self: Rc<Self>, context: &PinIn, p: &TreeSearcher<T>, name: usize, id: usize) -> Rc<dyn Node<T>> {
        if self.data.borrow().len() >= BTREE_THRESHOLD {
            let pattern = self.data.borrow()[0];
            let ret = Rc::new(NSlice::new(pattern, pattern + self.match_tree(p)));
            for j in 0..self.data.borrow().len() / 2 {
                ret.clone().put(context, p, self.data.borrow()[j * 2], self.data.borrow()[j * 2 + 1]);
            }
            ret.clone().put(context, p, name, id);
            ret
        } else {
            self.data.borrow_mut().push(name);
            self.data.borrow_mut().push(id);
            self
        }
    }
}

impl<T> NDense<T> {
    pub fn match_tree(&self, searcher: &TreeSearcher<T>) -> usize {
        let mut i = 0;
        loop {
            let a = searcher.compressor.borrow().chars[self.data.borrow()[0] + i];
            for j in 1..self.data.borrow().len() / 2 {
                let b = searcher.compressor.borrow().chars[self.data.borrow()[j * 2] + i];
                if a != b || a == '\0' {
                    return i;
                }
            }

            i += 1;
        }
    }
}

pub struct NSlice<T> where T: 'static {
    exit: RefCell<Rc<dyn Node<T>>>,
    start: usize,
    end: Cell<usize>,
}

impl<T> NSlice<T> where T: 'static {
    pub fn new(start: usize, end: usize) -> Self {
        NSlice {
            start,
            end: Cell::new(end),
            exit: RefCell::new(Rc::new(NMap::new()))
        }
    }

    pub fn get_slice(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>, offset: usize, start: usize) {
        if self.start + start == self.end.get() {
            self.exit.borrow().get_offset(context, p, ret, offset);
        } else if offset == p.accelerator.search_string.borrow().chars().count() {
            if p.logic != SearcherLogic::Equal {
                self.exit.borrow().get(context, p, ret);
            }
        } else {
            let ch = p.compressor.borrow().chars[self.start + start];
            p.accelerator.get(context, ch, offset).for_each(|i| {
                self.get_slice(context, p, ret, offset + i as usize, start + 1);
            });
        }
    }

    pub fn cut(&self, p: &TreeSearcher<T>, offset: usize) {
        let insert = Rc::new(NMap::new());
        if offset + 1 == self.end.get() {
            insert.put_char(p.compressor.borrow().chars[offset], self.exit.borrow().clone());
        } else {
            let half = Rc::new(NSlice::new(offset + 1, self.end.get()));
            *half.exit.borrow_mut() = self.exit.borrow().clone();

            insert.put_char(p.compressor.borrow().chars[offset], half);
        }

        *self.exit.borrow_mut() = insert;
        self.end.set(offset);
    }
}

impl<T> Node<T> for NSlice<T> where T: 'static {
    fn get_offset(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>, offset: usize) {
        self.get_slice(context, p, ret, offset, 0);
    }

    fn get(&self, context: &PinIn, p: &TreeSearcher<T>, ret: &mut dyn Collection<usize>) {
        self.exit.borrow().get(context, p, ret);
    }

    fn put(self: Rc<Self>, context: &PinIn, p: &TreeSearcher<T>, name: usize, id: usize) -> Rc<dyn Node<T>> {
        let len = self.end.get() - self.start;
        let matched = p.accelerator.common(self.start, name, len);
        if matched >= len {
            let node = self.exit.borrow().clone().put(context, p, name + len, id);
            *self.exit.borrow_mut() = node;
        } else {
            self.cut(p, self.start + matched);

            let node = self.exit.borrow().clone().put(context, p, name + matched, id);
            *self.exit.borrow_mut() = node;
        }

        if self.start == self.end.get() {
            self.exit.borrow().clone()
        } else {
            self
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
    pub fn test_accelerator(&self, a: &Accelerator, context: &PinIn, offset: usize, start: usize) -> bool {
        match *self {
            SearcherLogic::Begin => a.begins(context, offset, start),
            SearcherLogic::Contain => a.contains(context, offset, start),
            SearcherLogic::Equal => a.matches(context, offset, start),
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
