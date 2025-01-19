macro_rules! parse {
    ({
        enum $ident:ident {
            $($var:ident($ty:ty),)*
        }
    }) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $ident {
            $($var($ty),)*
        }

        impl Spanned for $ident{
            fn span(&self) -> Span {
                match self {
                    $(Self::$var(inner) => inner.span(),)*
                }
            }
        }
    };
    ({
        struct $ident:ident {
            $($name:ident: $ty:ty,)*
        }
    }) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $ident {
            $(pub $name: $ty,)*
        }


        impl Spanned for $ident {
            fn span(&self) -> Span {
                let mut span = Span::call_site();
                $(
                    span.extend(self.$name.span());
                )*
                span
            }
        }
    };
}
