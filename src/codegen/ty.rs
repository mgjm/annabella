use std::{fmt, ops::Deref, rc::Rc};

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
}

impl Type {
    pub fn parse(s: &str) -> Option<RcType> {
        Some(
            match s {
                "Bool" => Self::Bool,
                "Character" => Self::Character,
                "Integer" => Self::Integer,
                "String" => Self::String,
                _ => return None,
            }
            .into(),
        )
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Void => "Void",
            Self::Bool => "Bool",
            Self::Character => "Character",
            Self::Integer => "Integer",
            Self::String => "String",
            Self::Function(ty) => ty.to_str(),
        }
    }

    pub fn fmt(&self) -> (&str, bool) {
        match self {
            Self::Void => ("<void>", false),
            Self::Bool => ("%d", true),
            Self::Character => ("%c", true),
            Self::Integer => ("%d", true),
            Self::String => ("%s", true),
            Self::Function(ty) => ty.c_fmt_str(),
        }
    }
}

trait TypeImpl: fmt::Debug {
    fn to_str(&self) -> &str;
    fn c_fmt_str(&self) -> (&str, bool);
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

    fn c_fmt_str(&self) -> (&str, bool) {
        ("<function>", false)
    }
}
