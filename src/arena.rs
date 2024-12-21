use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::page::{PAGE_LEN_BITS, PAGE_LEN_MASK, Slot};
use crate::table::{PageIndex, Table};

#[derive(Default)]
pub struct Arena {
    table: Table,
    most_recent_pages: HashMap<TypeId, PageIndex, NoOpHash>,
}

impl<T: 'static> std::ops::Index<Idx<T>> for Arena {
    type Output = T;

    fn index(&self, index: Idx<T>) -> &Self::Output {
        let (page, slot) = index.split();
        &self.table.page::<T>(page)[slot]
    }
}

impl Arena {
    pub fn table(&self) -> &Table {
        &self.table
    }

    pub fn alloc<T: Any>(&mut self, mut value: T) -> Idx<T> {
        let mut page = *self
            .most_recent_pages
            .entry(value.type_id())
            .or_insert_with(|| self.table.append_page::<T>());

        loop {
            match self.table.page_mut(page).allocate(value) {
                Ok(slot) => {
                    break Idx::new(page, slot);
                }
                Err(ownership) => {
                    value = ownership;
                    page = self.table.append_page::<T>();
                    self.most_recent_pages.insert(value.type_id(), page);
                    continue;
                }
            }
        }
    }
}

pub struct Idx<T> {
    raw: u32,
    phantom: PhantomData<T>,
}

impl<T: std::hash::Hash> std::hash::Hash for Idx<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T: Ord> Ord for Idx<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T: PartialOrd> PartialOrd for Idx<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<T: Eq> Eq for Idx<T> {}

impl<T: PartialEq> PartialEq for Idx<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T> Copy for Idx<T> {}

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::fmt::Debug for Idx<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut type_name = std::any::type_name::<T>();
        type_name = type_name.split("::").last().unwrap_or(type_name);
        write!(f, "Idx::<{}>({})", type_name, self.raw)
    }
}

impl<T> Idx<T> {
    pub fn new(page: PageIndex, slot: Slot) -> Self {
        Self { raw: page.as_u32() << PAGE_LEN_BITS | slot.as_u32(), phantom: PhantomData }
    }

    pub fn split(self) -> (PageIndex, Slot) {
        let raw = self.raw as usize;
        let slot = raw & PAGE_LEN_MASK;
        let page = raw >> PAGE_LEN_BITS;
        (PageIndex::new(page), Slot::new(slot))
    }
}

#[derive(Clone, Default)]
pub struct NoOpHash;

impl std::hash::BuildHasher for NoOpHash {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher(0)
    }
}

pub struct NoOpHasher(u64);

impl std::hash::Hasher for NoOpHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = bytes.iter().fold(self.0, |hash, b| hash.rotate_left(8).wrapping_add(*b as u64));
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

#[cfg(test)]
mod tests {
    use crate::Arena;

    #[test]
    fn test_arena() {
        let mut arena = Arena::default();

        assert_eq!(arena.table.len(), 0);
        let a = arena.alloc(42);
        assert_eq!(arena.table.len(), 1);
        let b = arena.alloc("42");
        assert_eq!(arena.table.len(), 2);
        let c = arena.alloc(42.0);
        assert_eq!(arena.table.len(), 3);

        assert_eq!(arena[a], 42);
        assert_eq!(arena[b], "42");
        assert_eq!(arena[c], 42.0);

        arena.alloc(40);
        arena.alloc(2);

        assert_eq!(arena.table.len(), 3);
    }
}
