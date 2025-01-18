use crate::{
    parser::{AssignStmt, ExprStmt, ReturnStmt, Stmt},
    tokenizer::Spanned,
    Result,
};

use super::{CCode, CodeGenExpr, CodeGenStmt, Context, ExprValue, SingleExprValue, Type};

impl CodeGenStmt for Stmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        match self {
            Stmt::Expr(stmt) => stmt.generate(ctx),
            Stmt::Return(stmt) => stmt.generate(ctx),
            Stmt::Assign(stmt) => stmt.generate(ctx),
        }
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
