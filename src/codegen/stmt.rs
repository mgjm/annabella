use crate::{
    parser::{AssignStmt, ExprStmt, Result, ReturnStmt, Stmt},
    tokenizer::Spanned,
};

use super::{CCode, CodeGenExpr, CodeGenStmt, Context, ExprValue};

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
        let ExprValue::Distinct(expr) = self.expr.generate(ctx)? else {
            return Err(self.expr.unrecoverable_error("ambiguous expression"));
        };

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
        let ExprValue::Distinct(expr) = self.expr.generate(ctx)? else {
            return Err(self.expr.unrecoverable_error("ambiguous expression"));
        };

        Ok(c_code! {
            return #expr;
        })
    }
}

impl CodeGenStmt for AssignStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode> {
        let ExprValue::Distinct(name) = self.name.generate(ctx)? else {
            return Err(self.name.unrecoverable_error("ambiguous expression"));
        };
        let ExprValue::Distinct(expr) = self.expr.generate(ctx)? else {
            return Err(self.expr.unrecoverable_error("ambiguous expression"));
        };
        Ok(c_code! {
            #name = #expr;
        })
    }
}
