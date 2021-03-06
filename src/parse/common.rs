use std::{
    ops::{BitAnd, BitOr},
    slice::IterMut,
};

use codemap::Spanned;

use crate::{value::Value, Token};

#[derive(Debug, Clone)]
pub(crate) struct NeverEmptyVec<T> {
    first: T,
    rest: Vec<T>,
}

impl<T> NeverEmptyVec<T> {
    pub const fn new(first: T) -> Self {
        Self {
            first,
            rest: Vec::new(),
        }
    }

    pub fn last(&self) -> &T {
        self.rest.last().unwrap_or(&self.first)
    }

    pub fn last_mut(&mut self) -> &mut T {
        self.rest.last_mut().unwrap_or(&mut self.first)
    }

    pub fn first(&mut self) -> &T {
        &self.first
    }

    pub fn first_mut(&mut self) -> &mut T {
        &mut self.first
    }

    pub fn push(&mut self, value: T) {
        self.rest.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.rest.pop()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.rest.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.rest.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }
}

/// A toplevel element beginning with something other than
/// `$`, `@`, `/`, whitespace, or a control character is either a
/// selector or a style.
#[derive(Debug)]
pub(super) enum SelectorOrStyle {
    Selector(String),
    Style(String, Option<Box<Spanned<Value>>>),
}

#[derive(Debug, Clone)]
pub(super) struct Branch {
    pub cond: Vec<Token>,
    pub toks: Vec<Token>,
}

impl Branch {
    pub fn new(cond: Vec<Token>, toks: Vec<Token>) -> Branch {
        Branch { cond, toks }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ContextFlags(u8);

pub(crate) struct ContextFlag(u8);

impl ContextFlags {
    pub const IN_MIXIN: ContextFlag = ContextFlag(1);
    pub const IN_FUNCTION: ContextFlag = ContextFlag(1 << 1);
    pub const IN_CONTROL_FLOW: ContextFlag = ContextFlag(1 << 2);
    pub const IN_KEYFRAMES: ContextFlag = ContextFlag(1 << 3);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn in_mixin(self) -> bool {
        (self.0 & Self::IN_MIXIN) != 0
    }

    pub fn in_function(self) -> bool {
        (self.0 & Self::IN_FUNCTION) != 0
    }

    pub fn in_control_flow(self) -> bool {
        (self.0 & Self::IN_CONTROL_FLOW) != 0
    }

    pub fn in_keyframes(self) -> bool {
        (self.0 & Self::IN_KEYFRAMES) != 0
    }
}

impl BitAnd<ContextFlag> for u8 {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: ContextFlag) -> Self::Output {
        self & rhs.0
    }
}

impl BitOr<ContextFlag> for ContextFlags {
    type Output = Self;
    fn bitor(self, rhs: ContextFlag) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
