use std::{fmt, iter, rc::Rc};

use quote::ToTokens;

use crate::{
    tokenizer::{Ident, Spanned},
    Result,
};

use super::{Context, TypeValue, Value};

#[derive(Clone)]
pub struct Type(Rc<Inner>);

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! singleton {
    ($ident:ident) => {{
        thread_local! {
            static TYPE: Type = Type::new(Inner::$ident);
        }
        TYPE.with(Clone::clone)
    }};
}

impl Type {
    fn new(inner: Inner) -> Self {
        Self(Rc::new(inner))
    }

    pub fn void() -> Self {
        singleton!(Void)
    }

    pub fn boolean() -> Self {
        singleton!(Boolean)
    }

    pub fn character() -> Self {
        singleton!(Character)
    }

    pub fn integer() -> Self {
        singleton!(Integer)
    }

    pub fn string() -> Self {
        singleton!(String)
    }

    pub fn function(ty: FunctionType) -> Self {
        Self::new(Inner::Function(ty))
    }

    pub fn enum_(ty: EnumType) -> Self {
        Self::new(Inner::Enum(ty))
    }

    pub fn subtype(ty: SubtypeType) -> Self {
        Self::new(Inner::Subtype(ty))
    }

    pub fn parse_ident(ident: &Ident, ctx: &Context) -> Result<Self> {
        let value = Self::parse_ident_value(ident, ctx)?;
        Ok(value.ty.clone())
    }

    pub fn parse_ident_value<'a>(ident: &Ident, ctx: &'a Context<'a>) -> Result<&'a TypeValue> {
        let Value::Type(value) = ctx.get(ident)? else {
            return Err(ident.unrecoverable_error("not a type name"));
        };
        Ok(value)
    }

    fn inner(&self) -> &Inner {
        &self.0
    }

    pub fn is_void(&self) -> bool {
        matches!(self.inner(), Inner::Void)
    }

    pub fn as_function(&self) -> Option<&FunctionType> {
        match self.inner() {
            Inner::Function(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self.inner() {
            Inner::Void => "Void",
            Inner::Boolean => "Boolean",
            Inner::Character => "Character",
            Inner::Integer => "Integer",
            Inner::String => "String",
            Inner::Function(ty) => ty.to_str(),
            Inner::Enum(ty) => ty.to_str(),
            Inner::Subtype(ty) => ty.to_str(),
        }
    }

    pub fn has_same_parent(&self, other: &Self) -> bool {
        match self.inner() {
            Inner::Void => matches!(other.inner(), Inner::Void),
            Inner::Boolean => matches!(other.inner(), Inner::Boolean),
            Inner::Character => matches!(other.inner(), Inner::Character),
            Inner::Integer => matches!(other.inner(), Inner::Integer),
            Inner::String => matches!(other.inner(), Inner::String),
            Inner::Function(ty) => ty.has_same_parent(other),
            Inner::Enum(ty) => ty.has_same_parent(other),
            Inner::Subtype(ty) => ty.has_same_parent(other),
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        proc_macro2::Ident::new(self.to_str(), proc_macro2::Span::call_site()).to_tokens(tokens)
    }
}

#[derive(Debug)]
pub enum Inner {
    Void,
    Boolean,
    Character,
    Integer,
    String,
    Function(FunctionType),
    Enum(EnumType),
    Subtype(SubtypeType),
}

trait TypeImpl: fmt::Debug {
    fn to_str(&self) -> &str;
    fn has_same_parent(&self, other: &Type) -> bool;
}

#[derive(Debug)]
pub struct FunctionType {
    pub args: Vec<Type>,
    pub return_type: Type,
}

impl TypeImpl for FunctionType {
    fn to_str(&self) -> &str {
        "Function"
    }

    fn has_same_parent(&self, other: &Type) -> bool {
        let Some(other) = other.as_function() else {
            return false;
        };

        self.args.len() == other.args.len()
            && iter::zip(&self.args, &other.args).all(|(a, b)| a.has_same_parent(b))
            && self.return_type.has_same_parent(&other.return_type)
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

    fn has_same_parent(&self, mut other: &Type) -> bool {
        let other = loop {
            other = match other.inner() {
                Inner::Subtype(ty) => &ty.parent,
                Inner::Enum(ty) => break ty,
                _ => return false,
            };
        };

        self == other
    }
}

#[derive(Debug)]
pub struct SubtypeType {
    // pub name: Ident,
    pub parent: Type,
}

impl SubtypeType {
    fn parent(&self) -> &Type {
        let mut parent = &self.parent;
        while let Inner::Subtype(p) = parent.inner() {
            parent = &p.parent;
        }
        parent
    }
}

impl TypeImpl for SubtypeType {
    fn to_str(&self) -> &str {
        self.parent().to_str()
    }

    fn has_same_parent(&self, other: &Type) -> bool {
        self.parent().has_same_parent(other)
    }
}
