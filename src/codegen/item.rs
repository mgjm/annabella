use crate::{
    parser::{Function, Item, Result, Variable},
    tokenizer::Spanned,
};

use super::{CCode, CodeGenStmt, Context, FunctionType, FunctionValue, Type, Value, VariableValue};

impl CodeGenStmt for Item {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        match self {
            Self::Function(item) => item.generate(ctx),
            Self::Variable(item) => item.generate(ctx),
        }
    }
}

impl Function {
    fn c_name(&self) -> CCode {
        use std::fmt::Write;

        let mut ident = format!("annabella_{}_", self.name);
        if let Some(args) = &self.args {
            for arg in args.iter() {
                write!(ident, "__{}", arg.ty).unwrap();
            }
        }
        if let Some((_, ty)) = &self.return_type {
            write!(ident, "___{ty}").unwrap();
        }
        let ident = quote::format_ident!("{}", ident);
        c_code! {
            #ident
        }
    }
}

impl CodeGenStmt for Function {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let name = self.c_name();
        let args = self.args().map(|arg| {
            let name = &arg.name;
            let ty = &arg.ty;
            c_code! { #ty #name }
        });

        let mut sub_ctx = ctx.subscope(self.return_type().map(Type::parse_ident).transpose()?);
        for arg in self.args() {
            let name = &arg.name;
            sub_ctx.insert(
                &arg.name,
                Value::Variable(VariableValue {
                    name: c_code! { #name },
                    ty: Type::parse_ident(&arg.ty)?,
                }),
            )?;
        }
        let items = self
            .items
            .iter()
            .map(|item| item.generate(&mut sub_ctx))
            .collect::<Result<Vec<_>>>()?;

        let stmts = self
            .stmts
            .iter()
            .map(|stmt| stmt.generate(&mut sub_ctx))
            .collect::<Result<Vec<_>>>()?;

        let return_type = if let Some(ty) = self.return_type() {
            c_code! { #ty }
        } else {
            c_code! { void }
        };

        ctx.push_function(c_code! {
            #return_type #name(#(#args),*) {
                #(#items)*
                #(#stmts)*
            }
        });

        let args = self
            .args()
            .map(|arg| Type::parse_ident(&arg.ty))
            .collect::<Result<Vec<_>>>()?;

        let return_type = if let Some(ty) = self.return_type() {
            Type::parse_ident(ty)?
        } else {
            Type::Void.into()
        };

        ctx.insert(
            &self.name,
            Value::Function(FunctionValue::new(
                name,
                Type::Function(FunctionType { args, return_type }).into(),
            )),
        )?;
        Ok(c_code!())
    }
}

impl CodeGenStmt for Variable {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let name = &self.name;
        ctx.insert(
            name,
            Value::Variable(VariableValue {
                name: c_code! { #name },
                ty: Type::parse_ident(&self.ty)?,
            }),
        )?;
        let ty = &self.ty;
        Ok(c_code! {
            #ty #name;
        })
    }
}
