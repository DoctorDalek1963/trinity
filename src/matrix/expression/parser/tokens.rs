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
pub struct TokenList<'n, 'l: 'n> {
    /// The list of tokens themselves.
    pub tokens: &'l [Token<'n>],
}

impl<'n, 'l: 'n> TokenList<'n, 'l> {
    /// The empty [`TokenList`], primarily used for asserting parser behaviour.
    #[cfg(test)]
    pub const EMPTY: Self = Self { tokens: &[] };

    /// Create a new [`TokenList`] from this list of tokens.
    #[inline]
    pub fn new<'t: 'l>(tokens: &'t [Token<'n>]) -> Self {
        Self { tokens }
    }
}

impl<'n, 'l: 'n> InputLength for TokenList<'n, 'l> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'n, 'l: 'n> InputTake for TokenList<'n, 'l> {
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

impl<'n, 'l: 'n> InputIter for TokenList<'n, 'l> {
    type Item = &'l Token<'n>;
    type Iter = Enumerate<std::slice::Iter<'l, Token<'n>>>;
    type IterElem = std::slice::Iter<'l, Token<'n>>;

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

// impl<'n, 'l: 'n> Compare<TokenList<'n, 'l>> for TokenList<'n, 'l> {
//     fn compare(&self, other: TokenList<'n, 'l>) -> nom::CompareResult {
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
//     fn compare_no_case(&self, t: TokenList<'n, 'l>) -> nom::CompareResult {
//         self.compare(t)
//     }
// }

// impl InputLength for Token<'_> {
//     fn input_len(&self) -> usize {
//         1
//     }
// }
//
// impl InputTake for Token<'_> {
//     fn take(&self, _count: usize) -> Self {
//         *self
//     }
//
//     fn take_split(&self, _count: usize) -> (Self, Self) {
//         (*self, *self)
//     }
// }
