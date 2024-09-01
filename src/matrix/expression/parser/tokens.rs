//! This module provides [`TokenList`] and implements various traits that make it compatible with
//! [`nom`] parsers.

use crate::matrix::expression::tokenise::Token;
use nom::{InputIter, InputLength, InputTake};
use std::iter::Enumerate;

/// A list of tokens.
///
/// The `'l` lifetime is the life of the list itself, and the `'n` lifetime is the life of the
/// [`MatrixName`](crate::matrix::MatrixName) variant in [`Token`].
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

impl<'l> InputLength for TokenList<'l> {
    fn input_len(&self) -> usize {
        self.tokens.len()
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

// impl<'l> Compare<TokenList<'l>> for TokenList<'l> {
//     fn compare(&self, other: Self) -> nom::CompareResult {
//         let min_length = self.tokens.len().min(other.tokens.len());
//         if &self.tokens[0..min_length] == &other.tokens[0..min_length] {
//             if self.tokens == other.tokens {
//                 nom::CompareResult::Ok
//             } else {
//                 nom::CompareResult::Incomplete
//             }
//         } else {
//             nom::CompareResult::Error
//         }
//     }
//
//     fn compare_no_case(&self, t: Self) -> nom::CompareResult {
//         self.compare(t)
//     }
// }

// impl InputLength for Token {
//     fn input_len(&self) -> usize {
//         1
//     }
// }
//
// impl InputTake for Token {
//     fn take(&self, _count: usize) -> Self {
//         *self
//     }
//
//     fn take_split(&self, _count: usize) -> (Self, Self) {
//         (*self, *self)
//     }
// }
