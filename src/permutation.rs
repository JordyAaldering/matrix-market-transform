/// Apply a permutation to a slice of elements.
///
/// Extracted from https://github.com/jeremysalwen/rust-permutations.
pub fn apply<T>(permutation: &mut Vec<usize>, slice: &mut Vec<T>) {
    for i in 0..permutation.len() {
        let i_idx = permutation[i];

        if idx_is_marked(i_idx) {
            continue;
        }

        let mut j = i;
        let mut j_idx = i_idx;

        // When we loop back to the first index, we stop
        while j_idx != i {
            permutation[j] = toggle_mark_idx(j_idx);
            slice.swap(j, j_idx);
            j = j_idx;
            j_idx = permutation[j];
        }

        permutation[j] = toggle_mark_idx(j_idx);
    }
}

/// Reset permutation back to the range (0..nvals).
#[inline]
pub fn reset(permutation: &mut Vec<usize>) {
    permutation.iter_mut().for_each(|idx|
        *idx = toggle_mark_idx(*idx));
}

/// Toggle the most-significant bit.
#[inline(always)]
fn toggle_mark_idx(idx: usize) -> usize {
    const MASK: usize = isize::MIN as usize;
    idx ^ MASK
}

/// Check if the most-significant bit is set.
#[inline(always)]
fn idx_is_marked(idx: usize) -> bool {
    const MASK: usize = isize::MIN as usize;
    (idx & MASK) != 0
}
