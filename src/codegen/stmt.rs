use crate::{
    codegen::IdentBuilder,
    parser::{AssignStmt, BlockStmt, ExprStmt, GotoStmt, IfStmt, LabelStmt, ReturnStmt, Stmt},
    tokenizer::Spanned,
    Result,
};

use super::{
    CCode, CodeGenExpr, CodeGenStmt, Context, ExprValue, LabelValue, SingleExprValue, Type, Value,
};

impl CodeGenStmt for Stmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        match self {
            Self::Label(stmt) => stmt.generate(ctx),
            Self::Expr(stmt) => stmt.generate(ctx),
            Self::Assign(stmt) => stmt.generate(ctx),
            Self::Return(stmt) => stmt.generate(ctx),
            Self::If(stmt) => stmt.generate(ctx),
            Self::Block(stmt) => stmt.generate(ctx),
            Self::Goto(stmt) => stmt.generate(ctx),
        }
    }
}

impl CodeGenStmt for LabelStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let value = ctx.get_or_insert(&self.label, || {
            let ident = IdentBuilder::label(&self.label);
            Value::Label(LabelValue {
                name: c_code! { #ident },
            })
        });
        let Value::Label(LabelValue { name }) = value else {
            return Err(self.label.unrecoverable_error("expected label name"));
        };
        Ok(c_code! {
            #name:
        })
    }
}

impl CodeGenStmt for ExprStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let expr = self.expr.generate_with_type_and_check(&Type::void(), ctx)?;
        Ok(c_code! {
            #expr;
        })
    }
}

impl CodeGenStmt for AssignStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let expr = self.name.generate(ctx)?.flat_map(|name| {
            let expr = self.expr.generate_with_type_and_check(&name.ty, ctx)?;
            Ok(SingleExprValue {
                ty: Type::void(),
                code: c_code! {
                    #name = #expr;
                },
                value: None,
            }
            .into())
        })?;
        let ExprValue::Distinct(expr) = expr else {
            return Err(self.expr.unrecoverable_error("ambiguous assignment"));
        };

        Ok(expr.code)
    }
}

impl CodeGenStmt for ReturnStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let Some(return_type) = ctx.return_type() else {
            return Err(self
                .return_
                .unrecoverable_error("return not allowed in this context"));
        };
        let expr = self.expr.generate_with_type_and_check(&return_type, ctx)?;
        Ok(c_code! {
            return #expr;
        })
    }
}

impl CodeGenStmt for IfStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let boolean = Type::boolean(ctx)?;
        let cond = self.cond.generate_with_type_and_check(&boolean, ctx)?;
        let stmts = self
            .stmts
            .iter()
            .map(|stmt| stmt.generate(ctx))
            .collect::<Result<Vec<_>>>()?;
        let else_ifs = self
            .elsifs
            .iter()
            .map(|e| {
                let cond = e.cond.generate_with_type_and_check(&boolean, ctx)?;
                let stmts = e
                    .stmts
                    .iter()
                    .map(|stmt| stmt.generate(ctx))
                    .collect::<Result<Vec<_>>>()?;
                Ok(c_code! {
                    else if (#cond) {
                        #(#stmts)*
                    }
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let else_ = self
            .else_
            .as_ref()
            .map(|e| {
                let stmts = e
                    .stmts
                    .iter()
                    .map(|stmt| stmt.generate(ctx))
                    .collect::<Result<Vec<_>>>()?;
                Ok(c_code! {
                    else {
                        #(#stmts)*
                    }
                })
            })
            .transpose()?;
        Ok(c_code! {
            if (#cond) {
                #(#stmts)*
            }
            #(#else_ifs)*
            #else_
        })
    }
}

impl CodeGenStmt for BlockStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let mut sub_ctx = ctx.subscope(ctx.return_type());

        let items = self
            .items()
            .map(|item| item.generate(&mut sub_ctx))
            .collect::<Result<Vec<_>>>()?;

        let stmts = self
            .stmts
            .iter()
            .map(|stmt| stmt.generate(&mut sub_ctx))
            .collect::<Result<Vec<_>>>()?;

        Ok(c_code! {
            {
                #(#items)*
                #(#stmts)*
            }
        })
    }
}

impl CodeGenStmt for GotoStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let value = ctx.get_or_insert(&self.label, || {
            let ident = IdentBuilder::label(&self.label);
            Value::Label(LabelValue {
                name: c_code! { #ident },
            })
        });
        let Value::Label(LabelValue { name }) = value else {
            return Err(self.label.unrecoverable_error("expected label name"));
        };
        Ok(c_code! {
            goto #name;
        })
    }
}
