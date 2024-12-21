use std::cell::UnsafeCell;
use std::mem::MaybeUninit;

pub const PAGE_LEN_BITS: usize = 10;
pub const PAGE_LEN_MASK: usize = PAGE_LEN - 1;
pub const PAGE_LEN: usize = 1 << PAGE_LEN_BITS;
pub const MAX_PAGES: usize = 1 << (32 - PAGE_LEN_BITS);

pub struct Page<T> {
    allocated: usize,
    data: Box<[UnsafeCell<MaybeUninit<T>>; PAGE_LEN]>,
}

impl<T> Default for Page<T> {
    fn default() -> Self {
        Self {
            allocated: 0,
            data: Box::new([const { UnsafeCell::new(MaybeUninit::uninit()) }; PAGE_LEN]),
        }
    }
}

impl<T> Page<T> {
    pub fn alloc(&mut self, value: T) -> Result<Slot, T> {
        let slot = self.allocated;
        if slot == PAGE_LEN {
            return Err(value);
        }

        let data = &self.data[slot];
        unsafe { (*data.get()).write(value) };
        self.allocated += 1;

        Ok(Slot::new(slot))
    }
}

impl<T: 'static> std::ops::Index<Slot> for Page<T> {
    type Output = T;

    fn index(&self, slot: Slot) -> &Self::Output {
        assert!(slot.as_usize() < self.allocated);
        unsafe { (*self.data[slot.as_usize()].get()).assume_init_ref() }
    }
}

impl<T> Drop for Page<T> {
    fn drop(&mut self) {
        unsafe {
            let to_drop =
                std::slice::from_raw_parts_mut(self.data.as_mut_ptr().cast::<T>(), self.allocated);
            std::ptr::drop_in_place(to_drop)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Slot(u32);

impl Slot {
    pub fn new(raw: usize) -> Self {
        assert!(raw < PAGE_LEN);
        Self(raw as u32)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
