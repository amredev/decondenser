//! This code was originally adapted from `prettyplease`'s `ring.rs`. See the
//! parent module's doc comment for more references.

use std::collections::{VecDeque, vec_deque};

/// A sliding deque is almost exactly the same as a regular [`VecDeque`] but if
/// you pop an element from the front of it, then the remaining elements will
/// stay at their original indices. If you pop several times from the front and
/// push some elements to the back the queue will effectively "slide" to the
/// right on the infinite axis of indices.
pub(super) struct SlidingDeque<T> {
    data: VecDeque<T>,

    /// Abstract index of the zero-th element in `data`.
    basis: usize,
}

impl<T> SlidingDeque<T> {
    pub(super) fn new() -> Self {
        Self {
            data: VecDeque::new(),
            basis: 0,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.data.len()
    }

    pub(super) fn iter(&self) -> vec_deque::Iter<'_, T> {
        self.data.iter()
    }

    pub(super) fn push_back(&mut self, value: T) -> usize {
        let index = self.basis + self.data.len();
        self.data.push_back(value);
        index
    }

    pub(super) fn basis(&self) -> usize {
        self.basis
    }

    pub(super) fn front(&self) -> Option<&T> {
        self.data.front()
    }

    pub(super) fn front_mut(&mut self) -> Option<&mut T> {
        self.data.front_mut()
    }

    pub(super) fn pop_front(&mut self) -> Option<T> {
        self.data.pop_front().inspect(|_| self.basis += 1)
    }

    pub(super) fn pop_back(&mut self) -> Option<T> {
        self.data.pop_back()
    }

    pub(super) fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index.checked_sub(self.basis)?)
    }
}
