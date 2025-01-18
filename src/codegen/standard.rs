use crate::{
    codegen::TypeValue,
    tokenizer::{Ident, Span},
    Result, Token,
};

use super::{CCode, Context, FunctionType, FunctionValue, Type, Value};

pub fn generate(ctx: &mut Context) -> Result<()> {
    ctx.push_type(c_code! {
        typedef char* String;
        typedef int Integer;
        typedef char Character;
    });

    macro_rules! integer_ops {
        ($($ada:tt $c:tt)*) => {
            $(
                let op: Token![$ada] = Default::default();
                let ident = quote::format_ident!("annabella__op_{op:?}__Integer__Integer");
                ctx.push_function(c_code! {
                    Integer #ident(Integer lhs, Integer rhs) {
                        return lhs $c rhs;
                    }
                });

                ctx.insert(
                    &op.operator_symbol(),
                    Value::Function(FunctionValue::new(
                         c_code! { #ident },
                         Type::function(FunctionType {
                            args: vec![Type::integer(), Type::integer()],
                            return_type: Type::integer(),
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

    for (ty, fmt) in [
        (Type::integer(), "%d"),
        (Type::string(), "%s"),
        (Type::character(), "%c"),
    ] {
        let ident = Ident {
            name: ty.to_str().into(),
            span: Span::call_site(),
        };
        ctx.insert(
            &ident,
            Value::Type(TypeValue {
                name: c_code! { #ident },
                ty: ty.clone(),
            }),
        )?;
        generate_print(ty, fmt, ctx)?;
    }

    ctx.push_include("<stdlib.h>");
    ctx.push_function(c_code! {
        void throw_Constraint_Error() {
            fprintf(stderr, "Error: Constraint_Error\n");
            exit(1);
        }
    });

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

    let ident = quote::format_ident!("annabella_print_{}", ty.to_str());
    let ty_ident = quote::format_ident!("{}", ty.to_str());
    ctx.push_function(c_code! {
        void #ident(#ty_ident self) {
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
