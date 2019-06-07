// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

/// Workaround for `Vec::retain()` passing `&T` instead of
/// `&mut T`. See RFC #2160 and issue #25477 for discussion
/// of inclusion of this in `std` (looks like it won't be),
/// and issue #43244 tracking `Vec::drain_filter()`, which
/// is in nightly as a more general proposed replacement,
/// but currently has stabilization issues.
///
/// This function calls `f` on each element *e* of `v`, with
/// *e* passed as a mutable reference. If `f` returns
/// `true`, *e* is retained; otherwise it is dropped.
/// `v` is truncated to the size of the retained elements, which
/// appear in the original order.
///
/// # Examples:
///
/// ```
/// # use synthkit::retain_mut;
/// let mut v = vec![2, 3, 5, 6, 7, 8];
/// let f = |x: &mut usize| {
///     let odd = *x & 1 == 1;
///     if odd {
///         *x *= 2;
///     }
///     odd
/// };
/// retain_mut(&mut v, f);
/// assert_eq!(&vec![6, 10, 14], &v);
/// ```
pub fn retain_mut<T, F>(v: &mut Vec<T>, mut f: F)
    where F: FnMut(&mut T) -> bool
{
    let mut j = 0;
    for i in 0..v.len() {
        if f(&mut v[i]) {
            if i > j {
                v.swap(i, j);
            }
            j += 1;
        }
    }
    v.truncate(j);
}

#[test]
fn retain_mut_empty() {
    let mut v = vec![];
    let f = |x: &mut usize| { *x % 2 == 1 };
    retain_mut(&mut v, f);
    assert!(v.is_empty());
}

#[test]
fn retain_mut_full() {
    let v = vec![1, 3, 5];
    let mut w = v.clone();
    let f = |x: &mut usize| { *x % 2 == 1 };
    retain_mut(&mut w, f);
    assert_eq!(&v, &w);
}

#[test]
fn retain_mut_partial() {
    let mut v = vec![2, 3, 5, 6, 7, 8];
    let f = |x: &mut usize| { *x % 2 == 1 };
    retain_mut(&mut v, f);
    assert_eq!(&vec![3, 5, 7], &v);
}
