use std::collections::{hash_map::Entry, HashMap};

use crate::{
    parser::Result,
    tokenizer::{Ident, Spanned},
};

use super::{CCode, RcType};

#[derive(Debug, Default)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    values: HashMap<Box<str>, Value>,
}

impl Scope<'_> {
    pub fn subscope(&mut self) -> Scope {
        Scope {
            parent: Some(self),
            values: Default::default(),
        }
    }

    pub fn insert(&mut self, ident: &Ident, value: Value) -> Result<()> {
        match self.values.entry(ident.name.clone()) {
            Entry::Occupied(entry) => entry.into_mut().insert(ident, value),
            Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    pub fn get(&self, ident: &Ident) -> Result<&Value> {
        let mut this = Some(self);
        while let Some(scope) = this {
            if let Some(value) = scope.values.get(&ident.name) {
                return Ok(value);
            }
            this = scope.parent;
        }
        Err(ident.unrecoverable_error("identifier not in scope"))
    }
}

#[derive(Debug)]
pub enum Value {
    Function(FunctionValue),
    Variable(VariableValue),
}

impl Value {
    fn insert(&mut self, ident: &Ident, value: Self) -> Result<()> {
        let (Self::Function(this), Self::Function(other)) = (self, value) else {
            return Err(ident.unrecoverable_error("identifier already inuse"));
        };
        this.insert(ident, other)
    }

    pub fn ty(&self) -> RcType {
        match self {
            Self::Function(value) => value.ty(),
            Self::Variable(value) => value.ty(),
        }
    }

    pub fn code(&self) -> CCode {
        match self {
            Self::Function(value) => value.code(),
            Self::Variable(value) => value.code(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionValue {
    pub name: CCode,
    pub ty: RcType,
}

impl FunctionValue {
    fn insert(&mut self, ident: &Ident, value: Self) -> Result<()> {
        let _ = value;
        Err(ident.unrecoverable_error("function overloading not yet implemented"))
    }

    fn ty(&self) -> RcType {
        self.ty.clone()
    }

    fn code(&self) -> CCode {
        self.name.clone()
    }
}

#[derive(Debug)]
pub struct VariableValue {
    pub name: CCode,
    pub ty: RcType,
}

impl VariableValue {
    fn ty(&self) -> RcType {
        self.ty.clone()
    }

    fn code(&self) -> CCode {
        self.name.clone()
    }
}
