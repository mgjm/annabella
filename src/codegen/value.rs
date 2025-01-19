use std::collections::{btree_map::Entry, BTreeMap};

use crate::{
    tokenizer::{Ident, Spanned},
    Result,
};

use super::{CCode, ExprValue, SingleExprValue, Type};

#[derive(Debug, Default)]
pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    values: BTreeMap<Box<str>, Value>,
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

    pub fn get_or_insert(&mut self, ident: &Ident, value: impl FnOnce() -> Value) -> &Value {
        self.values.entry(ident.name.clone()).or_insert_with(value)
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
    Type(TypeValue),
    Variable(VariableValue),
    Label(LabelValue),
}

impl Value {
    fn insert(&mut self, ident: &Ident, value: Self) -> Result<()> {
        let (Self::Function(this), Self::Function(other)) = (self, value) else {
            return Err(ident.unrecoverable_error("identifier already in use"));
        };
        this.insert(ident, other)
    }

    pub(super) fn expr_value(&self) -> ExprValue {
        match self {
            Self::Function(value) => value.expr_value(),
            Self::Type(_) => unreachable!(),
            Self::Variable(value) => value.expr_value(),
            Self::Label(_) => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionValue {
    overloads: Vec<FunctionOverload>,
}

impl FunctionValue {
    pub fn new(name: CCode, ty: Type) -> Self {
        Self {
            overloads: vec![FunctionOverload { name, ty }],
        }
    }

    fn insert(&mut self, ident: &Ident, value: Self) -> Result<()> {
        let _ = ident;
        self.overloads.extend(value.overloads);
        Ok(())
    }

    pub(super) fn expr_value(&self) -> ExprValue {
        ExprValue::new(self.overloads.iter().map(|ol| SingleExprValue {
            ty: ol.ty.clone(),
            code: ol.name.clone(),
            value: None,
        }))
        .unwrap()
    }
}

#[derive(Debug)]
struct FunctionOverload {
    pub name: CCode,
    pub ty: Type,
}

#[derive(Debug)]
pub struct TypeValue {
    pub ty: Type,
}

#[derive(Debug)]
pub struct VariableValue {
    pub name: CCode,
    pub ty: Type,
}

impl VariableValue {
    pub(super) fn expr_value(&self) -> ExprValue {
        SingleExprValue {
            ty: self.ty.clone(),
            code: self.name.clone(),
            value: None,
        }
        .into()
    }
}

#[derive(Debug)]
pub struct LabelValue {
    pub name: CCode,
}
