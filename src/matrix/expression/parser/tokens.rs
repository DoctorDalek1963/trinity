//! This module provides [`TokenList`] and implements various traits that make it compatible with
//! [`nom`] parsers.

use crate::matrix::expression::tokenise::Token;
use nom::{InputIter, InputTake};
use std::iter::Enumerate;

/// A list of tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TokenList<'l> {
    /// The list of tokens themselves.
    pub tokens: &'l [Token],
}

impl<'l> TokenList<'l> {
    /// The empty [`TokenList`], primarily used for asserting parser behaviour.
    #[cfg(test)]
    pub const EMPTY: Self = Self { tokens: &[] };

    /// Create a new [`TokenList`] from this list of tokens.
    #[inline]
    pub fn new<'t: 'l>(tokens: &'t [Token]) -> Self {
        Self { tokens }
    }
}

impl<'l> InputTake for TokenList<'l> {
    fn take(&self, count: usize) -> Self {
        Self {
            tokens: &self.tokens[0..count],
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (first, second) = self.tokens.split_at(count);
        (TokenList { tokens: second }, TokenList { tokens: first })
    }
}

impl<'l> InputIter for TokenList<'l> {
    type Item = &'l Token;
    type Iter = Enumerate<std::slice::Iter<'l, Token>>;
    type IterElem = std::slice::Iter<'l, Token>;

    fn iter_indices(&self) -> Self::Iter {
        self.tokens.iter().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.tokens.iter()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tokens.iter().position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.tokens.len() >= count {
            Ok(count)
        } else {
            Err(nom::Needed::Unknown)
        }
    }
}
