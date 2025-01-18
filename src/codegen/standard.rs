use crate::{
    codegen::TypeValue,
    parser::Result,
    tokenizer::{Ident, Span},
    Token,
};

use super::{CCode, Context, FunctionType, FunctionValue, RcType, Type, Value};

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
                         Type::Function(FunctionType {
                            args: vec![Type::Integer.into(), Type::Integer.into()],
                            return_type: Type::Integer.into(),
                        })
                        .into(),
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
        (Type::Integer, "%d"),
        (Type::String, "%s"),
        (Type::Character, "%c"),
    ] {
        let ty: RcType = ty.into();
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

    Ok(())
}

pub fn generate_print(ty: RcType, fmt: &'static str, ctx: &mut Context) -> Result<()> {
    generate_custom_print(
        ty,
        c_code! {
            printf(#fmt "\n", self);
        },
        ctx,
    )
}

pub fn generate_custom_print(ty: RcType, code: CCode, ctx: &mut Context) -> Result<()> {
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
            Type::Function(FunctionType {
                args: vec![ty],
                return_type: Type::Void.into(),
            })
            .into(),
        )),
    )?;

    Ok(())
}
