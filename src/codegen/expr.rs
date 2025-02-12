use std::iter;

use crate::{
    parser::{
        AggregateExpr, BaseName, BinaryOp, ComponentChoices, Expr, ExprBinary, ExprLit,
        ExprShortCircuit, FunctionCall, LitChar, LitNumber, LitStr, Name, QualifiedExpr,
        QualifiedExprValue, QualifiedExprValueExpr, RecordComponentAssociationList,
        SelectedComponent, ShortCircuitOp,
    },
    tokenizer::{Ident, Span, Spanned},
    Result,
};

use super::{
    ArgumentMode, CCode, CodeGenExpr, CompileTimeValue, Context, DynamicExprValue, ExprValue,
    Permission, SingleExprValue, Type,
};

impl CodeGenExpr for Expr {
    #[expect(unused_variables)]
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        match self {
            Self::Lit(expr) => expr.generate(ctx),
            Self::Name(expr) => expr.generate(ctx)?.implicit_dereference(ctx),
            Self::Qualified(expr) => expr.generate(ctx),
            Self::Aggregate(expr) => expr.generate(ctx),
            Self::Unary(expr) => todo!(),
            Self::Binary(expr) => expr.generate(ctx),
            Self::ShortCircuit(expr) => expr.generate(ctx),
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
            perm: Permission::Read,
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
            perm: Permission::Read,
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
            perm: Permission::Read,
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
            Self::Select(name) => name.generate(ctx),
            Self::FunctionCall(name) => name.generate(ctx),
        }
    }

    fn generate_type(&self, ctx: &mut Context) -> Result<Type> {
        match self {
            Self::Base(name) => name.generate_type(ctx),
            Self::Select(name) => name.generate_type(ctx),
            Self::FunctionCall(name) => name.generate_type(ctx),
        }
    }
}

impl CodeGenExpr for BaseName {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        Ok(match self {
            BaseName::Ident(ident) => ctx.get(ident)?.expr_value(),
        })
    }

    fn generate_type(&self, ctx: &mut Context) -> Result<Type> {
        Ok(match self {
            Self::Ident(ident) => Type::from_ident(ident, ctx)?,
        })
    }
}

impl CodeGenExpr for QualifiedExpr {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        let ty = self.mark.generate_type(ctx)?;
        let code = self.value.generate_with_type_and_check(&ty, ctx)?;
        Ok(SingleExprValue {
            ty,
            perm: Permission::Read,
            code,
            value: None,
        }
        .into())
    }
}

impl CodeGenExpr for QualifiedExprValue {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        match self {
            Self::Expr(expr) => expr.generate(ctx),
        }
    }
}

impl CodeGenExpr for QualifiedExprValueExpr {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        self.expr.generate(ctx)
    }
}

impl CodeGenExpr for AggregateExpr {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        match self {
            Self::Record(aggregate) => aggregate.generate(ctx),
        }
    }
}

impl CodeGenExpr for RecordComponentAssociationList {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        let associations = self
            .associations
            .iter()
            .map(|association| {
                let expr = association.expr.generate(ctx)?;
                Ok((
                    association.choices.as_ref().map(|(c, _)| c.clone()),
                    association.expr.span(),
                    expr,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let span = self.span();

        Ok(DynamicExprValue::new(span, move |ty| {
            let Some(record) = ty.as_record() else {
                return Err(span.unrecoverable_error("expected to be a record"));
            };

            let mut values: Vec<Option<CCode>> = vec![None; record.fields.len()];
            for (i, (choices, span, expr)) in associations.iter().enumerate() {
                let expr = |ty| Ok(expr.clone().filter_type(span, ty)?.with_check(ty));
                match choices {
                    None => {
                        values[i] = Some(expr(&record.fields[i].ty)?);
                    }
                    Some(ComponentChoices::Names(names)) => {
                        for name in names.iter() {
                            let Some((i, _, field)) = record.fields.get_full(&name.name) else {
                                return Err(name.unrecoverable_error("unknown field name"));
                            };
                            values[i] = Some(expr(&field.ty)?);
                        }
                    }
                    Some(ComponentChoices::Others(_)) => {
                        for (field, value) in record.fields.values().zip(&mut values) {
                            if value.is_none() {
                                *value = Some(expr(&field.ty)?);
                            }
                        }
                    }
                }
            }
            let values = iter::zip(record.fields.values(), values).map(|(field, v)| {
                let ty = &field.ty;
                v.unwrap_or_else(|| c_code! { (#ty){} })
            });

            Ok(SingleExprValue {
                ty: ty.clone(),
                perm: Permission::Read,
                code: c_code! {
                    (#ty){
                        #(#values,)*
                    }
                },
                value: None,
            }
            .into())
        })
        .into())
    }
}

impl CodeGenExpr for SelectedComponent {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        self.prefix
            .generate(ctx)?
            .flat_map(|prefix| prefix.ty.select(&prefix, &self.name))
    }
}

pub(super) fn generate_function_call<'a, A, E>(
    name: &Name,
    args: A,
    ctx: &mut Context,
) -> Result<ExprValue>
where
    A: ExactSizeIterator<Item = &'a E> + Clone,
    E: CodeGenExpr + 'static,
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
                .map(|(arg, arg_ty)| {
                    let code = arg.generate_with_type_and_check(&arg_ty.ty, ctx)?;
                    Ok(match arg_ty.mode {
                        ArgumentMode::In => code,
                        ArgumentMode::Out | ArgumentMode::InOut => c_code! { & #code },
                    })
                })
                .collect::<Result<Vec<_>>>()?
        };
        Ok(SingleExprValue {
            ty: ty.return_type.clone(),
            perm: Permission::Read,
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

impl CodeGenExpr for ExprShortCircuit {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue> {
        let boolean = Type::boolean(ctx)?;

        let lhs = self.lhs.generate_with_type_and_check(&boolean, ctx)?;
        let rhs = self.rhs.generate_with_type_and_check(&boolean, ctx)?;
        let op = match self.op {
            ShortCircuitOp::And(_) => c_code! { && },
            ShortCircuitOp::Or(_) => c_code! { || },
        };

        Ok(SingleExprValue {
            ty: boolean,
            perm: Permission::Read,
            code: c_code! { #lhs #op #rhs },
            value: None,
        }
        .into())
    }
}

impl ExprValue {
    fn implicit_dereference(self, _ctx: &mut Context) -> Result<ExprValue> {
        self.flat_map(|value| {
            Ok(match value.ty.as_function() {
                Some(f) if f.args.is_empty() => SingleExprValue {
                    ty: f.return_type.clone(),
                    perm: Permission::Read,
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
