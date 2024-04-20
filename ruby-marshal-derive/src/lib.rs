use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::Expr;
use syn::Field;
use syn::Ident;
use syn::Lit;
use syn::LitByteStr;
use syn::Meta;
use syn::Token;

#[proc_macro_derive(FromValue, attributes(ruby_marshal))]
pub fn derive_from_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return quote_spanned! {
                input.span() =>
                compile_error!("only structs are supported");
            }
            .into();
        }
    };
    let input_fields = match &input_data.fields {
        syn::Fields::Named(fields) => fields,
        _ => {
            return quote_spanned! {
                input.span() =>
                compile_error!("only named field structs are supported");
            }
            .into();
        }
    };

    let container_attributes =
        match parse_container_attributes(&input).map_err(syn::Error::into_compile_error) {
            Ok(value) => value,
            Err(error) => {
                return error.into();
            }
        };

    let object_name = container_attributes;

    let mut fields = Vec::with_capacity(input_fields.named.len());
    for field in input_fields.named.iter() {
        let field_attributes =
            match parse_field_attributes(&field).map_err(syn::Error::into_compile_error) {
                Ok(value) => value,
                Err(error) => {
                    return error.into();
                }
            };
        let name_attribute = field_attributes;

        let name = field
            .ident
            .as_ref()
            .expect("named field structs should have named fields");

        let name_str = match name_attribute {
            Some(name) => name,
            None => LitByteStr::new(name.to_string().as_bytes(), name.span()),
        };
        fields.push(FromValueField { name, name_str });
    }

    let option_fields = fields.iter().enumerate().map(|(i, _field)| {
        let ident = format_ident!("option_field_{i}");
        quote! {
            let mut #ident = None;
        }
    });
    let match_arms = fields.iter().enumerate().map(|(i, field)| {
        let ident = format_ident!("option_field_{i}");
        let field_name = &field.name_str;
        quote! {
            #field_name => {
                if #ident.is_some() {
                    return Err(::ruby_marshal::FromValueError::DuplicateInstanceVariable { name: key.into() });
                }
                #ident = Some(ctx.from_value(value)?);
            }
        }
    });
    let unpack_option_fields = fields.iter().enumerate().map(|(i, field)| {
        let option_field_ident = format_ident!("option_field_{i}");
        let field_ident = format_ident!("field_{i}");
        let field_name = &field.name_str;
        quote! {
            let #field_ident = #option_field_ident.ok_or_else(|| ::ruby_marshal::FromValueError::MissingInstanceVariable {
                name: #field_name.into(),
            })?;
        }
    });
    let init_struct_fields = fields.iter().enumerate().map(|(i, field)| {
        let struct_field_ident = &field.name;
        let field_ident = format_ident!("field_{i}");
        quote! {
            #struct_field_ident: #field_ident,
        }
    });

    let input_name = &input.ident;
    let tokens = quote! {
        impl<'a> ::ruby_marshal::FromValue<'a> for #input_name {
            fn from_value(ctx: &::ruby_marshal::FromValueContext, value: &'a::ruby_marshal::Value) -> Result<Self, ::ruby_marshal::FromValueError> {
                let value: &::ruby_marshal::ObjectValue = ::ruby_marshal::FromValue::from_value(ctx, value)?;
                {
                    let name = value.name();
                    let name: &::ruby_marshal::SymbolValue = ctx.from_value(name.into())?;
                    let name = name.value();

                    if name != #object_name {
                        return Err(::ruby_marshal::FromValueError::UnexpectedObjectName { name: name.into() });
                    }
                }

                #(#option_fields)*

                for (key, value) in value.instance_variables().iter().copied() {
                    let key: &::ruby_marshal::SymbolValue = ctx.from_value(key.into())?;
                    let key = key.value();

                    match key {
                        #(#match_arms)*
                        _ => {
                            return Err(::ruby_marshal::FromValueError::UnknownInstanceVariable { name: key.into() });
                        }
                    }
                }

                #(#unpack_option_fields)*

                Ok(Self {
                    #(#init_struct_fields)*
                })
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}

fn parse_container_attributes(input: &DeriveInput) -> syn::Result<LitByteStr> {
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

fn parse_field_attributes(field: &Field) -> syn::Result<Option<LitByteStr>> {
    let mut name = None;
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

    Ok(name)
}

struct FromValueField<'a> {
    name: &'a Ident,
    name_str: LitByteStr,
}
