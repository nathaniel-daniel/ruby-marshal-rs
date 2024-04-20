use crate::parse_container_attributes;
use crate::parse_field_attributes;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::Ident;
use syn::LitByteStr;
use syn::Type;

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
            match parse_field_attributes(field).map_err(syn::Error::into_compile_error) {
                Ok(value) => value,
                Err(error) => {
                    return error.into();
                }
            };

        let name = field
            .ident
            .as_ref()
            .expect("named field structs should have named fields");

        let name_str = match field_attributes.name {
            Some(name) => name,
            None => LitByteStr::new(format!("@{name}").as_bytes(), name.span()),
        };
        fields.push(FromValueField {
            name,
            name_str,
            ty: &field.ty,
            from_value: field_attributes.from_value,
        });
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
        let ty = &field.ty;
        let ty_span = ty.span();

        let get_value = match field.from_value.as_ref() {
            Some(from_value) => {
                quote_spanned! {from_value.span()=>
                    let value = {
                        struct Wrapper(#ty);

                        impl<'a> ::ruby_marshal::FromValue<'a> for Wrapper {
                            fn from_value(
                                ctx: &::ruby_marshal::FromValueContext,
                                value: &'a::ruby_marshal::Value
                            ) -> Result<Self, ::ruby_marshal::FromValueError> {
                                let value = #from_value(ctx, value)?;

                                Ok(Self(value))
                            }
                        }

                        let value: Wrapper = ctx.from_value(value)?;
                        value.0
                    };
                }
            }
            None => {
                quote_spanned! {ty_span=>
                    let value = ctx.from_value(value)?;
                }
            }
        };

        quote! {
            #field_name => {
                if #ident.is_some() {
                    return Err(
                        ::ruby_marshal::FromValueError::DuplicateInstanceVariable {
                            name: key.into()
                        }
                    );
                }

                #get_value
                #ident = Some(value);
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
            fn from_value(
                ctx: &::ruby_marshal::FromValueContext,
                value: &'a::ruby_marshal::Value
            ) -> Result<Self, ::ruby_marshal::FromValueError> {
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

struct FromValueField<'a> {
    name: &'a Ident,
    name_str: LitByteStr,
    ty: &'a Type,
    from_value: Option<syn::Path>,
}
