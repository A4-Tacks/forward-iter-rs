#![no_std]
#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

use core::{fmt::Debug, iter::FusedIterator};

use alloc::vec::{IntoIter, Vec};

extern crate alloc;

/// Create from [`ForwardIterExt::forward`]
#[must_use]
#[derive(Debug, Clone)]
pub struct ForwardIter<I: Iterator> {
    iter: Option<I>,
    buf: IntoIter<I::Item>,
}

impl<I: Iterator> ForwardIter<I> {
    fn backward(&mut self) -> &mut IntoIter<I::Item> {
        if let Some(iter) = self.iter.take() {
            self.buf = iter.collect::<Vec<_>>().into_iter();
        }
        &mut self.buf
    }
}

macro_rules! choice {
    ($self:ident:$name:ident $t:tt) => {
        $self.iter.as_mut()
            .map_or_else(|| $self.buf.$name $t, |iter| iter.$name $t)
    };
}

impl<I: Iterator + FusedIterator> FusedIterator for ForwardIter<I> {}
impl<I: Iterator + ExactSizeIterator> ExactSizeIterator for ForwardIter<I> {}
impl<I: Iterator> Iterator for ForwardIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        choice!(self:next())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        choice!(self:nth(n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.as_ref()
            .map_or_else(|| self.buf.size_hint(), I::size_hint)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where Self: Sized,
          F: FnMut(B, Self::Item) -> B,
    {
        match self.iter {
            Some(iter) => iter.fold(init, f),
            None => self.buf.fold(init, f),
        }
    }
}
impl<I: Iterator> DoubleEndedIterator for ForwardIter<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.backward().next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.backward().nth_back(n)
    }

    fn rfold<B, F>(mut self, init: B, f: F) -> B
    where Self: Sized,
          F: FnMut(B, Self::Item) -> B,
    {
        self.backward();
        self.buf.rfold(init, f)
    }
}

/// Ensure that side effects in iterators always run forward
pub trait ForwardIterExt: Iterator + Sized {
    /// Ensure that side effects in iterators always run forward
    ///
    /// Space complexity
    /// - call [`Iterator`] methods: *O*(1)
    /// - call [`DoubleEndedIterator`] methods: *O*(n)
    ///
    /// # Examples
    ///
    /// **Fail case**:
    /// ```rust,should_panic
    /// let mut i = 0;
    /// let x: Vec<_> = (0..4).map(|_| { i += 1; i }).rev().collect();
    /// assert_eq!(x, [4, 3, 2, 1]); // fail!
    /// ```
    ///
    /// **Rewrite to**:
    ///
    /// ```rust
    /// # use forward_iter::ForwardIterExt as _;
    /// let mut i = 0;
    /// let x: Vec<_> = (0..4).map(|_| { i += 1; i }).forward().rev().collect();
    /// assert_eq!(x, [4, 3, 2, 1]); // success!
    /// ```
    fn forward(self) -> ForwardIter<Self> {
        ForwardIter {
            iter: Some(self),
            buf: IntoIter::default(),
        }
    }
}
impl<I: Iterator> ForwardIterExt for I { }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some_next() {
        let mut i = 0;
        let iter = &mut (0..4).map(|_| { i += 1; i }).forward();
        assert_eq!(iter.next(), Some(1));
        let x: Vec<_> = iter.rev().collect();
        assert_eq!(x, [4, 3, 2]);
    }

    #[test]
    fn some_next2() {
        let mut i = 0;
        let iter = &mut (0..4).map(|_| { i += 1; i }).forward();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
