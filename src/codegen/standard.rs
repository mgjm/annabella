use crate::{parser::Result, Token};

use super::{CCode, Context, FunctionType, FunctionValue, Type, Value};

pub fn generate(ctx: &mut Context) -> Result<()> {
    ctx.push_type(c_code! {
        typedef char* String;
        typedef int Integer;
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
                    Value::Function(FunctionValue {
                        name: c_code! { #ident },
                        ty: Type::Function(FunctionType {
                            args: vec![Type::Integer.into(), Type::Integer.into()],
                            return_type: Type::Integer.into(),
                        })
                        .into(),
                    }),
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
