use crate::{
    codegen::TypeValue,
    parser::{Expr, ExprLit, LitNumber, Range, SignedTypeDefinition},
    tokenizer::{Ident, Span},
    Result, Token,
};

use super::{CCode, CodeGenType, Context, FunctionType, FunctionValue, IdentBuilder, Type, Value};

pub fn generate(ctx: &mut Context) -> Result<()> {
    ctx.push_type(c_code! {
        typedef char* String;
        typedef char Character;
    });

    ctx.push_include("<stdlib.h>");
    ctx.push_function(c_code! {
        void throw_Constraint_Error() {
            fprintf(stderr, "Error: Constraint_Error\n");
            exit(1);
        }
    });

    generate_integer(ctx)?;

    for (ty, fmt) in [(Type::string(), "%s"), (Type::character(), "%c")] {
        let ident = Ident {
            name: ty.to_str().into(),
            span: Span::call_site(),
        };
        ctx.insert(&ident, Value::Type(TypeValue { ty: ty.clone() }))?;
        generate_print(ty, fmt, ctx)?;
    }

    Ok(())
}

fn generate_integer(ctx: &mut Context) -> Result<()> {
    let ident = Ident {
        name: "Integer".into(),
        span: Span::call_site(),
    };
    SignedTypeDefinition {
        range_keyword: Default::default(),
        range: Range {
            start: Expr::Lit(ExprLit::Number(LitNumber {
                lit: crate::tokenizer::Literal {
                    str: i16::MIN.to_string().into(),
                    span: Span::call_site(),
                },
            })),
            dot_dot: Default::default(),
            end: Expr::Lit(ExprLit::Number(LitNumber {
                lit: crate::tokenizer::Literal {
                    str: i16::MAX.to_string().into(),
                    span: Span::call_site(),
                },
            })),
        },
    }
    .generate(&ident, ctx)?;

    Ok(())
}

pub(crate) fn generate_signed_ops(ty: &Type, ctx: &mut Context) -> Result<()> {
    macro_rules! integer_ops {
        ($($ada:tt $c:tt)*) => {
            $(
                let op: Token![$ada] = Default::default();
                let ident = IdentBuilder::op_function(op, ty);
                let mut code = c_code! { lhs $c rhs };
                if let Some(constraint_check) = ty.needs_constraint_check(&Type::integer()) {
                    code = c_code! { #constraint_check(#code) }
                }
                ctx.push_function(c_code! {
                    #ty #ident(#ty lhs, #ty rhs) {
                        return #code;
                    }
                });

                ctx.insert(
                    &op.operator_symbol(),
                    Value::Function(FunctionValue::new(
                         c_code! { #ident },
                         Type::function(FunctionType {
                            args: vec![ty.clone(), ty.clone()],
                            return_type: ty.clone(),
                        })
                    )),
                )?;

            )*
        };
    }

    integer_ops! {
        + +
        - -
        * *
        / /
        = ==
        /= !=
        < <
        <= <=
        > >
        >= >=
        and &
        or |
        xor ^
    }

    Ok(())
}

pub fn generate_print(ty: Type, fmt: &'static str, ctx: &mut Context) -> Result<()> {
    generate_custom_print(
        ty,
        c_code! {
            printf(#fmt "\n", self);
        },
        ctx,
    )
}

pub fn generate_custom_print(ty: Type, code: CCode, ctx: &mut Context) -> Result<()> {
    let print = Ident {
        name: "Print".into(),
        span: Span::call_site(),
    };

    let ident = IdentBuilder::print(&ty);
    ctx.push_function(c_code! {
        void #ident(#ty self) {
            #code
        }
    });

    ctx.insert(
        &print,
        Value::Function(FunctionValue::new(
            c_code! { #ident },
            Type::function(FunctionType {
                args: vec![ty],
                return_type: Type::void(),
            }),
        )),
    )?;

    Ok(())
}
