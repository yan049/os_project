//! Page Table Entry

use crate::mem::utils::{PhysPageNum, PPN_MASK};

// The format of Sv39 page table entry:
// |  63-54 |  53-28 |  27-19 |  18-10 | 9-8 |7|6|5|4|3|2|1|0|
// | Unused | PPN[2] | PPN[1] | PPN[0] | RSW |D|A|G|U|X|W|R|V|

bitflags::bitflags! {
    pub struct PTEFlags: u8 {
        /// Valid
        const V = 0b0000_0001;
        /// Readable
        const R = 0b0000_0010;
        /// Writable
        const W = 0b0000_0100;
        /// Executable
        const X = 0b0000_1000;
        /// It this page accessible to user mode?
        const U = 0b0001_0000;
        /// Global mappings (are those that exist in all address spaces)
        const G = 0b0010_0000;
        /// Accessed
        const A = 0b0100_0000;
        /// Dirty
        const D = 0b1000_0000;
    }
}


#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    const FLAG_SHIFT: usize = 10;
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << Self::FLAG_SHIFT | flags.bits() as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> Self::FLAG_SHIFT & PPN_MASK).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }
    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }
    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

