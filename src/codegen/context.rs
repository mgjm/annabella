use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use super::{CCode, Scope, Type};

pub struct Base {
    inner: Inner,
}

impl Base {
    pub fn context(&mut self) -> Context {
        Context {
            inner: &mut self.inner,
            scope: Default::default(),
            return_type: None,
        }
    }
}

pub struct Context<'a> {
    inner: &'a mut Inner,
    scope: Scope<'a>,
    return_type: Option<Type>,
}

#[derive(Default)]
struct Inner {
    includes: Vec<&'static str>,
    // types: Vec<CCode>,
    functions: Vec<CCode>,
    main: Vec<CCode>,
}

impl Context<'_> {
    pub fn base() -> Base {
        Base {
            inner: Inner::default(),
        }
    }

    pub fn push_include(&mut self, include: &'static str) {
        self.inner.includes.push(include);
    }

    pub fn push_type(&mut self, code: CCode) {
        // self.inner.types.push(code);
        self.inner.functions.push(code);
    }

    pub fn push_function(&mut self, code: CCode) {
        self.inner.functions.push(code);
    }

    pub fn push_main(&mut self, code: CCode) {
        self.inner.main.push(code);
    }

    pub fn subscope(&mut self, return_type: Option<Type>) -> Context {
        Context {
            inner: self.inner,
            scope: self.scope.subscope(),
            return_type,
        }
    }

    pub fn return_type(&self) -> Option<Type> {
        self.return_type.clone()
    }
}

impl fmt::Display for Context<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for include in &self.inner.includes {
            writeln!(f, "#include {include}")?;
        }
        writeln!(f)?;

        // for type_ in &self.inner.types {
        //     type_.fmt(f)?;
        // }
        // writeln!(f)?;

        for function in &self.inner.functions {
            function.fmt(f)?;
            writeln!(f)?;
        }

        let main = &self.inner.main;
        c_code! {
            int main() {
                #(#main)*
                return 0;
            }
        }
        .fmt(f)
    }
}

impl<'a> Deref for Context<'a> {
    type Target = Scope<'a>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for Context<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}
