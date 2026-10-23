//! A small-block allocator for the executables that run the engine (the command line tool and
//! the browser modules). Opt in with `#[global_allocator]` in the binary; the library itself
//! never installs it.
//!
//! The interpreter creates and drops a very large number of tiny objects (every unit result
//! is a few small blocks). The system allocator spends about 40 % of the run time of a heavy
//! worksheet there, so blocks up to 512 bytes come from per-thread free lists, carved out of
//! 64 KB chunks taken from the system allocator. Freed blocks go back onto the list of the
//! freeing thread (a block may move between threads). Memory is not returned to the system
//! before the process ends, which suits a program that converts one worksheet and exits, and
//! a browser module that is torn down with its worker.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::ptr;

const GRAIN: usize = 16;
const MAX_SMALL: usize = 512;
const CLASSES: usize = MAX_SMALL / GRAIN;
const CHUNK: usize = 64 * 1024;

struct Heap {
    free: [*mut u8; CLASSES],
    bump: *mut u8,
    end: *mut u8,
}

thread_local! {
    static HEAP: UnsafeCell<Heap> = const {
        UnsafeCell::new(Heap { free: [ptr::null_mut(); CLASSES], bump: ptr::null_mut(), end: ptr::null_mut() })
    };
}

pub struct FastAlloc;

#[inline]
fn is_small(layout: &Layout) -> bool {
    layout.size() <= MAX_SMALL && layout.align() <= GRAIN
}

#[inline]
fn class_of(size: usize) -> usize {
    // Size 0 is treated as one grain; classes are 1..=CLASSES, stored at index class - 1.
    (size.max(1) + GRAIN - 1) / GRAIN - 1
}

unsafe impl GlobalAlloc for FastAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !is_small(&layout) {
            return System.alloc(layout);
        }
        let c = class_of(layout.size());
        match HEAP.try_with(|h| {
            let h = &mut *h.get();
            let head = h.free[c];
            if !head.is_null() {
                h.free[c] = *(head as *mut *mut u8);
                return head;
            }
            let sz = (c + 1) * GRAIN;
            if (h.end as usize) - (h.bump as usize) < sz {
                let chunk = System.alloc(Layout::from_size_align_unchecked(CHUNK, GRAIN));
                if chunk.is_null() {
                    return chunk;
                }
                h.bump = chunk;
                h.end = chunk.add(CHUNK);
            }
            let p = h.bump;
            h.bump = p.add(sz);
            p
        }) {
            Ok(p) => p,
            // Thread teardown: fall back to a block of the class size from the system.
            Err(_) => System.alloc(Layout::from_size_align_unchecked((c + 1) * GRAIN, GRAIN)),
        }
    }

    #[inline]
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        if !is_small(&layout) {
            return System.dealloc(p, layout);
        }
        let c = class_of(layout.size());
        // During thread teardown the block is leaked, which is harmless.
        let _ = HEAP.try_with(|h| {
            let h = &mut *h.get();
            *(p as *mut *mut u8) = h.free[c];
            h.free[c] = p;
        });
    }

    #[inline]
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_small = is_small(&layout);
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
        let new_small = is_small(&new_layout);
        if !old_small && !new_small {
            return System.realloc(p, layout, new_size);
        }
        if old_small && new_small && class_of(layout.size()) == class_of(new_size) {
            return p;
        }
        let n = self.alloc(new_layout);
        if !n.is_null() {
            ptr::copy_nonoverlapping(p, n, layout.size().min(new_size));
            self.dealloc(p, layout);
        }
        n
    }
}
