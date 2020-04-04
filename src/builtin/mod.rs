use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::args::CallArgs;
use crate::error::SassResult;
use crate::scope::Scope;
use crate::value::Value;

#[macro_use]
mod macros;

mod color;
mod list;
mod map;
mod math;
mod meta;
mod selector;
mod string;

static FUNCTION_COUNT: AtomicUsize = AtomicUsize::new(0);

// TODO: impl Fn
#[derive(Clone)]
pub(crate) struct Builtin(pub fn(CallArgs, &Scope) -> SassResult<Value>, usize);
impl Builtin {
    pub fn new(body: fn(CallArgs, &Scope) -> SassResult<Value>) -> Builtin {
        let count = FUNCTION_COUNT.fetch_add(1, Ordering::Relaxed);
        Self(body, count)
    }
}

impl PartialEq for Builtin {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

pub(crate) static GLOBAL_FUNCTIONS: Lazy<HashMap<String, Builtin>> = Lazy::new(|| {
    let mut m = HashMap::new();
    color::register(&mut m);
    list::register(&mut m);
    map::register(&mut m);
    math::register(&mut m);
    meta::register(&mut m);
    string::register(&mut m);
    m
});
