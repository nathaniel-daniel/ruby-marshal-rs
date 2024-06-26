mod from_value;
mod into_value;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::Expr;
use syn::Field;
use syn::Lit;
use syn::LitByteStr;
use syn::Meta;
use syn::Token;

#[proc_macro_derive(FromValue, attributes(ruby_marshal))]
pub fn derive_from_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_value::derive(input)
}

#[proc_macro_derive(IntoValue, attributes(ruby_marshal))]
pub fn derive_into_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_value::derive(input)
}

pub(crate) fn parse_container_attributes(input: &DeriveInput) -> syn::Result<LitByteStr> {
    let mut object_name = None;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("ruby_marshal") {
            let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

            for meta in nested.iter() {
                match meta {
                    Meta::NameValue(name_value) if name_value.path.is_ident("object") => {
                        if object_name.is_some() {
                            return Err(syn::Error::new(
                                meta.span(),
                                "duplicate object attributes",
                            ));
                        }

                        let value = match &name_value.value {
                            Expr::Lit(value) => match &value.lit {
                                Lit::ByteStr(value) => Some(value),
                                _ => None,
                            },
                            _ => None,
                        };

                        let value = match value {
                            Some(value) => value,
                            None => {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "object name must be a byte string literal",
                                ));
                            }
                        };

                        object_name = Some(value.clone());
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "unrecognized ruby_marshal attribute",
                        ));
                    }
                }
            }
        }
    }

    let object_name = match object_name {
        Some(object_name) => object_name,
        None => {
            return Err(syn::Error::new_spanned(input, "missing object attribute"));
        }
    };

    Ok(object_name)
}

pub(crate) struct FieldAttributes {
    pub name: Option<LitByteStr>,
    pub from_value: Option<syn::Path>,
    pub into_value: Option<syn::Path>,
}

pub(crate) fn parse_field_attributes(field: &Field) -> syn::Result<FieldAttributes> {
    let mut name = None;
    let mut from_value = None;
    let mut into_value = None;
    for attr in field.attrs.iter() {
        if attr.path().is_ident("ruby_marshal") {
            let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

            for meta in nested.iter() {
                match meta {
                    Meta::NameValue(name_value) if name_value.path.is_ident("name") => {
                        if name.is_some() {
                            return Err(syn::Error::new(meta.span(), "duplicate name attributes"));
                        }

                        let value = match &name_value.value {
                            Expr::Lit(value) => match &value.lit {
                                Lit::ByteStr(value) => Some(value),
                                _ => None,
                            },
                            _ => None,
                        };

                        let value = match value {
                            Some(value) => value,
                            None => {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "field name must be a byte string literal",
                                ));
                            }
                        };

                        name = Some(value.clone());
                    }
                    Meta::NameValue(name_value) if name_value.path.is_ident("from_value") => {
                        if from_value.is_some() {
                            return Err(syn::Error::new(
                                meta.span(),
                                "duplicate from_value attributes",
                            ));
                        }

                        let value = match &name_value.value {
                            Expr::Lit(value) => match &value.lit {
                                Lit::Str(value) => Some(value),
                                _ => None,
                            },
                            _ => None,
                        };

                        let value = match value {
                            Some(value) => value,
                            None => {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "from_value attribute must be a string literal",
                                ));
                            }
                        };

                        let value = value.parse::<syn::Path>()?;

                        from_value = Some(value);
                    }
                    Meta::NameValue(name_value) if name_value.path.is_ident("into_value") => {
                        if into_value.is_some() {
                            return Err(syn::Error::new(
                                meta.span(),
                                "duplicate into_value attributes",
                            ));
                        }

                        let value = match &name_value.value {
                            Expr::Lit(value) => match &value.lit {
                                Lit::Str(value) => Some(value),
                                _ => None,
                            },
                            _ => None,
                        };

                        let value = match value {
                            Some(value) => value,
                            None => {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "into_value attribute must be a string literal",
                                ));
                            }
                        };

                        let value = value.parse::<syn::Path>()?;

                        into_value = Some(value.clone());
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "unrecognized ruby_marshal attribute",
                        ));
                    }
                }
            }
        }
    }

    Ok(FieldAttributes {
        name,
        from_value,
        into_value,
    })
}
