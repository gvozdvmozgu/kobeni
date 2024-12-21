use tany::LTAny;

use crate::page::{MAX_PAGES, Page};

#[derive(Default)]
pub struct Table {
    pages: Vec<LTAny>,
}

impl Table {
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }

    pub fn page<T: 'static>(&self, page: PageIndex) -> &Page<T> {
        self.pages[page.as_usize()].downcast_ref().unwrap()
    }

    pub fn page_mut<T: 'static>(&mut self, page: PageIndex) -> &mut Page<T> {
        self.pages[page.as_usize()].downcast_mut().unwrap()
    }

    pub fn append_page<T: 'static>(&mut self) -> PageIndex {
        let page = self.pages.len();
        self.pages.push(LTAny::new(Page::<T>::default()));
        PageIndex::new(page)
    }
}

#[derive(Clone, Copy)]
pub struct PageIndex(u32);

impl PageIndex {
    pub fn new(raw: usize) -> Self {
        assert!(raw < MAX_PAGES);
        Self(raw as u32)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
