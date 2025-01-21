use crate::{
    parser::{Function, Item, Variable},
    Result,
};

use super::{
    CCode, CodeGenStmt, Context, FunctionType, FunctionValue, IdentBuilder, Permission, Type,
    Value, VariableValue,
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
        let mut sub_ctx = ctx.subscope(
            self.return_type()
                .map(|ty| Type::from_ident(ty, ctx))
                .transpose()?,
        );

        let args = self
            .args()
            .map(|arg| {
                let ty = Type::from_ident(&arg.ty, &sub_ctx)?;
                let ident = IdentBuilder::variable(&arg.name);
                let code = c_code! { #ty #ident };
                sub_ctx.insert(
                    &arg.name,
                    Value::Variable(VariableValue {
                        name: c_code! { #ident },
                        ty,
                        perm: Permission::Read,
                    }),
                )?;
                Ok(code)
            })
            .collect::<Result<Vec<_>>>()?;

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
            let ty = Type::from_ident(ty, ctx)?;
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
            .map(|arg| Type::from_ident(&arg.ty, ctx))
            .collect::<Result<Vec<_>>>()?;

        let return_type = if let Some(ty) = self.return_type() {
            Type::from_ident(ty, ctx)?
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
        let ty = Type::from_ident(&self.ty, ctx)?;
        let ident = IdentBuilder::variable(&self.name);
        let code = c_code! {
            #ty #ident;
        };
        ctx.insert(
            &self.name,
            Value::Variable(VariableValue {
                name: c_code! { #ident },
                ty,
                perm: Permission::ReadWrite,
            }),
        )?;
        Ok(code)
    }
}
