use std;
use std::cmp::Ordering;
use std::convert::AsRef;

#[derive(Clone, Debug)]
pub struct Permutation {
    forward: bool,
    pub indices: Vec<usize>,
}

impl std::cmp::PartialEq for Permutation {
    ///  This method compares two Permutations for equality, and is used by `==`
    fn eq(&self, other: &Permutation) -> bool {
        if self.forward == other.forward {
            self.indices == other.indices
        } else {
            self.indices
                .iter()
                .enumerate()
                .all(|(i, &j)| other.indices[j] == i)
        }
    }
}
impl std::cmp::Eq for Permutation {}
impl<'a, 'b> std::ops::Mul<&'b Permutation> for &'a Permutation {
    type Output = Permutation;
    /// Multiply permutations, in the mathematical sense.
    ///
    /// Given two permutations `a`, and `b`, `a * b` is defined as
    /// the permutation created by first applying b, then applying a.
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let p1 = Permutation::oneline([1, 0, 2]);
    /// let p2 = Permutation::oneline([0, 2, 1]);
    /// assert_eq!(&p1 * &p2, Permutation::oneline([1,2,0]));
    /// ```

    fn mul(self, rhs: &'b Permutation) -> Self::Output {
        match (self.forward, rhs.forward) {
            (_, false) => Permutation::oneline(self.apply_slice(&rhs.indices)).inverse(),
            (false, true) => return self * &(rhs * &Permutation::one(self.len())),
            (true, true) => Permutation {
                forward: true,
                indices: rhs.apply_inv_slice(&self.indices),
            },
        }
    }
}

impl Permutation {
    /// Create a permutation from a vector of indices.
    ///
    /// from_vec(v) returns the permutation P such that P applied to [1,2,...,N] is v.
    /// Note that this notation is the inverse of the usual [one-line notation](https://en.wikipedia.org/wiki/Permutation#Definition_and_notations)
    /// used in mathematics.  This is a known issue and may change in a newer release.
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let vec = vec!['a','b','c','d'];
    /// let permutation = Permutation::from_vec([0,2,3,1]);
    /// assert_eq!(permutation.apply_slice(&vec), vec!['a','c','d','b']);
    /// ```
    #[deprecated(since = "0.4.0", note = "Please replace with oneline(vec).inverse()")]
    pub fn from_vec<V>(vec: V) -> Permutation
    where
        V: Into<Vec<usize>>,
    {
        let result = Permutation {
            forward: false,
            indices: vec.into(),
        };

        debug_assert!(result.valid());
        return result;
    }

    /// Create a permutation from zero-based oneline notation
    ///
    /// This creates a permutation from [one-line notation](https://en.wikipedia.org/wiki/Permutation#Definition_and_notations)
    /// notation used in mathematics, but using zero-based indices rather than the one-based indices
    /// typically used in mathematics.
    ///
    /// Note that this is the inverse notation used by the deprecated `Permutation::from_vec()`.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let vec = vec!['a','b','c','d'];
    /// let permutation = Permutation::oneline([0,2,3,1]);
    /// assert_eq!(permutation.apply_slice(&vec), vec!['a','d','b','c']);
    /// ```
    pub fn oneline<V>(vec: V) -> Permutation
    where
        V: Into<Vec<usize>>,
    {
        let result = Permutation {
            forward: true,
            indices: vec.into(),
        };

        debug_assert!(result.valid());
        return result;
    }

    /// Computes the permutation that would sort a given slice.
    ///
    /// This is the same as `permutation::sort()`, but assigned in-place to `self` rather than
    /// allocating a new permutation.
    ///
    /// # Panics
    ///
    /// If self.len() != vec.len()
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// // Say you have a permutation that we don't need anymore...
    /// let mut permutation = permutation::sort(&[0,1,2]);
    ///
    /// // You can reuse it rather than allocating a new one, as long as the length is the same
    /// let mut vec = vec!['z','w','h'];
    /// permutation.assign_from_sort(&vec);
    /// let permuted = permutation.apply_slice(&vec);
    /// vec.sort();
    /// assert_eq!(vec, permuted);
    ///
    /// // You can also use it to sort multiple arrays based on the ordering of one.
    /// let names = vec!["Bob", "Steve", "Jane"];
    /// let salary = vec![10, 5, 15];
    /// permutation.assign_from_sort(&salary);
    /// let ordered_names = permutation.apply_slice(&names);
    /// let ordered_salaries = permutation.apply_slice(&salary);
    /// assert_eq!(ordered_names, vec!["Steve", "Bob", "Jane"]);
    /// assert_eq!(ordered_salaries, vec![5, 10, 15]);
    /// ```
    pub fn assign_from_sort<T, S>(&mut self, slice: S)
    where
        T: Ord,
        S: AsRef<[T]>,
    {
        let s = slice.as_ref();
        assert_eq!(self.len(), s.len());
        //We use the reverse permutation form, because its more efficient for applying to indices.
        self.indices.sort_by_key(|&i| &s[i]);
    }

    /// Computes the permutation that would sort a given slice by a comparator.
    ///
    /// This is the same as `permutation::sort_by()`, but assigned in-place to `self` rather than
    /// allocating a new permutation.
    ///
    /// # Panics
    ///
    /// If self.len() != vec.len()
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// // Say you have a permutation that we don't need anymore...
    /// let mut permutation = permutation::sort(&[0,1,2,3,4,5]);
    ///
    /// // You can assign to it rather than allocating a new one, as long as the length is the same
    /// let mut vec = vec!['z','w','h','a','s','j'];
    /// permutation.assign_from_sort_by(&vec, |a, b| b.cmp(a));
    /// let permuted = permutation.apply_slice(&vec);
    /// vec.sort_by(|a,b| b.cmp(a));
    /// assert_eq!(vec, permuted);
    /// ```
    pub fn assign_from_sort_by<T, S, F>(&mut self, slice: S, mut compare: F)
    where
        S: AsRef<[T]>,
        F: FnMut(&T, &T) -> Ordering,
    {
        let s = slice.as_ref();
        assert_eq!(self.indices.len(), s.len());
        // We use the reverse permutation form, because its more efficient for applying to indices.
        self.indices.sort_by(|&i, &j| compare(&s[i], &s[j]));
    }

    /// Computes the permutation that would sort a given slice by a key function.
    ///
    /// This is the same as `permutation::sort_by_key()`, but assigned in-place to `self` rather than
    /// allocating a new permutation.
    ///
    /// # Panics
    ///
    /// If self.len() != vec.len()
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// // Say you have a permutation that we don't need anymore...
    /// let mut permutation = permutation::sort(&[0,1,2,3,4,5]);
    ///
    /// // You can assign to it rather than allocating a new one, as long as the length is the same
    /// let mut vec = vec![2, 4, 6, 8, 10, 11];
    /// permutation.assign_from_sort_by_key(&vec, |a| a % 3);
    /// let permuted = permutation.apply_slice(&vec);
    /// vec.sort_by_key(|a| a % 3);
    /// assert_eq!(vec, permuted);
    /// ```
    pub fn assign_from_sort_by_key<T, S, B, F>(&mut self, slice: S, mut f: F)
    where
        B: Ord,
        S: AsRef<[T]>,
        F: FnMut(&T) -> B,
    {
        let s = slice.as_ref();
        assert_eq!(self.indices.len(), s.len());
        //We use the reverse permutation form, because its more efficient for applying to indices.
        self.indices.sort_by_key(|&i| f(&s[i]));
    }
    /// Return the identity permutation of size N.
    ///
    /// This returns the identity permutation of N elements.
    ///
    /// # Examples
    /// ```
    /// # use permutation::Permutation;
    /// let vec = vec!['a','b','c','d'];
    /// let permutation = Permutation::one(4);
    /// assert_eq!(permutation.apply_slice(&vec), vec!['a','b','c','d']);
    /// ```
    pub fn one(len: usize) -> Permutation {
        Permutation {
            forward: false,
            indices: (0..len).collect(),
        }
    }
    /// Return the size of a permutation.
    ///
    /// This is the number of elements that the permutation acts on.
    ///
    /// # Examples
    /// ```
    /// use permutation::Permutation;
    /// let permutation = Permutation::one(4);
    /// assert_eq!(permutation.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        return self.indices.len();
    }
    /// Check whether a permutation is valid.
    ///
    /// A permutation can be invalid if it was constructed with an
    /// incorrect vector using ``::from_vec()`` or ``::oneline()``.
    /// Debug assertions will catch this on construction, so it should
    /// never return true.
    ///
    pub fn valid(&self) -> bool {
        let mut sorted = self.indices.clone();
        sorted.sort();
        return sorted.iter().enumerate().all(|(i, &j)| i == j);
    }

    /// Return the inverse of a permutation.
    ///
    /// This returns a permutation that will undo a given permutation.
    /// Internally, this does not compute the inverse, but just flips a bit.
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let permutation = Permutation::oneline([0,2,3,1]);
    /// assert_eq!(permutation.inverse(), Permutation::oneline([0,3,1,2]));
    /// ```
    pub fn inverse(mut self) -> Permutation {
        self.forward ^= true;
        return self;
    }

    /// Normalize the internal storage of the `Permutation`, optimizing it for forward or inverse application.
    ///
    /// Internally, the permutation has a bit to indicate whether it is inverted.
    /// This is because given a permutation P, it is just as easy to compute `P^-1 * Q`
    /// as it is to compute `P * Q`. However, computing the entries of `P^-1` requires some computation.
    /// However, when applying to the permutation to an index, the permutation has a "preferred" direction, which
    /// is much quicker to compute.
    ///
    /// The `normalize()` method does not change the value of the permutation, but
    /// it converts it into the preferred form for applying `P` or its inverse, respectively.
    ///
    /// If `backward` is `false`, it will be in the preferred form for applying `P`,
    /// if `backward` is `true`, it will be in the preferred form for appling `P^-1`
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let permutation = Permutation::oneline([0, 3, 2, 5, 1, 4]);
    /// let reversed = permutation.inverse().normalize(true);
    /// assert_eq!(reversed.apply_inv_idx(3), 5);
    /// ```
    pub fn normalize(self, backward: bool) -> Permutation {
        if self.forward ^ backward {
            self
        } else {
            let len = self.len();
            if backward {
                &self * &Permutation::one(len)
            } else {
                (&self.inverse() * &Permutation::one(len)).inverse()
            }
        }
    }
    fn apply_idx_fwd(&self, idx: usize) -> usize {
        self.indices.iter().position(|&v| v == idx).unwrap()
    }
    fn apply_idx_bkwd(&self, idx: usize) -> usize {
        self.indices[idx]
    }

    /// Apply the permutation to an index.
    ///
    /// Given an index of an element, this will return the new index
    /// of that element after applying the permutation.
    ///
    /// Note that the running time will be O(1) or O(N) depending on
    /// how the permutation is normalized (see [`Permutation::normalize`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let permutation = Permutation::oneline([0,2,1]);
    /// assert_eq!(permutation.apply_idx(1), 2);
    pub fn apply_idx(&self, idx: usize) -> usize {
        match self.forward {
            false => self.apply_idx_fwd(idx),
            true => self.apply_idx_bkwd(idx),
        }
    }

    /// Apply the inverse of a permutation to an index.
    ///
    /// Given an index of an element, this will return the new index
    /// of that element after applying 'P^-1'.
    ///
    /// Equivalently, if `P.apply_idx(i) == j`, then `P.apply_inv_idx(j) == i`.
    ///
    /// Note that the running time will be O(1) or O(N) depending on
    /// how the permutation is normalized (see [`Permutation::normalize`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let permutation = Permutation::oneline([0,2,1]);
    /// assert_eq!(permutation.apply_inv_idx(2), 1);
    /// ```
    pub fn apply_inv_idx(&self, idx: usize) -> usize {
        match self.forward {
            true => self.apply_idx_fwd(idx),
            false => self.apply_idx_bkwd(idx),
        }
    }
    fn apply_slice_fwd<T: Clone, S>(&self, slice: S) -> Vec<T>
    where
        S: AsRef<[T]>,
    {
        let s = slice.as_ref();
        self.indices.iter().map(|&idx| s[idx].clone()).collect()
    }

    fn apply_slice_bkwd<T: Clone, S>(&self, slice: S) -> Vec<T>
    where
        S: AsRef<[T]>,
    {
        let s = slice.as_ref();
        let mut other: Vec<T> = s.to_vec();
        for (i, idx) in self.indices.iter().enumerate() {
            other[*idx] = s[i].clone();
        }
        return other;
    }

    // For the in place methods, we apply each cycle in the permutation in turn, marking the indices with their MSB when
    // they have been resolved. The MSB will always be unset as long as n <= isize::max_value().
    // This way, we can recover the original indices in O(n) and perform no heap allocations.

    #[inline(always)]
    fn toggle_mark_idx(idx: usize) -> usize {
        idx ^ isize::min_value() as usize
    }

    #[inline(always)]
    fn idx_is_marked(idx: usize) -> bool {
        (idx & (isize::min_value() as usize)) != 0
    }

    fn apply_slice_bkwd_in_place<T, S>(&mut self, slice: &mut S)
    where
        S: AsMut<[T]> + ?Sized,
    {
        let s = slice.as_mut();
        assert_eq!(s.len(), self.len());
        assert!(s.len() <= isize::max_value() as usize);

        for idx in self.indices.iter() {
            debug_assert!(!Self::idx_is_marked(*idx));
        }

        for i in 0..self.indices.len() {
            let i_idx = self.indices[i];

            if Self::idx_is_marked(i_idx) {
                continue;
            }

            let mut j = i;
            let mut j_idx = i_idx;

            // When we loop back to the first index, we stop
            while j_idx != i {
                self.indices[j] = Self::toggle_mark_idx(j_idx);
                s.swap(j, j_idx);
                j = j_idx;
                j_idx = self.indices[j];
            }

            self.indices[j] = Self::toggle_mark_idx(j_idx);
        }

        for idx in self.indices.iter_mut() {
            debug_assert!(Self::idx_is_marked(*idx));
            *idx = Self::toggle_mark_idx(*idx);
        }
    }

    fn apply_slice_fwd_in_place<T, S>(&mut self, slice: &mut S)
    where
        S: AsMut<[T]> + ?Sized,
    {
        let s = slice.as_mut();
        assert_eq!(s.len(), self.len());
        assert!(s.len() <= isize::max_value() as usize);

        for idx in self.indices.iter() {
            debug_assert!(!Self::idx_is_marked(*idx));
        }

        for i in 0..self.indices.len() {
            let i_idx = self.indices[i];

            if Self::idx_is_marked(i_idx) {
                continue;
            }

            let mut j = i;
            let mut j_idx = i_idx;

            // When we loop back to the first index, we stop
            while j_idx != i {
                self.indices[j] = Self::toggle_mark_idx(j_idx);
                s.swap(i, j_idx);
                j = j_idx;
                j_idx = self.indices[j];
            }

            self.indices[j] = Self::toggle_mark_idx(j_idx);
        }

        for idx in self.indices.iter_mut() {
            debug_assert!(Self::idx_is_marked(*idx));
            *idx = Self::toggle_mark_idx(*idx);
        }
    }

    /// Apply a permutation to a slice of elements
    ///
    /// Given a slice of elements, this will permute the elements according
    /// to this permutation and clone them to a `Vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let permutation = Permutation::oneline([0,3,1,2]);
    /// let vec = vec!['a','b','c','d'];
    /// assert_eq!(permutation.apply_slice(&vec), vec!['a', 'c', 'd', 'b']);
    /// ```
    #[must_use]
    pub fn apply_slice<T: Clone, S>(&self, slice: S) -> Vec<T>
    where
        S: AsRef<[T]>,
    {
        let s = slice.as_ref();
        assert_eq!(s.len(), self.len());
        match self.forward {
            false => self.apply_slice_fwd(s),
            true => self.apply_slice_bkwd(s),
        }
    }
    /// Apply the inverse of a permutation to a slice of elements
    ///
    /// Given a slice of elements, this will permute the elements according
    /// to the inverse of this permutation and clone them to a `Vec`.
    /// This is equivalent to "undoing" the permutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let permutation = Permutation::oneline([0,3,1,2]);
    /// let vec = vec!['a','b', 'c', 'd'];
    /// assert_eq!(permutation.apply_inv_slice(vec), vec!['a', 'd', 'b', 'c']);
    /// ```
    #[must_use]
    pub fn apply_inv_slice<T: Clone, S>(&self, slice: S) -> Vec<T>
    where
        S: AsRef<[T]>,
    {
        let s = slice.as_ref();
        assert_eq!(s.len(), self.len());
        match self.forward {
            false => self.apply_slice_bkwd(s),
            true => self.apply_slice_fwd(s),
        }
    }

    /// Apply a permutation to a slice of elements
    ///
    /// Given a slice of elements, this will permute the elements in place according
    /// to this permutation.
    ///
    /// This method borrows `self` mutably to avoid allocations, but the permutation
    /// will be unchanged after it returns.
    ///
    /// # Panics
    ///
    /// If `slice.len() != self.len()`.
    /// If `slice.len()` > isize::max_value(), due to implementation reasons.
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let mut permutation = Permutation::oneline([0,3,1,2]);
    /// let mut vec = vec!['a', 'b', 'c', 'd'];
    /// let permutation_old = permutation.clone();
    /// permutation.apply_slice_in_place(&mut vec);
    /// assert_eq!(vec, vec!['a', 'c', 'd', 'b']);
    /// assert_eq!(permutation, permutation_old);
    /// ```
    pub fn apply_slice_in_place<T, S>(&mut self, slice: &mut S)
    where
        S: AsMut<[T]> + ?Sized,
    {
        match self.forward {
            false => self.apply_slice_bkwd_in_place(slice),
            true => self.apply_slice_fwd_in_place(slice),
        }
    }

    /// Apply the inverse of a permutation to a slice of elements
    ///
    /// Given a slice of elements, this will permute the elements in place according
    /// to the inverse of this permutation.
    /// This is equivalent to "undoing" the permutation.
    ///
    /// This method borrows `self` mutably to avoid allocations, but the permutation
    /// will be unchanged after it returns.
    ///
    /// # Panics
    ///
    /// If `slice.len() != self.len()`.
    /// If `slice.len()` > isize::max_value(), due to implementation reasons.
    ///
    /// # Examples
    ///
    /// ```
    /// # use permutation::Permutation;
    /// let mut permutation = Permutation::oneline([0,3,1,2]);
    /// let mut vec = vec!['a', 'b', 'c', 'd'];
    /// permutation.apply_inv_slice_in_place(&mut vec);
    /// assert_eq!(vec, vec!['a', 'd', 'b', 'c']);
    /// ```
    pub fn apply_inv_slice_in_place<T, S>(&mut self, slice: &mut S)
    where
        S: AsMut<[T]> + ?Sized,
    {
        match self.forward {
            false => self.apply_slice_fwd_in_place(slice),
            true => self.apply_slice_bkwd_in_place(slice),
        }
    }
}
/// Return the permutation that would sort a given slice.
///
/// This calculates the permutation that if it were applied to the slice,
/// would put the elements in sorted order.
///
/// # Examples
///
/// ```
/// # use permutation::Permutation;
/// let mut vec = vec!['z','w','h','a','s','j'];
/// let permutation = permutation::sort(&vec);
/// let permuted = permutation.apply_slice(&vec);
/// vec.sort();
/// assert_eq!(vec, permuted);
/// ```
///
/// You can also use it to sort multiple arrays based on the ordering of one.
///
/// ```
/// let names = vec!["Bob", "Steve", "Jane"];
/// let salary = vec![10, 5, 15];
/// let permutation = permutation::sort(&salary);
/// let ordered_names = permutation.apply_slice(&names);
/// let ordered_salaries = permutation.apply_slice(&salary);
/// assert_eq!(ordered_names, vec!["Steve", "Bob", "Jane"]);
/// assert_eq!(ordered_salaries, vec![5, 10, 15]);
/// ```
pub fn sort<T, S>(slice: S) -> Permutation
where
    T: Ord,
    S: AsRef<[T]>,
{
    let s = slice.as_ref();
    let mut permutation = Permutation::one(s.len());
    //We use the reverse permutation form, because its more efficient for applying to indices.
    permutation.indices.sort_by_key(|&i| &s[i]);
    return permutation;
}

/// Return the permutation that would sort a given slice, but might not
/// preserve the order of equal elements.
///
/// This calculates the permutation that if it were applied to the slice,
/// would put the elements in sorted order.
///
/// # Examples
///
/// ```
/// # use permutation::Permutation;
/// let mut vec = vec!['z','w','h','a','s','j'];
/// let permutation = permutation::sort_unstable(&vec);
/// let permuted = permutation.apply_slice(&vec);
/// vec.sort();
/// assert_eq!(vec, permuted);
/// ```
///
/// You can also use it to sort multiple arrays based on the ordering of one.
///
/// ```
/// let names = vec!["Bob", "Steve", "Jane"];
/// let salary = vec![10, 5, 15];
/// let permutation = permutation::sort_unstable(&salary);
/// let ordered_names = permutation.apply_slice(&names);
/// let ordered_salaries = permutation.apply_slice(&salary);
/// assert_eq!(ordered_names, vec!["Steve", "Bob", "Jane"]);
/// assert_eq!(ordered_salaries, vec![5, 10, 15]);
/// ```
pub fn sort_unstable<T, S>(slice: S) -> Permutation
where
    T: Ord,
    S: AsRef<[T]>,
{
    let s = slice.as_ref();
    let mut permutation = Permutation::one(s.len());
    //We use the reverse permutation form, because its more efficient for applying to indices.
    permutation.indices.sort_unstable_by_key(|&i| &s[i]);
    return permutation;
}

/// Return the permutation that would sort a given slice by a comparator.
///
/// This is the same as `permutation::sort()` except that it allows you to specify
/// the comparator to use when sorting similar to `std::slice.sort_by()`.
///
/// If the comparator does not define a total ordering, the order of the elements is unspecified.
/// Per the [Rust Docs](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_by),
/// an order is a total order if it is (for all `a`, `b` and `c`):
///
/// * total and antisymmetric: exactly one of `a < b`, `a == b` or `a > b` is true, and
/// * transitive, `a < b` and `b < c` implies `a < c`. The same must hold for both `==` and `>`.
///
/// # Examples
///
/// ```
/// # use permutation::Permutation;
/// let mut vec = vec!['z','w','h','a','s','j'];
/// let permutation = permutation::sort_by(&vec, |a, b| b.cmp(a));
/// let permuted = permutation.apply_slice(&vec);
/// vec.sort_by(|a,b| b.cmp(a));
/// assert_eq!(vec, permuted);
/// ```
pub fn sort_by<T, S, F>(slice: S, mut compare: F) -> Permutation
where
    S: AsRef<[T]>,
    F: FnMut(&T, &T) -> Ordering,
{
    let s = slice.as_ref();
    let mut permutation = Permutation::one(s.len());
    //We use the reverse permutation form, because its more efficient for applying to indices.
    permutation.indices.sort_by(|&i, &j| compare(&s[i], &s[j]));
    return permutation;
}

/// Return the permutation that would sort a given slice by a comparator, but might not
/// preserve the order of equal elements.
///
/// This is the same as `permutation::sort_unstable()` except that it allows you to specify
/// the comparator to use when sorting similar to `std::slice.sort_unstable_by()`.
///
/// If the comparator does not define a total ordering, the order of the elements is unspecified.
/// Per the [Rust Docs](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_unstable_by),
/// an order is a total order if it is (for all `a`, `b` and `c`):
///
/// * total and antisymmetric: exactly one of `a < b`, `a == b` or `a > b` is true, and
/// * transitive, `a < b` and `b < c` implies `a < c`. The same must hold for both `==` and `>`.
///
/// # Examples
///
/// ```
/// # use permutation::Permutation;
/// let mut vec = vec!['z','w','h','a','s','j'];
/// let permutation = permutation::sort_unstable_by(&vec, |a, b| b.cmp(a));
/// let permuted = permutation.apply_slice(&vec);
/// vec.sort_by(|a,b| b.cmp(a));
/// assert_eq!(vec, permuted);
/// ```
pub fn sort_unstable_by<T, S, F>(slice: S, mut compare: F) -> Permutation
where
    S: AsRef<[T]>,
    F: FnMut(&T, &T) -> Ordering,
{
    let s = slice.as_ref();
    let mut permutation = Permutation::one(s.len());
    //We use the reverse permutation form, because its more efficient for applying to indices.
    permutation
        .indices
        .sort_unstable_by(|&i, &j| compare(&s[i], &s[j]));
    return permutation;
}

/// Return the permutation that would sort a given slice by a key function.
///
/// This is the same as `permutation::sort()` except that it allows you to specify
/// the key function simliar to `std::slice.sort_by_key()`
///
/// # Examples
///
/// ```
/// # use permutation::Permutation;
/// let mut vec = vec![2, 4, 6, 8, 10, 11];
/// let permutation = permutation::sort_by_key(&vec, |a| a % 3);
/// let permuted = permutation.apply_slice(&vec);
/// vec.sort_by_key(|a| a % 3);
/// assert_eq!(vec, permuted);
/// ```
pub fn sort_by_key<T, S, B, F>(slice: S, mut f: F) -> Permutation
where
    B: Ord,
    S: AsRef<[T]>,
    F: FnMut(&T) -> B,
{
    let s = slice.as_ref();
    let mut permutation = Permutation::one(s.len());
    //We use the reverse permutation form, because its more efficient for applying to indices.
    permutation.indices.sort_by_key(|&i| f(&s[i]));
    return permutation;
}

/// Return the permutation that would sort a given slice by a key function, but might not
/// preserve the order of equal elements.
///
/// This is the same as `permutation::sort_unstable()` except that it allows you to specify
/// the key function simliar to `std::slice.sort_unstable_by_key()`
///
/// # Examples
///
/// ```
/// # use permutation::Permutation;
/// let mut vec = vec![2, 4, 6, 8, 10, 11];
/// let permutation = permutation::sort_unstable_by_key(&vec, |a| a % 3);
/// let permuted = permutation.apply_slice(&vec);
/// vec.sort_by_key(|a| a % 3);
/// assert_eq!(vec, permuted);
/// ```
pub fn sort_unstable_by_key<T, S, B, F>(slice: S, mut f: F) -> Permutation
where
    B: Ord,
    S: AsRef<[T]>,
    F: FnMut(&T) -> B,
{
    let s = slice.as_ref();
    let mut permutation = Permutation::one(s.len());
    //We use the reverse permutation form, because its more efficient for applying to indices.
    permutation.indices.sort_unstable_by_key(|&i| f(&s[i]));
    return permutation;
}
