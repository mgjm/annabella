use crate::{
    parser::{ExprStmt, Function, Result, ReturnStmt, Stmt},
    tokenizer::Spanned,
};

use super::{
    CCode, CodeGenExpr, CodeGenStmt, Context, FunctionType, FunctionValue, RcType, Type, Value,
    VariableValue,
};

impl CodeGenStmt for Stmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        match self {
            Stmt::Expr(stmt) => stmt.generate(ctx),
            Stmt::Return(stmt) => stmt.generate(ctx),
            Stmt::Function(stmt) => stmt.generate(ctx),
        }
    }
}

impl CodeGenStmt for ExprStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let expr = self.expr.generate(ctx)?;
        let (fmt, use_value) = expr.ty.fmt();
        Ok(if use_value {
            c_code! {
                printf(#fmt "\n", #expr);
            }
        } else {
            c_code! {
                #expr;
                printf(#fmt "\n");
            }
        })
    }
}

impl CodeGenStmt for ReturnStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let expr = self.expr.generate(ctx)?;
        Ok(c_code! {
            return #expr;
        })
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

        let mut sub_ctx = ctx.subscope();
        for arg in self.args() {
            let name = &arg.name;
            sub_ctx.insert(
                &arg.name,
                Value::Variable(VariableValue {
                    name: c_code! { #name },
                    ty: Type::parse(&arg.ty.name)
                        .ok_or_else(|| arg.ty.unrecoverable_error("unknown type"))?,
                }),
            )?;
        }

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
                #(#stmts)*
            }
        });

        let args = self
            .args()
            .map(|arg| {
                Type::parse(&arg.ty.name)
                    .ok_or_else(|| arg.ty.unrecoverable_error("unsupported type"))
            })
            .collect::<Result<Vec<_>>>()?;

        let return_type = if let Some(ty) = self.return_type() {
            Type::parse(&ty.name).ok_or_else(|| ty.unrecoverable_error("unspupported type"))?
        } else {
            Type::Void.into()
        };

        ctx.insert(
            &self.name,
            Value::Function(FunctionValue {
                name,
                ty: Type::Function(FunctionType { args, return_type }).into(),
            }),
        )?;
        Ok(c_code!())
    }
}
