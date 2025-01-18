use std::fmt::{self, Write};

use proc_macro2::{Ident as CIdent, Span};

use crate::{parser::token::Token, tokenizer::Ident};

use super::Type;

pub struct IdentBuilder {
    str: String,
}

impl IdentBuilder {
    fn base() -> Self {
        Self {
            str: "annabella_".into(),
        }
    }

    fn push_str(&mut self, s: &str) -> &mut Self {
        self.str.push_str(s);
        self
    }

    fn ty(&mut self, ty: &Type) -> &mut Self {
        self.push_str("__").push_str(ty.to_str())
    }

    fn ident(&mut self, ident: &Ident) -> &mut Self {
        self.push_str("__").push_str(&ident.name)
    }

    fn debug(&mut self, fmt: impl fmt::Debug) -> &mut Self {
        self.push_str("__");
        write!(self.str, "{fmt:?}").unwrap();
        self
    }

    fn args<'a>(&mut self, args: impl Iterator<Item = &'a Ident>) -> &mut Self {
        for arg in args {
            self.ident(arg);
        }
        self
    }

    fn build(&self) -> CIdent {
        CIdent::new(&self.str, Span::call_site())
    }

    // fn build(&self) -> CIdent {
    //     use std::sync::atomic::AtomicUsize;
    //     static COUNTER: AtomicUsize = AtomicUsize::new(0);
    //     let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    //     CIdent::new(&format!("annabella_{n}"), Span::call_site())
    // }

    fn start(kind: &str) -> Self {
        let mut this = Self::base();
        this.push_str(kind);
        this
    }

    pub fn constraint_check(ty: &Ident) -> CIdent {
        Self::start("constraint").ident(ty).build()
    }

    pub fn enum_value(name: &Ident, value: &Ident) -> CIdent {
        Self::start("enum").ident(name).ident(value).build()
    }

    pub fn op_function(op: impl Token, ty: &Type) -> CIdent {
        let _ = op;
        Self::start("op").debug(op).ty(ty).build()
    }

    pub fn print(ty: &Type) -> CIdent {
        Self::start("print").ty(ty).build()
    }

    pub fn function<'a>(
        name: &Ident,
        args: impl Iterator<Item = &'a Ident>,
        return_type: Option<&Ident>,
    ) -> CIdent {
        Self::start("function")
            .ident(name)
            .args(args)
            .push_str("_")
            .args(return_type.into_iter())
            .build()
    }

    pub fn type_(name: &Ident) -> CIdent {
        CIdent::new(&name.name, Span::call_site())
        // Self::start("type").ident(name).build()
    }
}
