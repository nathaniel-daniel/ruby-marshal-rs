mod from_value;

#[proc_macro_derive(FromValue, attributes(ruby_marshal))]
pub fn derive_from_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_value::derive(input)
}
