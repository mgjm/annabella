use std::{fmt, rc::Rc};

use quote::ToTokens;

use crate::{
    parser::Item,
    tokenizer::{Ident, Span, Spanned},
    Result,
};

#[macro_use]
mod c_code;

mod context;
mod expr;
mod ident;
mod item;
mod standard;
mod stmt;
mod ty;
mod type_item;
mod value;

pub use self::{c_code::CCode, context::Context, ident::IdentBuilder, ty::*, value::*};

pub fn run(items: Vec<Item>) -> Result<String> {
    let mut ctx = Context::base();
    let mut ctx = ctx.context();
    let ctx = &mut ctx;
    ctx.push_include("<stdio.h>");
    standard::generate(ctx)?;
    for item in &items {
        let code = item.generate(ctx)?;
        ctx.push_main(code);
    }

    let ident = Ident {
        name: "main".into(),
        span: Span::call_site(),
    };
    let value = ctx
        .get(&ident)?
        .expr_value()
        .filter_distinct(&ident, |value| {
            matches!(
                value.ty.as_function(),
                Some(ty) if ty.args.is_empty() && ty.return_type.is_void(),
            )
        })?;
    ctx.push_main(c_code! { #value(); });

    Ok(ctx.to_string())
}

trait CodeGenStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode>;
}

trait CodeGenType {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode>;
}

#[derive(Debug, Clone)]
enum ExprValue {
    Distinct(SingleExprValue),
    Ambiguous(Vec<SingleExprValue>),
    Dynamic(DynamicExprValue),
}

impl Spanned for ExprValue {
    fn span(&self) -> Span {
        Span::call_site()
    }
}

impl CodeGenExpr for ExprValue {
    fn generate(&self, _ctx: &mut Context) -> Result<ExprValue> {
        Ok(self.clone())
    }
}

impl ExprValue {
    fn new(iter: impl IntoIterator<Item = SingleExprValue>) -> Option<Self> {
        let mut values: Vec<_> = iter.into_iter().collect();
        match values.len() {
            0 => None,
            1 => Some(Self::Distinct(values.pop().unwrap())),
            _ => Some(Self::Ambiguous(values)),
        }
    }

    fn flat_map(self, mut f: impl FnMut(SingleExprValue) -> Result<Self>) -> Result<Self> {
        match self {
            Self::Distinct(value) => f(value),
            Self::Ambiguous(values) => {
                assert!(
                    values.len() >= 2,
                    "value is not ambiguous: {} options",
                    values.len()
                );
                let mut last_err = None;
                let mut new_values = Vec::new();
                for value in values {
                    match f(value) {
                        Ok(Self::Distinct(value)) => new_values.push(value),
                        Ok(Self::Ambiguous(values)) => new_values.extend(values),
                        Ok(Self::Dynamic(values)) => {
                            return Err(
                                values.unrecoverable_error("dynamic expression type not allowed")
                            )
                        }
                        Err(err) => {
                            last_err = Some(err);
                        }
                    }
                }
                Self::new(new_values).ok_or_else(|| last_err.unwrap())
            }
            Self::Dynamic(values) => {
                return Err(values.unrecoverable_error("dynamic expression type not allowed"))
            }
        }
    }

    fn filter_type(mut self, span: &impl Spanned, ty: &Type) -> Result<SingleExprValue> {
        Ok(loop {
            self = match self {
                ExprValue::Distinct(value) => {
                    if ty.can_assign(&value.ty) {
                        break value;
                    } else {
                        return Err(span.unrecoverable_error("expression type not allowed"));
                    }
                }
                ExprValue::Ambiguous(values) => {
                    let mut values = values.into_iter().filter(|value| ty.can_assign(&value.ty));
                    let Some(value) = values.next() else {
                        return Err(span.unrecoverable_error("expression type not allowed"));
                    };
                    if values.next().is_some() {
                        return Err(span.unrecoverable_error("ambiguous expression"));
                    } else {
                        break value;
                    }
                }
                ExprValue::Dynamic(value) => value.generate(ty)?,
            }
        })
    }

    fn filter_distinct(
        self,
        span: &impl Spanned,
        mut f: impl FnMut(&SingleExprValue) -> bool,
    ) -> Result<SingleExprValue> {
        Ok(match self {
            ExprValue::Distinct(value) => {
                if f(&value) {
                    value
                } else {
                    return Err(span.unrecoverable_error("expression type not allowed"));
                }
            }
            ExprValue::Ambiguous(values) => {
                let mut values = values.into_iter().filter(f);
                let Some(value) = values.next() else {
                    return Err(span.unrecoverable_error("expression type not allowed"));
                };
                if values.next().is_some() {
                    return Err(span.unrecoverable_error("ambiguous expression"));
                } else {
                    value
                }
            }
            ExprValue::Dynamic(_) => {
                return Err(span.unrecoverable_error("dynamic expression type not allowed"))
            }
        })
    }
}

impl From<SingleExprValue> for ExprValue {
    fn from(value: SingleExprValue) -> Self {
        Self::Distinct(value)
    }
}

impl From<DynamicExprValue> for ExprValue {
    fn from(value: DynamicExprValue) -> Self {
        Self::Dynamic(value)
    }
}

#[derive(Clone)]
struct DynamicExprValue {
    span: Span,
    generate: Rc<dyn Fn(&Type) -> Result<ExprValue>>,
}

impl fmt::Debug for DynamicExprValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DynamicExprValue").finish_non_exhaustive()
    }
}

impl Spanned for DynamicExprValue {
    fn span(&self) -> Span {
        self.span
    }
}

impl DynamicExprValue {
    fn new(span: Span, generate: impl Fn(&Type) -> Result<ExprValue> + 'static) -> Self {
        Self {
            span,
            generate: Rc::new(generate),
        }
    }

    fn generate(&self, ty: &Type) -> Result<ExprValue> {
        (self.generate)(ty)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Permission {
    Read,
    ReadWrite,
}

impl Permission {
    fn can_write(self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

#[derive(Debug, Clone)]
#[expect(dead_code)]
enum CompileTimeValue {
    Character(char),
    Boolean(bool),
    Integer(i64),
    String(String),
}

#[derive(Debug, Clone)]
struct SingleExprValue {
    ty: Type,
    perm: Permission,
    code: CCode,
    #[expect(dead_code)]
    value: Option<CompileTimeValue>,
}

impl SingleExprValue {
    fn with_check(self, ty: &Type) -> CCode {
        if let Some(constraint_check) = ty.needs_constraint_check(&self.ty) {
            let code = self.code;
            c_code! {
                #constraint_check(#code)
            }
        } else {
            self.code
        }
    }
}

impl ToTokens for SingleExprValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.code.to_tokens(tokens)
    }
}

trait CodeGenExpr: Spanned + Sized {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue>;

    fn generate_with_type_and_check(&self, ty: &Type, ctx: &mut Context) -> Result<CCode> {
        Ok(self.generate(ctx)?.filter_type(self, ty)?.with_check(ty))
    }

    fn generate_to_boolean(&self, ctx: &mut Context) -> Result<CCode> {
        self.generate_with_type_and_check(&Type::boolean(ctx)?, ctx)
    }

    fn generate_type(&self, ctx: &mut Context) -> Result<Type> {
        let _ = ctx;
        Err(self.unrecoverable_error(format!("not a type: {}", std::any::type_name::<Self>())))
    }
}

impl ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        proc_macro2::Ident::new(&self.name, proc_macro2::Span::call_site()).to_tokens(tokens)
    }
}
