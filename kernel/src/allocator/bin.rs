use alloc::heap::{AllocErr, Layout};
use std;
use std::cmp::max;
use std::fmt;

use allocator::linked_list::LinkedList;
use allocator::util::*;

const BINS: usize = 30; // enough for RPi3B of which RAM < 1G
const BIN_OFFSET: usize = 3;
// i.e. minimum bin level
// SHOULD BE >= `log2(size_of::<usize>() / 8)`
// in order for `LinkedList` to work
// minimum size for a block of bin: `2usize.pow(BIN_OFFSET)`
const MAX_ALIGN: usize = 4096; // at most `MAX_ALIGN` bytes free mem is ignored

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    start: usize,
    end: usize,
    len: usize,
    bins: [Option<LinkedList>; BINS],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let start = align_up(start, MAX_ALIGN); // align all free memory
        let mem_size = end - start;
        check_memory(mem_size);
        let mut bins = [None; BINS];
        let mut current = start; // start of mem which is not associated to bins yet
        for level in (BIN_OFFSET..(max_bin_level(mem_size) + 1)).rev() {
            let mut list = LinkedList::new();
            if end - current >= 2usize.pow(level as u32) {
                unsafe { list.push(current as *mut usize) };
                current += 2usize.pow(level as u32);
            }
            bins[bin_level_to_index(level)] = Some(list);
        }
        if end - current == 2usize.pow(BIN_OFFSET as u32) {
            unsafe { bins[0].expect("p1").push(current as *mut usize) };
        }
        Allocator {
            start: start,
            end: end,
            len: max_bin_level(mem_size) - BIN_OFFSET,
            bins: bins,
        }
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if layout.align() > MAX_ALIGN {
            // only when the whole free mem can be aligned,
            // the result can be guaranteed to be aligned
            // NOTE: if free mem start at 0,
            // then the result will be aligned in any cases automagically
            // but obviously it cannot be meet
            return Err(AllocErr::Unsupported {
                details: "Requested layout alignment size too big. \
                          Tweak MAX_ALIGN constant in bin allocator.",
            });
        }
        let target_level = target_level_from_layout(&layout); // minimum reqired level
        let mut available_level = None;
        for level in target_level..(self.len + BIN_OFFSET + 1) {
            if !self.bin_by_level(level).is_empty() {
                available_level = Some(level);
                break;
            }
        }
        if let Some(available_level) = available_level {
            let mut address = self.bin_by_level(available_level).pop().expect("p4");
            self.bin_by_level(available_level).pop();
            let mut level = available_level;
            while level > target_level {
                // divide the block and push the first half to the bin of `level - 1`
                //  until `level` reachs the target
                level -= 1;
                unsafe { self.bin_by_level(level).push(address) };
                address = unsafe { address.add(2usize.pow(level as u32) / 8) };
            }
            Ok(address as *mut u8) // <del>automagically</del> aligned
        } else {
            Err(AllocErr::Exhausted { request: layout })
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self._dealloc(ptr, target_level_from_layout(&layout))
    }

    #[inline(always)]
    fn _dealloc(&mut self, ptr: *mut u8, level: usize) {
        let address = ptr as usize;
        let offset = self.start;
        let offset_address = address - self.start;
        let mut parent_address = None; // start address of the parental block
        for node in self.bin_by_level(level).iter_mut() {
            if ((((node.value() as usize - offset) >> level) ^ (offset_address >> level)) % 2 == 1)
                && (((node.value() as usize - offset) >> level + 1)
                    == (offset_address >> (level + 1)))
            {
                // TODO: does not merge top two
                // the first and second half of the corresponding higher level
                // block are both free, merge them
                parent_address = Some({
                    let sibling_address = node.pop() as usize;
                    if address < sibling_address {
                        address as *mut u8
                    } else {
                        sibling_address as *mut u8
                    }
                });
                break;
            }
        }
        if let Some(parent_address) = parent_address {
            self._dealloc(parent_address, level + 1)
        } else {
            unsafe { self.bin_by_level(level).push(address as *mut usize) };
        }
    }

    fn bin_by_level(&mut self, level: usize) -> &mut LinkedList {
        self.bins[bin_level_to_index(level)]
            .as_mut()
            .expect("Bin level out of bound")
    }

    #[allow(dead_code)]
    /// Validate there are no overlap between any two bin instances (for debugging).
    fn check_bins(&mut self) {
        let mut bins = [(0, 0); 1000];
        let mut index = 0;
        for l in BIN_OFFSET..(BIN_OFFSET + self.len + 1) {
            for n in self.bin_by_level(l).iter() {
                bins[index] = {
                    let s = n as usize;
                    let e = s + 2usize.pow(l as u32);
                    (s, e)
                };
                index += 1;
                if (n as usize) % min(MAX_ALIGN, 2usize.pow(l as u32)) != 0 {
                    panic!("SHOULD HAVE BEEN ALIGNED: {:?} l{}", n, l);
                }
            }
        }
        let len = index;
        use std::cmp::{max, min};
        let mut a;
        for index in 0..len {
            for indexx in (index + 1)..len {
                if {
                    a = max(
                        0,
                        min(bins[index].1, bins[indexx].1) as i128
                            - max(bins[index].0, bins[indexx].0) as i128,
                    );
                    a != 0
                } {
                    panic!(
                        "BIN Overlaps {}: {:x} l{} - {:x} l{}",
                        a,
                        bins[index].0,
                        log2(bins[index].1 - bins[index].0),
                        bins[indexx].0,
                        log2(bins[indexx].1 - bins[indexx].0)
                    );
                }
            }
        }
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Allocator")
            .field("start", &(self.start as *mut usize))
            .field("end", &(self.end as *mut usize))
            .field("size", &((self.end - self.start) as *mut usize))
            .field("number_of_bins", &self.len)
            .finish()
            .and_then(|_| {
                fmt.write_str("\n").and_then(|_| {
                    fmt.debug_list()
                        .entries(
                            self.bins
                                .iter()
                                .filter(|item| item.is_some())
                                .map(|item| item.unwrap()),
                        ).finish()
                })
            })
    }
}

/// Validate the pre-defined constant meets the reqirements enforced by the size of available memory.
fn check_memory(mem_size: usize) {
    if !(BIN_OFFSET >= log2(std::mem::size_of::<usize>() / 8)) {
        panic!(
            "BIN_OFFSET in bin allocator is too small \
             to fit LinkedList. Tweak it."
        )
    }
    if mem_size < 2usize.pow(BIN_OFFSET as u32) {
        panic!(
            "Memory too small. \
             Tweak the BIN_OFFSET constant in bin allocator."
        );
    }
    if !(bin_level_to_index(max_bin_level(mem_size)) < BINS) {
        panic!(
            "Memory too large ({}). \
             Tweak the BINS constant in bin allocatior.",
            mem_size
        );
    }
}

/// Calc the target level (the least level of bins to request mem from) from layout.
///
/// # Note
/// In order for the automagical alignment to work, the size of mem to request should be
/// `max(layout.align(), layout.size())`
fn target_level_from_layout(layout: &Layout) -> usize {
    max(log2_up(max(layout.size(), layout.align())), 3) // for alignment
}

/// Maximum level of bins can exist at initialization. It is constrained by the size of the whole
/// available mem.
fn max_bin_level(mem_size: usize) -> usize {
    log2(if mem_size.is_power_of_two() {
        mem_size / 2
    } else {
        mem_size
    })
}

fn bin_level_to_index(level: usize) -> usize {
    level - BIN_OFFSET
}

#[allow(dead_code)]
fn bin_index_to_level(index: usize) -> usize {
    index + BIN_OFFSET
}
