use std::{fmt, ptr, rc::Rc};

use quote::ToTokens;

use crate::{
    tokenizer::{Ident, Spanned},
    Result,
};

use super::{CCode, Context, TypeValue, Value};

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

    pub fn signed(ty: SignedType) -> Self {
        Self::new(Inner::Signed(ty))
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
            Inner::Signed(ty) => ty.to_str(),
            Inner::Subtype(ty) => ty.to_str(),
        }
    }

    /// Is it allowed to assign a `source` value to `self`?
    pub fn can_assign(&self, source: &Self) -> bool {
        match self.inner() {
            Inner::Void => matches!(source.inner(), Inner::Void),
            Inner::Boolean => matches!(source.inner(), Inner::Boolean),
            Inner::Character => matches!(source.inner(), Inner::Character),
            Inner::Integer => matches!(source.inner(), Inner::Integer),
            Inner::String => matches!(source.inner(), Inner::String),
            Inner::Function(ty) => ty.can_assign(source),
            Inner::Enum(ty) => ty.can_assign(source),
            Inner::Signed(ty) => ty.can_assign(source),
            Inner::Subtype(ty) => ty.can_assign(source),
        }
    }

    /// Is a constraint check required when assigning a `source` value to `self`?
    pub fn needs_constraint_check(&self, source: &Self) -> Option<&CCode> {
        match self.inner() {
            Inner::Void => None,
            Inner::Boolean => None,
            Inner::Character => None,
            Inner::Integer => None,
            Inner::String => None,
            Inner::Function(ty) => ty.needs_constraint_check(source),
            Inner::Enum(ty) => ty.needs_constraint_check(source),
            Inner::Signed(ty) => ty.needs_constraint_check(source),
            Inner::Subtype(ty) => ty.needs_constraint_check(source),
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let new_ident = || proc_macro2::Ident::new(self.to_str(), proc_macro2::Span::call_site());
        match self.inner() {
            Inner::Void => new_ident().to_tokens(tokens),
            Inner::Boolean => new_ident().to_tokens(tokens),
            Inner::Character => new_ident().to_tokens(tokens),
            Inner::Integer => new_ident().to_tokens(tokens),
            Inner::String => new_ident().to_tokens(tokens),
            Inner::Function(_) => new_ident().to_tokens(tokens),
            Inner::Enum(ty) => ty.ident.to_tokens(tokens),
            Inner::Signed(ty) => ty.ident.to_tokens(tokens),
            Inner::Subtype(ty) => ty.parent().to_tokens(tokens),
        }
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
    Signed(SignedType),
    Subtype(SubtypeType),
}

trait TypeImpl: fmt::Debug {
    fn to_str(&self) -> &str;
    fn can_assign(&self, source: &Type) -> bool;
    fn needs_constraint_check(&self, source: &Type) -> Option<&CCode>;
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

    fn can_assign(&self, source: &Type) -> bool {
        let Some(source) = source.as_function() else {
            return false;
        };

        ptr::eq(self, source)
    }

    fn needs_constraint_check(&self, _target: &Type) -> Option<&CCode> {
        None
    }
}

#[derive(Debug)]
pub struct EnumType {
    pub name: Ident,
    pub ident: proc_macro2::Ident,
    pub values: Vec<Ident>,
}

impl TypeImpl for EnumType {
    fn to_str(&self) -> &str {
        &self.name.name
    }

    fn can_assign(&self, mut source: &Type) -> bool {
        let source = loop {
            source = match source.inner() {
                Inner::Subtype(ty) => &ty.parent,
                Inner::Enum(ty) => break ty,
                _ => return false,
            };
        };

        ptr::eq(self, source)
    }

    fn needs_constraint_check(&self, _target: &Type) -> Option<&CCode> {
        None
    }
}

#[derive(Debug)]
pub struct SignedType {
    pub name: Ident,
    pub ident: proc_macro2::Ident,
    pub constraint_check: Option<CCode>,
}

impl TypeImpl for SignedType {
    fn to_str(&self) -> &str {
        &self.name.name
    }

    fn can_assign(&self, mut source: &Type) -> bool {
        let source = loop {
            source = match source.inner() {
                Inner::Integer => return true,
                Inner::Subtype(ty) => &ty.parent,
                Inner::Signed(ty) => break ty,
                _ => return false,
            };
        };

        ptr::eq(self, source)
    }

    fn needs_constraint_check(&self, source: &Type) -> Option<&CCode> {
        if let Inner::Signed(source) = source.inner() {
            if ptr::eq(self, source) {
                return None;
            }
        }
        self.constraint_check.as_ref()
    }
}

#[derive(Debug)]
pub struct SubtypeType {
    pub parent: Type,
    pub constraint_check: Option<CCode>,
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

    fn can_assign(&self, source: &Type) -> bool {
        self.parent().can_assign(source)
    }

    fn needs_constraint_check(&self, source: &Type) -> Option<&CCode> {
        if let Inner::Subtype(source) = source.inner() {
            if ptr::eq(self, source) {
                return None;
            }
        }
        self.constraint_check.as_ref()
    }
}
