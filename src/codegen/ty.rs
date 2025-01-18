use std::{fmt, ops::Deref, rc::Rc};

use crate::{
    parser::Result,
    tokenizer::{Ident, Spanned},
};

use super::{Context, Value};

#[derive(Clone, PartialEq, Eq)]
pub struct RcType(Rc<Type>);

impl fmt::Debug for RcType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for RcType {
    type Target = Type;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Type> for RcType {
    fn from(value: Type) -> Self {
        thread_local! {
            static VOID: Rc<Type> = Rc::new(Type::Void);
            static BOOL: Rc<Type> = Rc::new(Type::Bool);
            static CHAR: Rc<Type> = Rc::new(Type::Character);
            static INTEGER: Rc<Type> = Rc::new(Type::Integer);
            static STRING: Rc<Type> = Rc::new(Type::String);
        }
        let singleton = match value {
            Type::Void => &VOID,
            Type::Bool => &BOOL,
            Type::Character => &CHAR,
            Type::Integer => &INTEGER,
            Type::String => &STRING,
            _ => return Self(Rc::new(value)),
        };
        Self(singleton.with(Clone::clone))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Bool,
    Character,
    Integer,
    String,
    Function(FunctionType),
    Enum(EnumType),
}

impl Type {
    pub fn void() -> RcType {
        Self::Void.into()
    }

    pub fn function(ty: FunctionType) -> RcType {
        Self::Function(ty).into()
    }

    pub fn enum_(ty: EnumType) -> RcType {
        Self::Enum(ty).into()
    }

    pub fn parse_ident(ident: &Ident, ctx: &Context) -> Result<RcType> {
        let Value::Type(value) = ctx.get(ident)? else {
            return Err(ident.unrecoverable_error("not a type name"));
        };
        Ok(value.ty.clone())
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Void => "Void",
            Self::Bool => "Bool",
            Self::Character => "Character",
            Self::Integer => "Integer",
            Self::String => "String",
            Self::Function(ty) => ty.to_str(),
            Self::Enum(ty) => ty.to_str(),
        }
    }
}

trait TypeImpl: fmt::Debug {
    fn to_str(&self) -> &str;
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionType {
    pub args: Vec<RcType>,
    pub return_type: RcType,
}

impl TypeImpl for FunctionType {
    fn to_str(&self) -> &str {
        "Function"
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EnumType {
    pub name: Ident,
    pub values: Vec<Ident>,
}

impl TypeImpl for EnumType {
    fn to_str(&self) -> &str {
        &self.name.name
    }
}
