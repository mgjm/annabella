use crate::{
    parser::{Function, Item, Variable},
    Result,
};

use super::{
    CCode, CodeGenStmt, Context, FunctionType, FunctionValue, IdentBuilder, Type, Value,
    VariableValue,
};

impl CodeGenStmt for Item {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        match self {
            Self::Function(item) => item.generate(ctx),
            Self::Type(item) => item.generate(ctx),
            Self::Subtype(item) => item.generate(ctx),
            Self::Variable(item) => item.generate(ctx),
        }
    }
}

impl Function {
    fn c_name(&self) -> CCode {
        let ident = IdentBuilder::function(
            &self.name,
            self.args().map(|arg| &arg.ty),
            self.return_type(),
        );
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

        let mut sub_ctx = ctx.subscope(
            self.return_type()
                .map(|arg| Type::parse_ident(arg, ctx))
                .transpose()?,
        );
        for arg in self.args() {
            let name = &arg.name;
            let ty = Type::parse_ident(&arg.ty, &sub_ctx)?;
            sub_ctx.insert(
                &arg.name,
                Value::Variable(VariableValue {
                    name: c_code! { #name },
                    ty,
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
            .map(|arg| Type::parse_ident(&arg.ty, ctx))
            .collect::<Result<Vec<_>>>()?;

        let return_type = if let Some(ty) = self.return_type() {
            Type::parse_ident(ty, ctx)?
        } else {
            Type::void()
        };

        ctx.insert(
            &self.name,
            Value::Function(FunctionValue::new(
                name,
                Type::function(FunctionType { args, return_type }),
            )),
        )?;
        Ok(c_code!())
    }
}

impl CodeGenStmt for Variable {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let name = &self.name;
        let ty = Type::parse_ident(&self.ty, ctx)?;
        ctx.insert(
            name,
            Value::Variable(VariableValue {
                name: c_code! { #name },
                ty: ty.clone(),
            }),
        )?;
        Ok(c_code! {
            #ty #name;
        })
    }
}
