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
        let name_attribute = field_attributes;

        let name = field
            .ident
            .as_ref()
            .expect("named field structs should have named fields");

        let name_str = match name_attribute {
            Some(name) => name,
            None => LitByteStr::new(name.to_string().as_bytes(), name.span()),
        };
        fields.push(IntoValueField { name, name_str });
    }

    let create_field_keys = fields.iter().enumerate().map(|(i, field)| {
        let ident = format_ident!("field_{i}_key");
        let name = &field.name_str;
        quote! {
            let #ident = arena.create_symbol(#name.into());;
        }
    });

    let create_field_values = fields.iter().enumerate().map(|(i, field)| {
        let ident = format_ident!("field_{i}_value");
        let field_name = &field.name;
        quote! {
            let #ident = self.#field_name.into_value(arena)?;
        }
    });

    let field_vec_entries = fields.iter().enumerate().map(|(i, _field)| {
        let key_ident = format_ident!("field_{i}_key");
        let value_ident = format_ident!("field_{i}_value");

        quote! {
            (#key_ident, #value_ident),
        }
    });

    let input_name = &input.ident;
    let tokens = quote! {
        impl ::ruby_marshal::IntoValue for #input_name {
            fn into_value(self, arena: &mut ::ruby_marshal::ValueArena) -> Result<::ruby_marshal::ValueHandle, ::ruby_marshal::IntoValueError> {
                let object_name = arena.create_symbol(#object_name.into());

                #(#create_field_keys)*

                #(#create_field_values)*

                let fields = vec![
                    #(#field_vec_entries)*
                ];

                let object = arena.create_object(object_name, fields);

                Ok(object.into())
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}

struct IntoValueField<'a> {
    name: &'a Ident,
    name_str: LitByteStr,
}
