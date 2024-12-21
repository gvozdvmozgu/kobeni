#![warn(unused_qualifications)]

mod arena;
mod page;
mod table;

pub use arena::{Arena, Idx};
pub use page::{MAX_PAGES, PAGE_LEN, PAGE_LEN_BITS, PAGE_LEN_MASK, Page, Slot};
pub use table::{PageIndex, Table};
