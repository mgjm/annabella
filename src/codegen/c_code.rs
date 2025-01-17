use std::fmt;

macro_rules! c_code {
    () => {
        CCode::new()
    };
    ($($tt:tt)*) => {
        CCode::from_token_stream(::quote::quote! { $($tt)* })
    };
}

#[derive(Default, Clone)]
pub struct CCode(proc_macro2::TokenStream);

impl CCode {
    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn from_token_stream(tokens: proc_macro2::TokenStream) -> Self {
        Self(Self::cleanup(tokens))
    }

    fn cleanup(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        tokens
            .into_iter()
            .map(|mut tree| {
                match &mut tree {
                    proc_macro2::TokenTree::Group(group) => {
                        *group = proc_macro2::Group::new(
                            group.delimiter(),
                            Self::cleanup(group.stream()),
                        )
                    }
                    proc_macro2::TokenTree::Literal(literal) => {
                        let lit = literal.to_string();
                        if let Some((lit, "8" | "16" | "32" | "64" | "128" | "size")) =
                            lit.split_once(['i', 'u'])
                        {
                            *literal = lit.parse().unwrap();
                        }
                    }
                    _ => {}
                }
                tree
            })
            .collect()
    }
}

impl quote::ToTokens for CCode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl fmt::Display for CCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for CCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
