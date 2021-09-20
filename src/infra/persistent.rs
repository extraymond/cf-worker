pub mod counter_adapter;
pub mod counter_document;

use worker::*;

pub struct Shared<T>(pub T);

pub type Context = Env;
pub type ContextDo = (State, Env);

unsafe impl<T> Sync for Shared<T> {}
unsafe impl<T> Send for Shared<T> {}
