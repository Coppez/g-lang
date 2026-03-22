use super::super::super::eval::Evaluator;
use crate::{ast::ast::Ident, interpreter::obj::Object};

impl Evaluator {
    pub fn register_ident(&mut self, ident: Ident, object: Object) -> Object {
        self.env.lock().unwrap().set(&ident, object.clone());
        object
    }
}
