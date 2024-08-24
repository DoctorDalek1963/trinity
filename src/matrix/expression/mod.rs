//! This module handles written expressions of matrices.
//!
//! We take an expression string, tokenise it, turn it into an AST (see
//! [`AstNode`](self::ast::AstNode)), and then [`evaulate`](self::ast::AstNode::evaluate) it.

pub mod ast;
