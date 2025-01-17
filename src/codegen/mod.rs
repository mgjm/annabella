use quote::ToTokens;

use crate::{
    parser::{Result, Stmt},
    tokenizer::Ident,
};

#[macro_use]
mod c_code;

mod context;
mod expr;
mod standard;
mod stmt;
mod ty;
mod value;

pub use self::{c_code::CCode, context::Context, ty::*, value::*};

pub fn run(stmts: Vec<Stmt>) -> Result<String> {
    let mut ctx = Context::base();
    let mut ctx = ctx.context();
    let ctx = &mut ctx;
    ctx.push_include("<stdio.h>");
    standard::generate(ctx)?;
    for stmt in &stmts {
        let code = stmt.generate(ctx)?;
        ctx.push_main(code);
    }
    ctx.push_main(c_code! { annabella_Main_(); });

    // dbg!(&**ctx);
    // todo!();
    Ok(ctx.to_string())
}

trait CodeGenStmt {
    fn generate(&self, ctx: &mut Context) -> Result<CCode>;
}

struct ExprValue {
    ty: RcType,
    code: CCode,
}

impl ToTokens for ExprValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.code.to_tokens(tokens)
    }
}

trait CodeGenExpr {
    fn generate(&self, ctx: &mut Context) -> Result<ExprValue>;
}
impl ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        proc_macro2::Ident::new(&self.name, proc_macro2::Span::call_site()).to_tokens(tokens)
    }
}
