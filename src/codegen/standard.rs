use crate::{
    parser::Result,
    tokenizer::{Ident, Span},
    Token,
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

    let print = Ident {
        name: "Print".into(),
        span: Span::call_site(),
    };
    for (ty, fmt) in [
        (Type::Integer, "%d"),
        (Type::String, "%s"),
        (Type::Character, "%c"),
    ] {
        let ident = quote::format_ident!("annabella_print_{}", ty.to_str());
        let ty_ident = quote::format_ident!("{}", ty.to_str());
        ctx.push_function(c_code! {
            void #ident(#ty_ident value) {
                printf(#fmt "\n", value);
            }
        });
        ctx.insert(
            &print,
            Value::Function(FunctionValue::new(
                c_code! { #ident },
                Type::Function(FunctionType {
                    args: vec![ty.into()],
                    return_type: Type::Void.into(),
                })
                .into(),
            )),
        )?;
    }

    Ok(())
}
