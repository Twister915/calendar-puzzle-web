#[macro_export(local_inner_macros)]
macro_rules! return_matching {
    ($expression: expr, $pattern: pat $(if $guard:expr)? $(,)?) => {
        match $expression {
            v @ $pattern $(if $guard)? => {
                return v;
            },
            _ => {}
        }
    }
}
