use std::{cmp::Ordering, collections::{HashMap, HashSet, VecDeque}, fmt::Debug};
use serde::{Deserialize, Serialize};

use crate::{error::PakResult, pointer::{PakPointer, PakTypedPointer, PakUntypedPointer}};

use super::{value::PakValue, Pak, PakBuilder};


//==============================================================================================
//        PakTree
//==============================================================================================

pub struct PakTree<'p> {
    pak : &'p Pak,
    meta : PakTreeMeta,
}

impl <'p> PakTree<'p> {
    pub fn new(pak: &'p Pak, key : &str) -> PakResult<PakTree<'p>> {
        let indices = pak.fetch_indices()?;
        let pointer = indices.get(key).unwrap();
        let meta : PakTreeMeta = pak.read_err(&pointer.as_pointer())?;
        
        Ok(PakTree {
            pak,
            meta,
        })
    }
    
    pub fn get(&self, value : &PakValue) -> PakResult<HashSet<PakTypedPointer>> {
        let pointer = self.meta.pages.get(&0).unwrap();
        let mut set = HashSet::new();
        self.get_r(value, *pointer, &mut set)?;
        Ok(set)
    }
    
    fn get_r(&self, value : &PakValue, current_page : PakUntypedPointer, set : &mut HashSet<PakTypedPointer>) -> PakResult<()> {
        let page : PakTreePage = self.pak.read_err(&current_page.as_pointer())?;
        
        for entry in page.values {
            if &entry.key < value {
                continue;
            } else if &entry.key > value {
                if let Some(index) = entry.previous {
                    let pointer = self.meta.pages.get(&index).unwrap();
                    self.get_r(value, *pointer, set)?;
                    return Ok(());
                }
            } else {
                entry.values.clone().into_iter().for_each(|value| {set.insert(value);});
                return Ok(());
            }
        }
        
        if let Some(index) = page.next {
            let pointer = self.meta.pages.get(&index).unwrap();
            self.get_r(value, *pointer, set)?;
        }
        
        Ok(())
    }
    
    pub fn get_less(&self, value : &PakValue) -> PakResult<HashSet<PakTypedPointer>> {
        let pointer = self.meta.pages.get(&0).unwrap();
        let mut results = HashSet::new();
        self.get_less_r(value, *pointer, &mut results, false)?;
        Ok(results)
    }
    
    pub fn get_less_eq(&self, value : &PakValue) -> PakResult<HashSet<PakTypedPointer>> {
        let pointer = self.meta.pages.get(&0).unwrap();
        let mut results = HashSet::new();
        self.get_less_r(value, *pointer, &mut results, true)?;
        Ok(results)
    }
    
    fn get_less_r(&self, value : &PakValue, current_page : PakUntypedPointer, set : &mut HashSet<PakTypedPointer>, match_eq : bool) -> PakResult<()> {
        let page : PakTreePage = self.pak.read_err(&current_page.as_pointer())?;
        
        for entry in page.values {
            if &entry.key > value {
                continue;
            } else if &entry.key < value {
                entry.values.clone().into_iter().for_each(|value| {set.insert(value);});
                if let Some(index) = entry.previous {
                    let pointer = self.meta.pages.get(&index).unwrap();
                    self.get_less_r(value, *pointer, set, match_eq)?;
                }
                continue;
            } else {
                if match_eq {
                    entry.values.clone().into_iter().for_each(|value| {set.insert(value);});
                }
                continue;
            }
        }
        
        if let Some(index) = page.next {
            let pointer = self.meta.pages.get(&index).unwrap();
            return self.get_less_r(value, *pointer, set, match_eq);
        }
        
        Ok(())
    }
    
    pub fn get_greater(&self, value : &PakValue) -> PakResult<HashSet<PakTypedPointer>> {
        let pointer = self.meta.pages.get(&0).unwrap();
        let mut results = HashSet::new();
        self.get_greater_r(value, *pointer, &mut results, false)?;
        Ok(results)
    }
    
    pub fn get_greater_eq(&self, value : &PakValue) -> PakResult<HashSet<PakTypedPointer>> {
        let pointer = self.meta.pages.get(&0).unwrap();
        let mut results = HashSet::new();
        self.get_greater_r(value, *pointer, &mut results, true)?;
        Ok(results)
    }
    
    fn get_greater_r(&self, value : &PakValue, current_page : PakUntypedPointer, set : &mut HashSet<PakTypedPointer>, match_eq : bool) -> PakResult<()> {
        let page : PakTreePage = self.pak.read_err(&current_page.as_pointer())?;
        
        for entry in page.values {
            if &entry.key < value {
                continue;
            } else if &entry.key > value {
                entry.values.clone().into_iter().for_each(|value| {set.insert(value);});
                if let Some(index) = entry.previous {
                    let pointer = self.meta.pages.get(&index).unwrap();
                    self.get_less_r(value, *pointer, set, match_eq)?;
                }
                continue;
            } else {
                if match_eq {
                    entry.values.clone().into_iter().for_each(|value| {set.insert(value);});
                }
                continue;
            }
        }
        
        if let Some(index) = page.next {
            let pointer = self.meta.pages.get(&index).unwrap();
            return self.get_greater_r(value, *pointer, set, match_eq);
        }
        
        Ok(())
    }
}

//==============================================================================================
//        PakTreeMeta
//==============================================================================================

#[derive(Deserialize, Serialize)]
pub struct PakTreeMeta {
    pages: HashMap<usize, PakUntypedPointer>,
}

//==============================================================================================
//        PakTreeBuilder
//==============================================================================================

/// This is a pretty limited b-tree implementation. Since the end product is a read only datastructure, deletion is not supported. This
/// has been optimized for read performance and memory efficiency.
#[derive(Debug)]
pub struct PakTreeBuilder {
    pages : Vec<PakTreePage>,
    max_size: usize,
}

impl PakTreeBuilder {
    pub fn new(power_of_two: u32) -> Self {
        PakTreeBuilder {
            pages: vec![PakTreePage::new()],
            max_size : 2usize.pow(power_of_two),
        }
    }
    
    pub fn access<'t>(&'t mut self) -> PakTreeBuilderAccess<'t> {
        PakTreeBuilderAccess {
            current: 0,
            table: self,
            trail : VecDeque::new()
        }
    }
    
    pub fn into_pak(self, pak : &mut PakBuilder) -> PakResult<PakPointer> {
        
        let mut page_map = HashMap::<usize, PakUntypedPointer>::new();
        for (index, page) in self.pages.into_iter().enumerate() {
            let pointer = pak.pak_no_search(page)?;
            page_map.insert(index as usize, pointer.as_untyped());
        }
        
        pak.pak_no_search(PakTreeMeta{ pages : page_map})
    } 
}

//==============================================================================================
//        PakTreeAccess
//==============================================================================================

pub struct PakTreeBuilderAccess<'t> {
    table: &'t mut PakTreeBuilder,
    current: usize,
    trail : VecDeque<usize>
}

impl PakTreeBuilderAccess<'_> {    
    // fn root(&mut self) -> &mut Self {
    //     self.current = self.table.root;
    //     self
    // }
    
    fn goto(&mut self, page: usize) -> &mut Self {
        self.trail.push_front(self.current);
        self.current = page;
        self
    }
    
    fn back(&mut self) -> &mut Self {
        let front = self.trail.pop_front();
        if let Some(index) = front {
            self.current = index;
        }
        self
    }
    
    fn current(&self) -> &PakTreePage {
        &self.table.pages[self.current]
    }
    
    fn current_mut(&mut self) -> &mut PakTreePage {
        &mut self.table.pages[self.current]
    }
    
    fn push(&mut self, entry : PakTreePageEntry) -> PakTreeStatus{
        self.current_mut().push(entry)
    }
    
    fn insert_entry(&mut self, entry : PakTreePageEntry) -> usize {
        let mut result = self.push(entry);
        while let PakTreeStatus::Next(index, entry) = result {
            self.goto(index);
            result = self.push(entry);
        }
        
        let index = match result {
            PakTreeStatus::Ok(index) => index,
            PakTreeStatus::Next(_, _) => 0,
        };
        
        if self.current().values.len() > self.table.max_size {
            self.split();
        }
        
        index
    }
    
    pub fn insert<K>(&mut self, key: K, value: PakTypedPointer) -> &mut Self where K: Into<PakValue> {
        self.insert_entry(PakTreePageEntry::new(key.into(), value));
        self
    }
    
    fn split(&mut self) {
        let mut leading_entries = VecDeque::new();
        let mut trailing_entries = VecDeque::new();
        
        let half_max = self.table.max_size / 2;
        
        for _ in 0..half_max {
            let current = self.current_mut();
            leading_entries.push_back(current.values.pop_front().unwrap());
            trailing_entries.push_front(current.values.pop_back().unwrap());
        }
        
        let mut middle_entry = self.current_mut().values.pop_front().unwrap();
        self.current_mut().values = trailing_entries;
        let leading_index = self.new_page(leading_entries);
        middle_entry.previous = Some(leading_index);
        
        self.back();
        
        self.insert_entry(middle_entry);
    }
    
    fn new_page(&mut self, values : VecDeque<PakTreePageEntry>) -> usize {
        let index = self.table.pages.len();
        self.table.pages.push(PakTreePage::new_with_entries(values));
        index
    }
}

//==============================================================================================
//        PakTreePage
//==============================================================================================

#[derive(Debug, Deserialize, Serialize)]
struct PakTreePage {
    values: VecDeque<PakTreePageEntry>,
    next: Option<usize>,
}

impl PakTreePage {
    fn new() -> Self {
        PakTreePage {
            values: VecDeque::new(),
            next: None,
        }
    }
    
    fn new_with_entries(entries: VecDeque<PakTreePageEntry>) -> Self {
        PakTreePage {
            values: entries,
            next: None,
        }
    }
    
    fn push(&mut self, mut e : PakTreePageEntry) -> PakTreeStatus {
        for (index, entry) in self.values.iter_mut().enumerate() {
            match entry.key.cmp(&e.key) {
                Ordering::Less => continue,
                Ordering::Greater => match entry.previous {
                        Some(next) => return PakTreeStatus::Next(next, e),
                        None => {
                            self.values.insert(index, e);
                            return PakTreeStatus::Ok(index);
                        },
                    },
                Ordering::Equal => {
                    entry.values.append(&mut e.values);
                    return PakTreeStatus::Ok(index);
                },
            }
        }
        match self.next {
            Some(index) => PakTreeStatus::Next(index, e),
            None => {
                self.values.push_back(e);
                return PakTreeStatus::Ok(self.values.len() - 1);
            },
        }
    }
}

enum PakTreeStatus {
    Ok(usize),
    Next(usize, PakTreePageEntry),
}

//==============================================================================================
//        PakTreePageEntry
//==============================================================================================

#[derive(Serialize, Deserialize)]
pub struct PakTreePageEntry {
    key: PakValue,
    values: Vec<PakTypedPointer>,
    previous: Option<usize>,
}

impl Debug for PakTreePageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.key.fmt(f)?;
        if let Some(previous) = self.previous {
            f.write_str(" -> ")?;
            write!(f, "{}", previous)?;
        }
        Ok(())
    }
}

impl PakTreePageEntry {
    pub fn new(key: PakValue, value: PakTypedPointer) -> Self {
        PakTreePageEntry {
            key,
            values : vec![value],
            previous: None,
        }
    }
}

impl PartialEq for PakTreePageEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for PakTreePageEntry {}

impl PartialOrd for PakTreePageEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Ord for PakTreePageEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}
