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
    let Some(ExprValue::Distinct(value)) = ctx.get(&ident)?.expr_value().filter(|value| {
        matches!(
            value.ty.as_function(),
            Some(ty) if ty.args.is_empty() && ty.return_type.is_void(),
        )
    }) else {
        return Err(ident.unrecoverable_error("main function not found"));
    };
    ctx.push_main(c_code! { #value(); });

    Ok(ctx.to_string())
}

trait CodeGenStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode>;
}

trait CodeGenType {
    fn generate(&self, name: &Ident, ctx: &mut Context) -> Result<CCode>;
}

#[derive(Debug)]
enum ExprValue {
    Distinct(SingleExprValue),
    Ambiguous(Vec<SingleExprValue>),
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
                        Err(err) => {
                            last_err = Some(err);
                        }
                    }
                }
                Self::new(new_values).ok_or_else(|| last_err.unwrap())
            }
        }
    }

    fn filter(self, mut f: impl FnMut(&SingleExprValue) -> bool) -> Option<Self> {
        match self {
            Self::Distinct(value) => {
                if f(&value) {
                    Some(Self::Distinct(value))
                } else {
                    None
                }
            }
            Self::Ambiguous(values) => Self::new(values.into_iter().filter(f)),
        }
    }

    // fn as_slice(&self) -> &[SingleExprValue] {
    //     match self {
    //         Self::Distinct(value) => std::slice::from_ref(value),
    //         Self::Ambiguous(values) => values,
    //     }
    // }

    // fn iter(&self) -> impl Iterator<Item = &SingleExprValue> {
    //     self.as_slice().iter()
    // }

    // fn all_combinations(
    //     list: Vec<Self>,
    //     mut f: impl FnMut(&[&SingleExprValue]) -> Result<Self>,
    // ) -> Result<Self> {
    //     if list.iter().all(|item| matches!(*item, Self::Ambiguous(_))) {
    //         let current: Vec<_> = list
    //             .iter()
    //             .map(|item| match item {
    //                 Self::Distinct(value) => value,
    //                 Self::Ambiguous(_) => unreachable!(),
    //             })
    //             .collect();
    //         return f(&current);
    //     }

    //     struct ResettableIter<'a, T> {
    //         values: &'a [T],
    //         index: usize,
    //     }

    //     impl<'a, T> ResettableIter<'a, T> {
    //         fn next(&mut self) -> Option<&'a T> {
    //             let value = self.values.get(self.index)?;
    //             self.index += 1;
    //             Some(value)
    //         }

    //         fn reset(&mut self) -> &'a T {
    //             self.index = 0;
    //             self.next().unwrap()
    //         }
    //     }

    //     let mut iters: Vec<_> = list
    //         .iter()
    //         .map(|item| ResettableIter {
    //             values: item.as_slice(),
    //             index: 0,
    //         })
    //         .collect();
    //     let mut current: Vec<_> = iters.iter_mut().map(|iter| iter.next().unwrap()).collect();

    //     let mut last_err = None;
    //     let mut new_values = Vec::new();
    //     'outer: loop {
    //         match f(&current) {
    //             Ok(Self::Distinct(value)) => new_values.push(value),
    //             Ok(Self::Ambiguous(values)) => new_values.extend(values),
    //             Err(err) => {
    //                 last_err = Some(err);
    //             }
    //         }

    //         for (iter, value) in std::iter::zip(&mut iters, &mut current) {
    //             if let Some(v) = iter.next() {
    //                 *value = v;
    //                 continue 'outer;
    //             } else {
    //                 *value = iter.reset();
    //             }
    //         }
    //         break;
    //     }
    //     Self::new(new_values).ok_or_else(|| last_err.unwrap())
    // }
}

impl From<SingleExprValue> for ExprValue {
    fn from(value: SingleExprValue) -> Self {
        Self::Distinct(value)
    }
}

#[derive(Debug)]
#[expect(dead_code)]
enum CompileTimeValue {
    Character(char),
    Boolean(bool),
    Integer(i64),
    String(String),
}

#[derive(Debug)]
struct SingleExprValue {
    ty: Type,
    code: CCode,
    #[expect(dead_code)]
    value: Option<CompileTimeValue>,
}

impl ToTokens for SingleExprValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.code.to_tokens(tokens)
    }
}

trait CodeGenExpr: Spanned {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue>;

    fn generate_with_type_and_check(&self, ty: &Type, ctx: &mut Context) -> Result<CCode> {
        let expr = match self.generate(ctx)?.filter(|value| ty.can_assign(&value.ty)) {
            Some(ExprValue::Distinct(expr)) => expr,
            Some(ExprValue::Ambiguous(_)) => {
                return Err(self.unrecoverable_error("ambiguous expression"));
            }
            None => {
                return Err(self.unrecoverable_error("expression type not allowed"));
            }
        };
        if let Some(constraint_check) = ty.needs_constraint_check(&expr.ty) {
            let code = expr.code;
            Ok(c_code! {
                #constraint_check(#code)
            })
        } else {
            Ok(expr.code)
        }
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
