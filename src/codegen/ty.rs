use std::{collections::BTreeMap, fmt, mem, ptr, rc::Rc};

use quote::ToTokens;

use crate::{
    codegen::IdentBuilder,
    parser::{ParamMode, SelectorName},
    tokenizer::{Ident, Span, Spanned},
    Result,
};

use super::{CCode, Context, ExprValue, SingleExprValue, Value};

#[derive(Clone)]
pub struct Type(Rc<Inner>);

enum_dispatch!({
    #[derive(Debug)]
    enum Inner {
        Void(VoidType),
        Character(CharacterType),
        Integer(IntegerType),
        String(StringType),
        Function(FunctionType),
        Enum(EnumType),
        Signed(SignedType),
        Record(RecordType),
        Subtype(SubtypeType),
    }
});

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! singleton {
    ($ident:ident, $ty:ident $(, $name:literal)?) => {{
        thread_local! {
            static TYPE: Type = Type::new(Inner::$ident($ty $({
                ident: IdentBuilder::type_(&Ident {
                    name: $name.into(),
                    span: Span::call_site(),
                }),
            })?));
        }
        TYPE.with(Clone::clone)
    }};
}

impl Type {
    fn new(inner: Inner) -> Self {
        Self(Rc::new(inner))
    }

    pub fn void() -> Self {
        singleton!(Void, VoidType)
    }

    pub fn boolean(ctx: &mut Context<'_>) -> Result<Self> {
        thread_local! {
            static BOOLEAN: Ident = Ident {
                name: "boolean".into(),
                span: Span::call_site(),
            };
        }
        BOOLEAN.with(|ident| Self::from_ident(ident, ctx))
    }

    pub fn character() -> Self {
        singleton!(Character, CharacterType, "character")
    }

    pub fn integer() -> Self {
        singleton!(Integer, IntegerType, "integer")
    }

    pub fn string() -> Self {
        singleton!(String, StringType, "string")
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

    pub fn record(ty: RecordType) -> Self {
        Self::new(Inner::Record(ty))
    }

    pub fn subtype(ty: SubtypeType) -> Self {
        Self::new(Inner::Subtype(ty))
    }

    pub fn from_ident(ident: &Ident, ctx: &Context) -> Result<Self> {
        Self::from_value(ctx.get(ident)?)
            .ok_or_else(|| ident.unrecoverable_error("not a type name"))
    }

    pub fn from_value(value: &Value) -> Option<Self> {
        let Value::Type(value) = value else {
            return None;
        };
        Some(value.ty.clone())
    }

    fn inner(&self) -> &Inner {
        &self.0
    }

    fn last_parent(&self) -> &Self {
        let mut inner = self;
        while let Inner::Subtype(p) = inner.inner() {
            inner = &p.parent;
        }
        inner
    }

    fn last_parent_inner(&self) -> &Inner {
        self.last_parent().inner()
    }

    fn parents(&self) -> Parents {
        Parents(Some(self.inner()))
    }

    pub fn is_void(&self) -> bool {
        matches!(self.inner(), Inner::Void(_))
    }

    pub fn as_function(&self) -> Option<&FunctionType> {
        match self.inner() {
            Inner::Function(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        Inner!(self.inner(), |value| value.to_str())
    }

    /// Is it allowed to assign a `source` value to `self`?
    pub fn can_assign(&self, source: &Self) -> bool {
        Inner!(self.inner(), |value| value.can_assign(source))
    }

    /// Is a constraint check required when assigning a `source` value to `self`?
    pub fn needs_constraint_check(&self, source: &Self) -> Option<&CCode> {
        Inner!(self.inner(), |value| value.needs_constraint_check(source))
    }

    pub(super) fn select(
        &self,
        prefix: &SingleExprValue,
        name: &SelectorName,
    ) -> Result<ExprValue> {
        Inner!(self.inner(), |value| value.select(prefix, name))
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        Inner!(self.inner(), |value| value.to_tokens(tokens))
    }
}

struct Parents<'a>(Option<&'a Inner>);

impl<'a> Iterator for Parents<'a> {
    type Item = &'a Inner;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.0?.parent();
        mem::replace(&mut self.0, parent)
    }
}

impl Inner {
    fn parent(&self) -> Option<&Inner> {
        let Self::Subtype(ty) = self else {
            return None;
        };
        Some(ty.parent.inner())
    }
}

trait TypeImpl: fmt::Debug {
    fn to_str(&self) -> &str;
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream);
    fn can_assign(&self, source: &Type) -> bool;
    fn needs_constraint_check(&self, source: &Type) -> Option<&CCode>;
    fn select(&self, prefix: &SingleExprValue, name: &SelectorName) -> Result<ExprValue> {
        let _ = prefix;
        Err(name.unrecoverable_error("select not supported on this type"))
    }
}

#[derive(Debug)]
pub struct VoidType;

impl TypeImpl for VoidType {
    fn to_str(&self) -> &str {
        "void"
    }

    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {
        unimplemented!("void type to tokens")
    }

    fn can_assign(&self, source: &Type) -> bool {
        matches!(source.inner(), Inner::Void(_))
    }

    fn needs_constraint_check(&self, _source: &Type) -> Option<&CCode> {
        None
    }
}

#[derive(Debug)]
pub struct CharacterType {
    ident: proc_macro2::Ident,
}

impl TypeImpl for CharacterType {
    fn to_str(&self) -> &str {
        "character"
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
    }

    fn can_assign(&self, source: &Type) -> bool {
        matches!(source.inner(), Inner::Character(_))
    }

    fn needs_constraint_check(&self, _source: &Type) -> Option<&CCode> {
        None
    }
}

#[derive(Debug)]
pub struct IntegerType {
    ident: proc_macro2::Ident,
}

impl TypeImpl for IntegerType {
    fn to_str(&self) -> &str {
        "integer"
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
    }

    fn can_assign(&self, source: &Type) -> bool {
        matches!(source.inner(), Inner::Integer(_))
    }

    fn needs_constraint_check(&self, _source: &Type) -> Option<&CCode> {
        None
    }
}

#[derive(Debug)]
pub struct StringType {
    ident: proc_macro2::Ident,
}

impl TypeImpl for StringType {
    fn to_str(&self) -> &str {
        "string"
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
    }

    fn can_assign(&self, source: &Type) -> bool {
        matches!(source.inner(), Inner::String(_))
    }

    fn needs_constraint_check(&self, _source: &Type) -> Option<&CCode> {
        None
    }
}

#[derive(Debug)]
pub struct FunctionType {
    pub args: Vec<ArgumentType>,
    pub return_type: Type,
}

impl TypeImpl for FunctionType {
    fn to_str(&self) -> &str {
        "Function"
    }

    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {
        todo!("function type to tokens")
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
pub struct ArgumentType {
    pub ty: Type,
    pub mode: ArgumentMode,
}

#[derive(Debug)]
pub enum ArgumentMode {
    In,
    Out,
    InOut,
}

impl From<ParamMode> for ArgumentMode {
    fn from(value: ParamMode) -> Self {
        Self::from(&value)
    }
}

impl From<&ParamMode> for ArgumentMode {
    fn from(value: &ParamMode) -> Self {
        match value {
            ParamMode::In(_) => Self::In,
            ParamMode::Out(_) => Self::Out,
            ParamMode::InOut(_) => Self::InOut,
        }
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

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
    }

    fn can_assign(&self, source: &Type) -> bool {
        match source.last_parent_inner() {
            Inner::Enum(source) => ptr::eq(self, source),
            _ => false,
        }
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

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
    }

    fn can_assign(&self, source: &Type) -> bool {
        match source.last_parent_inner() {
            Inner::Integer(_) => true,
            Inner::Signed(source) => ptr::eq(self, source),
            _ => false,
        }
    }

    fn needs_constraint_check(&self, source: &Type) -> Option<&CCode> {
        if let Inner::Signed(source) = source.last_parent_inner() {
            if ptr::eq(self, source) {
                return None;
            }
        }
        self.constraint_check.as_ref()
    }
}

#[derive(Debug)]
pub struct RecordType {
    pub name: Ident,
    pub ident: proc_macro2::Ident,
    pub fields: BTreeMap<Box<str>, RecordField>,
}

impl TypeImpl for RecordType {
    fn to_str(&self) -> &str {
        &self.name.name
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
    }

    fn can_assign(&self, source: &Type) -> bool {
        match source.last_parent_inner() {
            Inner::Record(source) => ptr::eq(self, source),
            _ => false,
        }
    }

    fn needs_constraint_check(&self, _source: &Type) -> Option<&CCode> {
        None
    }

    fn select(&self, prefix: &SingleExprValue, name: &SelectorName) -> Result<ExprValue> {
        let Some(field) = self.fields.get(match name {
            SelectorName::Ident(ident) => &ident.name,
        }) else {
            return Err(name.unrecoverable_error("unknown field name"));
        };

        let ident = &field.ident;
        Ok(SingleExprValue {
            ty: field.ty.clone(),
            perm: prefix.perm,
            code: c_code! { (#prefix).#ident },
            value: None,
        }
        .into())
    }
}

#[derive(Debug)]
pub struct RecordField {
    pub ident: proc_macro2::Ident,
    pub ty: Type,
}

#[derive(Debug)]
pub struct SubtypeType {
    pub parent: Type,
    pub constraint_check: Option<CCode>,
}

impl SubtypeType {
    fn last_parent(&self) -> &Type {
        self.parent.last_parent()
    }
}

impl TypeImpl for SubtypeType {
    fn to_str(&self) -> &str {
        self.last_parent().to_str()
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.last_parent().to_tokens(tokens);
    }

    fn can_assign(&self, source: &Type) -> bool {
        self.last_parent().can_assign(source)
    }

    fn needs_constraint_check(&self, source: &Type) -> Option<&CCode> {
        for source in source.parents() {
            if let Inner::Subtype(source) = source {
                if ptr::eq(self, source) {
                    return None;
                }
            }
        }
        self.constraint_check.as_ref()
    }
}
