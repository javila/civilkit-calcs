//! Port of the .NET runtime `ArraySortHelper` introspective sort.
//!
//! Calcpad sorts with `Span<T>.Sort()`, `Span<T>.Sort(Comparison<T>)` and the keys/items
//! overloads. Their result for equal keys (and so the output of `order`) depends on the
//! exact algorithm, which is reproduced here: insertion sort for partitions of up to 16
//! items, median of three pivots, and heap sort when the depth limit is reached.
//!
//! Two variants exist in .NET and they differ slightly:
//! * the comparison based `ArraySortHelper<T>` (used when a `Comparison<T>` is passed),
//! * the generic `GenericArraySortHelper<T>` (default comparer of an `IComparable<T>`),
//!   which uses `LessThan`/`GreaterThan` and bounds checks in the partition loops.
//!
//! A failing comparison is reported like .NET does ("Failed to compare two elements in the
//! array."), as an unexpected error.
use crate::math_parser_exception::{CResult, MathParserException};

const INTROSORT_SIZE_THRESHOLD: usize = 16;

fn compare_failed(_e: MathParserException) -> MathParserException {
    MathParserException::unexpected("Failed to compare two elements in the array.")
}

fn depth_limit(len: usize) -> i32 {
    2 * ((usize::BITS - 1 - (len as u32 as usize).leading_zeros()) as i32 + 1)
}

/// Swaps keys (and the matching items, if any).
#[inline]
fn swap<T, V>(keys: &mut [T], items: &mut Option<&mut [V]>, i: usize, j: usize) {
    keys.swap(i, j);
    if let Some(v) = items {
        v.swap(i, j);
    }
}

// ----- ArraySortHelper<T> with a Comparison<T> ---------------------------------------------

/// `Span<T>.Sort(Comparison<T>)` and `Span<TKey>.Sort(Span<TValue>, Comparison<TKey>)`.
pub fn sort_by<T: Clone, V: Clone>(
    keys: &mut [T],
    mut items: Option<&mut [V]>,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    if keys.len() > 1 {
        intro_sort_cmp(keys, &mut items, 0, keys.len(), depth_limit(keys.len()), cmp).map_err(compare_failed)?;
    }
    Ok(())
}

fn swap_if_greater_cmp<T, V>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
    i: usize,
    j: usize,
) -> CResult<()> {
    if cmp(&keys[i], &keys[j])? > 0 {
        swap(keys, items, i, j);
    }
    Ok(())
}

fn intro_sort_cmp<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    len: usize,
    mut depth: i32,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    let mut partition_size = len;
    while partition_size > 1 {
        if partition_size <= INTROSORT_SIZE_THRESHOLD {
            if partition_size == 2 {
                return swap_if_greater_cmp(keys, items, cmp, lo, lo + 1);
            }
            if partition_size == 3 {
                swap_if_greater_cmp(keys, items, cmp, lo, lo + 1)?;
                swap_if_greater_cmp(keys, items, cmp, lo, lo + 2)?;
                return swap_if_greater_cmp(keys, items, cmp, lo + 1, lo + 2);
            }
            return insertion_sort_cmp(keys, items, lo, partition_size, cmp);
        }
        if depth == 0 {
            return heap_sort_cmp(keys, items, lo, partition_size, cmp);
        }
        depth -= 1;
        let p = pick_pivot_and_partition_cmp(keys, items, lo, partition_size, cmp)?;
        intro_sort_cmp(keys, items, lo + p + 1, partition_size - (p + 1), depth, cmp)?;
        partition_size = p;
    }
    Ok(())
}

fn pick_pivot_and_partition_cmp<T: Clone, V>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    len: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<usize> {
    let hi = len - 1;
    let middle = hi >> 1;
    swap_if_greater_cmp(keys, items, cmp, lo, lo + middle)?;
    swap_if_greater_cmp(keys, items, cmp, lo, lo + hi)?;
    swap_if_greater_cmp(keys, items, cmp, lo + middle, lo + hi)?;
    let pivot = keys[lo + middle].clone();
    swap(keys, items, lo + middle, lo + hi - 1);
    let mut left = 0usize;
    let mut right = hi - 1;
    while left < right {
        loop {
            left += 1;
            if cmp(&keys[lo + left], &pivot)? >= 0 {
                break;
            }
        }
        loop {
            right -= 1;
            if cmp(&pivot, &keys[lo + right])? >= 0 {
                break;
            }
        }
        if left >= right {
            break;
        }
        swap(keys, items, lo + left, lo + right);
    }
    if left != hi - 1 {
        swap(keys, items, lo + left, lo + hi - 1);
    }
    Ok(left)
}

fn heap_sort_cmp<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    n: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    let mut i = n >> 1;
    while i >= 1 {
        down_heap_cmp(keys, items, lo, i, n, cmp)?;
        i -= 1;
    }
    let mut i = n;
    while i > 1 {
        swap(keys, items, lo, lo + i - 1);
        down_heap_cmp(keys, items, lo, 1, i - 1, cmp)?;
        i -= 1;
    }
    Ok(())
}

fn down_heap_cmp<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    mut i: usize,
    n: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    let d = keys[lo + i - 1].clone();
    let dv = items.as_ref().map(|v| v[lo + i - 1].clone());
    while i <= n >> 1 {
        let mut child = 2 * i;
        if child < n && cmp(&keys[lo + child - 1], &keys[lo + child])? < 0 {
            child += 1;
        }
        if cmp(&d, &keys[lo + child - 1])? >= 0 {
            break;
        }
        keys[lo + i - 1] = keys[lo + child - 1].clone();
        if let Some(v) = items {
            v[lo + i - 1] = v[lo + child - 1].clone();
        }
        i = child;
    }
    keys[lo + i - 1] = d;
    if let (Some(v), Some(dv)) = (items, dv) {
        v[lo + i - 1] = dv;
    }
    Ok(())
}

fn insertion_sort_cmp<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    len: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    for i in 0..len - 1 {
        let t = keys[lo + i + 1].clone();
        let tv = items.as_ref().map(|v| v[lo + i + 1].clone());
        let mut j = i as isize;
        while j >= 0 && cmp(&t, &keys[lo + j as usize])? < 0 {
            keys[lo + j as usize + 1] = keys[lo + j as usize].clone();
            if let Some(v) = items {
                v[lo + j as usize + 1] = v[lo + j as usize].clone();
            }
            j -= 1;
        }
        keys[(lo as isize + j + 1) as usize] = t;
        if let (Some(v), Some(tv)) = (items.as_mut(), tv) {
            v[(lo as isize + j + 1) as usize] = tv;
        }
    }
    Ok(())
}

// ----- GenericArraySortHelper<T> (default comparer) ----------------------------------------

/// `Span<T>.Sort()` / `Span<TKey>.Sort(Span<TValue>)` for a type with `IComparable<T>`.
/// `cmp(a, b)` is `a.CompareTo(b)`; `LessThan` is `cmp < 0` and `GreaterThan` is `cmp > 0`.
pub fn sort_default<T: Clone, V: Clone>(
    keys: &mut [T],
    mut items: Option<&mut [V]>,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    if keys.len() > 1 {
        intro_sort_gen(keys, &mut items, 0, keys.len(), depth_limit(keys.len()), cmp).map_err(compare_failed)?;
    }
    Ok(())
}

/// `Span<double>.Sort()` and `Span<double>.Sort(Span<TValue>)`: NaNs are moved to the
/// front first, then the rest is sorted with `<`.
pub fn sort_doubles<V: Clone>(keys: &mut [f64], mut items: Option<&mut [V]>) {
    if keys.len() < 2 {
        return;
    }
    // SortUtils.MoveNansToFront
    let mut left = 0usize;
    for i in 0..keys.len() {
        if keys[i].is_nan() {
            keys.swap(left, i);
            if let Some(v) = items.as_mut() {
                v.swap(left, i);
            }
            left += 1;
        }
    }
    let rest = &mut keys[left..];
    if rest.len() > 1 {
        let mut it = items.map(|v| &mut v[left..]);
        let n = rest.len();
        let _ = intro_sort_gen(rest, &mut it, 0, n, depth_limit(n), &mut |a: &f64, b: &f64| {
            Ok(if *a < *b {
                -1
            } else if *a > *b {
                1
            } else {
                0
            })
        });
    }
}

fn swap_if_greater_gen<T, V>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
    i: usize,
    j: usize,
) -> CResult<()> {
    // GenericArraySortHelper: if (i != j && LessThan(keys[j], keys[i])) swap
    if i != j && cmp(&keys[j], &keys[i])? < 0 {
        swap(keys, items, i, j);
    }
    Ok(())
}

fn intro_sort_gen<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    len: usize,
    mut depth: i32,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    let mut partition_size = len;
    while partition_size > 1 {
        if partition_size <= INTROSORT_SIZE_THRESHOLD {
            if partition_size == 2 {
                return swap_if_greater_gen(keys, items, cmp, lo, lo + 1);
            }
            if partition_size == 3 {
                swap_if_greater_gen(keys, items, cmp, lo, lo + 1)?;
                swap_if_greater_gen(keys, items, cmp, lo, lo + 2)?;
                return swap_if_greater_gen(keys, items, cmp, lo + 1, lo + 2);
            }
            return insertion_sort_gen(keys, items, lo, partition_size, cmp);
        }
        if depth == 0 {
            return heap_sort_gen(keys, items, lo, partition_size, cmp);
        }
        depth -= 1;
        let p = pick_pivot_and_partition_gen(keys, items, lo, partition_size, cmp)?;
        intro_sort_gen(keys, items, lo + p + 1, partition_size - (p + 1), depth, cmp)?;
        partition_size = p;
    }
    Ok(())
}

fn pick_pivot_and_partition_gen<T: Clone, V>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    len: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<usize> {
    let hi = len - 1;
    let middle = hi >> 1;
    swap_if_greater_gen(keys, items, cmp, lo, lo + middle)?;
    swap_if_greater_gen(keys, items, cmp, lo, lo + hi)?;
    swap_if_greater_gen(keys, items, cmp, lo + middle, lo + hi)?;
    let pivot = keys[lo + middle].clone();
    swap(keys, items, lo + middle, lo + hi - 1);
    let mut left = 0usize;
    let mut right = hi - 1;
    while left < right {
        // while (left < hi - 1 && GreaterThan(pivot, keys[++left])) ;
        while left < hi - 1 {
            left += 1;
            if cmp(&pivot, &keys[lo + left])? <= 0 {
                break;
            }
        }
        // while (right > 0 && LessThan(pivot, keys[--right])) ;
        while right > 0 {
            right -= 1;
            if cmp(&pivot, &keys[lo + right])? >= 0 {
                break;
            }
        }
        if left >= right {
            break;
        }
        swap(keys, items, lo + left, lo + right);
    }
    if left != hi - 1 {
        swap(keys, items, lo + left, lo + hi - 1);
    }
    Ok(left)
}

fn heap_sort_gen<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    n: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    let mut i = n >> 1;
    while i >= 1 {
        down_heap_gen(keys, items, lo, i, n, cmp)?;
        i -= 1;
    }
    let mut i = n;
    while i > 1 {
        swap(keys, items, lo, lo + i - 1);
        down_heap_gen(keys, items, lo, 1, i - 1, cmp)?;
        i -= 1;
    }
    Ok(())
}

fn down_heap_gen<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    mut i: usize,
    n: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    let d = keys[lo + i - 1].clone();
    let dv = items.as_ref().map(|v| v[lo + i - 1].clone());
    while i <= n >> 1 {
        let mut child = 2 * i;
        if child < n && cmp(&keys[lo + child - 1], &keys[lo + child])? < 0 {
            child += 1;
        }
        if cmp(&d, &keys[lo + child - 1])? >= 0 {
            break;
        }
        keys[lo + i - 1] = keys[lo + child - 1].clone();
        if let Some(v) = items {
            v[lo + i - 1] = v[lo + child - 1].clone();
        }
        i = child;
    }
    keys[lo + i - 1] = d;
    if let (Some(v), Some(dv)) = (items, dv) {
        v[lo + i - 1] = dv;
    }
    Ok(())
}

fn insertion_sort_gen<T: Clone, V: Clone>(
    keys: &mut [T],
    items: &mut Option<&mut [V]>,
    lo: usize,
    len: usize,
    cmp: &mut dyn FnMut(&T, &T) -> CResult<i32>,
) -> CResult<()> {
    for i in 0..len - 1 {
        let t = keys[lo + i + 1].clone();
        let tv = items.as_ref().map(|v| v[lo + i + 1].clone());
        let mut j = i as isize;
        while j >= 0 && cmp(&t, &keys[lo + j as usize])? < 0 {
            keys[lo + j as usize + 1] = keys[lo + j as usize].clone();
            if let Some(v) = items {
                v[lo + j as usize + 1] = v[lo + j as usize].clone();
            }
            j -= 1;
        }
        keys[(lo as isize + j + 1) as usize] = t;
        if let (Some(v), Some(tv)) = (items.as_mut(), tv) {
            v[(lo as isize + j + 1) as usize] = tv;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_like_a_sort() {
        let mut v: Vec<f64> = (0..100).map(|i| ((i * 37) % 101) as f64).collect();
        let mut idx: Vec<i32> = (0..100).collect();
        sort_doubles(&mut v, Some(&mut idx));
        for w in v.windows(2) {
            assert!(w[0] <= w[1]);
        }
        let mut k: Vec<i32> = (0..50).map(|i| (i * 7) % 13).collect();
        sort_by::<i32, i32>(&mut k, None, &mut |a, b| Ok(-(a.cmp(b) as i32))).unwrap();
        for w in k.windows(2) {
            assert!(w[0] >= w[1]);
        }
    }
}
