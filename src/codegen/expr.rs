use crate::{
    parser::{
        BaseName, BinaryOp, Expr, ExprBinary, ExprLit, FunctionCall, LitChar, LitNumber, LitStr,
        Name,
    },
    tokenizer::{Ident, Span, Spanned},
    Result,
};

use super::{CCode, CodeGenExpr, CompileTimeValue, Context, ExprValue, SingleExprValue, Type};

impl CodeGenExpr for Expr {
    #[expect(unused_variables)]
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        match self {
            Self::Lit(expr) => expr.generate(ctx),
            Self::Name(expr) => expr.generate(ctx)?.implicit_dereference(ctx),
            Self::Unary(expr) => todo!(),
            Self::Binary(expr) => expr.generate(ctx),
            Self::ShortCircuit(expr) => todo!(),
        }
    }
}

impl CodeGenExpr for ExprLit {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        match self {
            Self::Str(lit) => lit.generate(ctx),
            Self::Char(lit) => lit.generate(ctx),
            Self::Number(lit) => lit.generate(ctx),
        }
    }
}

impl CodeGenExpr for LitStr {
    fn generate(&self, _ctx: &mut Context) -> Result<ExprValue> {
        let str = self.str();
        Ok(SingleExprValue {
            ty: Type::string(),
            code: c_code! { #str },
            value: Some(CompileTimeValue::String(str)),
        }
        .into())
    }
}

impl CodeGenExpr for LitChar {
    fn generate(&self, _ctx: &mut Context) -> Result<ExprValue> {
        let char = self.char();
        Ok(SingleExprValue {
            ty: Type::character(),
            code: c_code! { #char },
            value: Some(CompileTimeValue::Character(char)),
        }
        .into())
    }
}

impl CodeGenExpr for LitNumber {
    fn generate(&self, _ctx: &mut Context) -> Result<ExprValue> {
        let num = self.number();
        Ok(SingleExprValue {
            ty: Type::integer(),
            code: c_code! { #num },
            value: Some(CompileTimeValue::Integer(num)),
        }
        .into())
    }
}

impl CodeGenExpr for Name {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        match self {
            Self::Base(name) => name.generate(ctx),
            Self::FunctionCall(name) => name.generate(ctx),
        }
    }
}

impl CodeGenExpr for BaseName {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        Ok(match self {
            BaseName::Ident(ident) => ctx.get(ident)?.expr_value(),
        })
    }
}

fn generate_function_call<'a, A>(name: &Name, args: A, ctx: &mut Context) -> Result<ExprValue>
where
    A: ExactSizeIterator<Item = &'a Expr> + Clone,
{
    let f = name.generate(ctx)?;
    f.flat_map(|f| {
        let Some(ty) = f.ty.as_function() else {
            return Err(name.unrecoverable_error("is not a function"));
        };

        let args = {
            let mut args = args.clone();

            let ty_num = ty.args.len();
            let arg_num = args.len();
            match arg_num.cmp(&ty_num) {
                std::cmp::Ordering::Less => {
                    return Err(name
                        .unrecoverable_error(format!("missing arguments: {arg_num} of {ty_num}")))
                }
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => {
                    return Err(args.nth(ty_num).unwrap().unrecoverable_error(format!(
                        "unexpected argument: {arg_num} of {ty_num}"
                    )))
                }
            }

            args.zip(&ty.args)
                .map(|(arg, ty)| arg.generate_with_type_and_check(ty, ctx))
                .collect::<Result<Vec<_>>>()?
        };
        Ok(SingleExprValue {
            ty: ty.return_type.clone(),
            code: c_code! {
                #f(#(#args),*)
            },
            value: None,
        }
        .into())
    })
}

impl CodeGenExpr for FunctionCall {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        generate_function_call(&self.name, self.args.iter(), ctx)
    }
}

impl ExprBinary {
    fn op_ident(&self) -> Ident {
        match self.op {
            BinaryOp::Pow(op) => op.operator_symbol(),
            BinaryOp::Mul(op) => op.operator_symbol(),
            BinaryOp::Div(op) => op.operator_symbol(),
            BinaryOp::Mod(op) => op.operator_symbol(),
            BinaryOp::Rem(op) => op.operator_symbol(),
            BinaryOp::Add(op) => op.operator_symbol(),
            BinaryOp::Sub(op) => op.operator_symbol(),
            BinaryOp::Concat(op) => op.operator_symbol(),
            BinaryOp::Eq(op) => op.operator_symbol(),
            BinaryOp::Ne(op) => op.operator_symbol(),
            BinaryOp::Lt(op) => op.operator_symbol(),
            BinaryOp::Le(op) => op.operator_symbol(),
            BinaryOp::Gt(op) => op.operator_symbol(),
            BinaryOp::Ge(op) => op.operator_symbol(),
            BinaryOp::And(op) => op.operator_symbol(),
            BinaryOp::Or(op) => op.operator_symbol(),
            BinaryOp::Xor(op) => op.operator_symbol(),
        }
    }
}

impl CodeGenExpr for ExprBinary {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        generate_function_call(
            &Name::Base(BaseName::Ident(self.op_ident())),
            [&*self.lhs, &*self.rhs].into_iter(),
            ctx,
        )
    }
}

impl ExprValue {
    fn implicit_dereference(self, _ctx: &mut Context) -> Result<ExprValue> {
        self.flat_map(|value| {
            Ok(match value.ty.as_function() {
                Some(f) if f.args.is_empty() => SingleExprValue {
                    ty: f.return_type.clone(),
                    code: c_code! { #value() },
                    value: None,
                }
                .into(),
                Some(_) => {
                    return Err(Span::call_site()
                        .unrecoverable_error("implicit dereference on function with arguments"))
                }
                _ => value.into(),
            })
        })
    }
}
