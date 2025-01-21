use crate::{
    codegen::TypeValue,
    parser::{EnumTypeDefinition, Expr, Range, SignedTypeDefinition},
    tokenizer::{Ident, Span},
    Result, Token,
};

use super::{CCode, CodeGenType, Context, FunctionType, FunctionValue, IdentBuilder, Type, Value};

pub fn generate(ctx: &mut Context) -> Result<()> {
    ctx.push_include("<stdlib.h>");
    ctx.push_function(c_code! {
        void throw_Constraint_Error() {
            fprintf(stderr, "Error: Constraint_Error\n");
            exit(1);
        }
    });

    generate_boolean(ctx)?;
    generate_integer(ctx)?;

    for (ty, code, fmt) in [
        (Type::string(), c_code! {char*}, "%s"),
        (Type::character(), c_code! {char}, "%c"),
    ] {
        let name = Ident {
            name: ty.to_str().into(),
            span: Span::call_site(),
        };
        ctx.push_type(c_code! {
            typedef #code #ty;
        });
        ctx.insert(&name, Value::Type(TypeValue { ty: ty.clone() }))?;
        generate_print(ty, fmt, ctx)?;
    }

    Ok(())
}

fn generate_boolean(ctx: &mut Context) -> Result<()> {
    let ident = Ident {
        name: "boolean".into(),
        span: Span::call_site(),
    };
    EnumTypeDefinition {
        values: [
            Ident {
                name: "false".into(),
                span: Span::call_site(),
            },
            Ident {
                name: "true".into(),
                span: Span::call_site(),
            },
        ]
        .into_iter()
        .collect(),
    }
    .generate(&ident, ctx)?;

    generate_boolean_logical_ops(&Type::boolean(ctx)?, ctx)
}

fn generate_integer(ctx: &mut Context) -> Result<()> {
    let ident = Ident {
        name: "integer".into(),
        span: Span::call_site(),
    };
    SignedTypeDefinition {
        range_keyword: Default::default(),
        range: Range {
            start: Expr::number(i32::MIN),
            dot_dot: Default::default(),
            end: Expr::number(i32::MAX),
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
        and &
        or |
        xor ^
    }

    generate_comparison_ops(ty, ctx)
}

pub(crate) fn generate_modular_ops(ty: &Type, modulus: &CCode, ctx: &mut Context) -> Result<()> {
    macro_rules! integer_ops {
        ($($ada:tt $c:tt {$($($map:tt)+)?})*) => {
            $(
                let op: Token![$ada] = Default::default();
                let ident = IdentBuilder::op_function(op, ty);
                let code = c_code! { (lhs $c rhs) };
                $(
                    let code = c_code! { (#code $($map)*) };
                )?
                ctx.push_function(c_code! {
                    #ty #ident(#ty lhs, #ty rhs) {
                        return #code % #modulus;
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
        + + {}
        - - { + #modulus}
        * * {}
        / / {}
        and & {}
        or | {}
        xor ^ {}
    }

    generate_comparison_ops(ty, ctx)
}

pub(crate) fn generate_boolean_logical_ops(ty: &Type, ctx: &mut Context) -> Result<()> {
    macro_rules! boolean_ops {
        ($($ada:tt $c:tt)*) => {
            $(
                let op: Token![$ada] = Default::default();
                let ident = IdentBuilder::op_function(op, ty);
                ctx.push_function(c_code! {
                    #ty #ident(#ty lhs, #ty rhs) {
                        return lhs $c rhs;
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

    boolean_ops! {
        and &&
        or ||
        xor ^
    }

    Ok(())
}

pub(crate) fn generate_comparison_ops(ty: &Type, ctx: &mut Context) -> Result<()> {
    let boolean = Type::boolean(ctx)?;
    macro_rules! integer_ops {
        ($($ada:tt $c:tt)*) => {
            $(
                let op: Token![$ada] = Default::default();
                let ident = IdentBuilder::op_function(op, ty);
                ctx.push_function(c_code! {
                    #boolean #ident(#ty lhs, #ty rhs) {
                        return lhs $c rhs;
                    }
                });

                ctx.insert(
                    &op.operator_symbol(),
                    Value::Function(FunctionValue::new(
                         c_code! { #ident },
                         Type::function(FunctionType {
                            args: vec![ty.clone(), ty.clone()],
                            return_type: boolean.clone(),
                        })
                    )),
                )?;

            )*
        };
    }

    integer_ops! {
        = ==
        /= !=
        < <
        <= <=
        > >
        >= >=
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

pub fn print() -> Ident {
    Ident {
        name: "print".into(),
        span: Span::call_site(),
    }
}

pub fn generate_custom_print(ty: Type, code: CCode, ctx: &mut Context) -> Result<()> {
    let print = print();

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
