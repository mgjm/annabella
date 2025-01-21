macro_rules! enum_dispatch {
    ({
        $(#[$meta:meta])*
        $vis:vis enum $ident:ident {
            $($variant:ident($ty:ty),)*
        }
    }) => {
        $(#[$meta])*
        $vis enum $ident {
            $($variant($ty),)*
        }

        macro_rules! $ident {
            ($self:expr, |$value:ident| $expr:expr) => {
                match $self {
                    $($ident::$variant($value) => $expr,)*
                }
            };
        }
    };
}
