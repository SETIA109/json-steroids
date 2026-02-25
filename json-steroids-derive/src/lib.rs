//! Procedural macros for json-steroids
//!
//! Generates efficient serializers and deserializers for data structures.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

/// Get the crate path - correctly resolves whether we are inside
/// the `json_steroids` crate itself or an external consumer.
fn crate_path() -> TokenStream2 {
    match crate_name("json-steroids") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
            quote! { ::#ident }
        }
        // Fallback: we are inside the crate being compiled (unit tests, benchmarks)
        Err(_) => quote! { crate },
    }
}

/// Derive macro for JSON serialization
///
/// # Example
/// ```ignore
/// #[derive(JsonSerialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// ```
#[proc_macro_derive(JsonSerialize, attributes(json))]
pub fn derive_json_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let krate = crate_path();

    let serialize_body = generate_serialize_body(&input.data, name, &krate);

    let expanded = quote! {
        impl #impl_generics #krate::JsonSerialize for #name #ty_generics #where_clause {
            fn json_serialize(&self, writer: &mut #krate::JsonWriter) {
                #serialize_body
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for JSON deserialization
///
/// # Example
/// ```ignore
/// #[derive(JsonDeserialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// ```
#[proc_macro_derive(JsonDeserialize, attributes(json))]
pub fn derive_json_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let krate = crate_path();

    let deserialize_body = generate_deserialize_body(&input.data, name, &krate);

    let expanded = quote! {
        impl #impl_generics #krate::JsonDeserialize for #name #ty_generics #where_clause {
            fn json_deserialize(parser: &mut #krate::JsonParser<'_>) -> #krate::Result<Self> {
                #deserialize_body
            }
        }
    };

    TokenStream::from(expanded)
}

/// Combined derive for both serialization and deserialization
#[proc_macro_derive(Json, attributes(json))]
pub fn derive_json(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let krate = crate_path();

    let serialize_body = generate_serialize_body(&input.data, name, &krate);
    let deserialize_body = generate_deserialize_body(&input.data, name, &krate);

    let expanded = quote! {
        impl #impl_generics #krate::JsonSerialize for #name #ty_generics #where_clause {
            fn json_serialize(&self, writer: &mut #krate::JsonWriter) {
                #serialize_body
            }
        }

        impl #impl_generics #krate::JsonDeserialize for #name #ty_generics #where_clause {
            fn json_deserialize(parser: &mut #krate::JsonParser<'_>) -> #krate::Result<Self> {
                #deserialize_body
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_serialize_body(data: &Data, _name: &Ident, krate: &TokenStream2) -> TokenStream2 {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_serializations: Vec<TokenStream2> = fields.named.iter().enumerate().map(|(i, f)| {
                        let field_name = f.ident.as_ref().unwrap();
                        let field_name_str = get_field_name(&f.attrs, field_name);
                        let is_first = i == 0;

                        if is_first {
                            quote! {
                                writer.write_key(#field_name_str);
                                #krate::JsonSerialize::json_serialize(&self.#field_name, writer);
                            }
                        } else {
                            quote! {
                                writer.write_comma();
                                writer.write_key(#field_name_str);
                                #krate::JsonSerialize::json_serialize(&self.#field_name, writer);
                            }
                        }
                    }).collect();

                    quote! {
                        writer.begin_object();
                        #(#field_serializations)*
                        writer.end_object();
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_serializations: Vec<TokenStream2> = (0..fields.unnamed.len())
                        .enumerate()
                        .map(|(i, idx)| {
                            let index = syn::Index::from(idx);
                            if i == 0 {
                                quote! {
                                    #krate::JsonSerialize::json_serialize(&self.#index, writer);
                                }
                            } else {
                                quote! {
                                    writer.write_comma();
                                    #krate::JsonSerialize::json_serialize(&self.#index, writer);
                                }
                            }
                        })
                        .collect();

                    quote! {
                        writer.begin_array();
                        #(#field_serializations)*
                        writer.end_array();
                    }
                }
                Fields::Unit => {
                    quote! { writer.write_null(); }
                }
            }
        }
        Data::Enum(data_enum) => {
            let variants: Vec<TokenStream2> = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let variant_name_str = variant_name.to_string();

                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            Self::#variant_name => {
                                writer.write_string(#variant_name_str);
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_names: Vec<Ident> = (0..fields.unnamed.len())
                            .map(|i| format_ident!("f{}", i))
                            .collect();
                        let field_serializations: Vec<TokenStream2> = field_names.iter().enumerate().map(|(i, name)| {
                            if i == 0 {
                                quote! { #krate::JsonSerialize::json_serialize(#name, writer); }
                            } else {
                                quote! {
                                    writer.write_comma();
                                    #krate::JsonSerialize::json_serialize(#name, writer);
                                }
                            }
                        }).collect();

                        quote! {
                            Self::#variant_name(#(#field_names),*) => {
                                writer.begin_object();
                                writer.write_key(#variant_name_str);
                                writer.begin_array();
                                #(#field_serializations)*
                                writer.end_array();
                                writer.end_object();
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_names: Vec<&Ident> = fields.named.iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();
                        let field_serializations: Vec<TokenStream2> = field_names.iter().enumerate().map(|(i, name)| {
                            let name_str = name.to_string();
                            if i == 0 {
                                quote! {
                                    writer.write_key(#name_str);
                                    #krate::JsonSerialize::json_serialize(#name, writer);
                                }
                            } else {
                                quote! {
                                    writer.write_comma();
                                    writer.write_key(#name_str);
                                    #krate::JsonSerialize::json_serialize(#name, writer);
                                }
                            }
                        }).collect();

                        quote! {
                            Self::#variant_name { #(#field_names),* } => {
                                writer.begin_object();
                                writer.write_key(#variant_name_str);
                                writer.begin_object();
                                #(#field_serializations)*
                                writer.end_object();
                                writer.end_object();
                            }
                        }
                    }
                }
            }).collect();

            quote! {
                match self {
                    #(#variants)*
                }
            }
        }
        Data::Union(_) => {
            quote! { compile_error!("Unions are not supported"); }
        }
    }
}

fn generate_deserialize_body(data: &Data, name: &Ident, krate: &TokenStream2) -> TokenStream2 {
    match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_declarations: Vec<TokenStream2> = fields
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().unwrap();
                        quote! { let mut #field_name = None; }
                    })
                    .collect();

                let field_matches: Vec<TokenStream2> = fields.named.iter().map(|f| {
                        let field_name = f.ident.as_ref().unwrap();
                        let field_name_str = get_field_name(&f.attrs, field_name);
                        quote! {
                            #field_name_str => {
                                #field_name = Some(#krate::JsonDeserialize::json_deserialize(parser)?);
                            }
                        }
                    }).collect();

                let field_unwraps: Vec<TokenStream2> = fields.named.iter().map(|f| {
                        let field_name = f.ident.as_ref().unwrap();
                        let field_name_str = field_name.to_string();
                        let is_option = is_option_type(&f.ty);

                        if is_option {
                            quote! {
                                #field_name: #field_name.unwrap_or(None)
                            }
                        } else {
                            quote! {
                                #field_name: #field_name.ok_or_else(|| #krate::JsonError::MissingField(#field_name_str.to_string()))?
                            }
                        }
                    }).collect();

                quote! {
                    parser.expect_object_start()?;
                    #(#field_declarations)*

                    loop {
                        match parser.next_object_key()? {
                            Some(key) => {
                                match key.as_ref() {
                                    #(#field_matches)*
                                    _ => {
                                        parser.skip_value()?;
                                    }
                                }
                            }
                            None => break,
                        }
                    }

                    parser.expect_object_end()?;

                    Ok(#name {
                        #(#field_unwraps),*
                    })
                }
            }
            Fields::Unnamed(fields) => {
                let field_deserializations: Vec<TokenStream2> = (0..fields.unnamed.len())
                    .enumerate()
                    .map(|(i, _)| {
                        if i == 0 {
                            quote! { #krate::JsonDeserialize::json_deserialize(parser)? }
                        } else {
                            quote! {
                                {
                                    parser.expect_comma()?;
                                    #krate::JsonDeserialize::json_deserialize(parser)?
                                }
                            }
                        }
                    })
                    .collect();

                quote! {
                    parser.expect_array_start()?;
                    let result = #name(#(#field_deserializations),*);
                    parser.expect_array_end()?;
                    Ok(result)
                }
            }
            Fields::Unit => {
                quote! {
                    parser.expect_null()?;
                    Ok(#name)
                }
            }
        },
        Data::Enum(data_enum) => {
            let variant_matches: Vec<TokenStream2> = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let variant_name_str = variant_name.to_string();

                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            #variant_name_str => Ok(#name::#variant_name)
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_deserializations: Vec<TokenStream2> = (0..fields.unnamed.len()).enumerate().map(|(i, _)| {
                            if i == 0 {
                                quote! { #krate::JsonDeserialize::json_deserialize(parser)? }
                            } else {
                                quote! {
                                    {
                                        parser.expect_comma()?;
                                        #krate::JsonDeserialize::json_deserialize(parser)?
                                    }
                                }
                            }
                        }).collect();

                        quote! {
                            #variant_name_str => {
                                parser.expect_array_start()?;
                                let result = #name::#variant_name(#(#field_deserializations),*);
                                parser.expect_array_end()?;
                                Ok(result)
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_declarations: Vec<TokenStream2> = fields.named.iter().map(|f| {
                            let field_name = f.ident.as_ref().unwrap();
                            quote! { let mut #field_name = None; }
                        }).collect();

                        let field_matches: Vec<TokenStream2> = fields.named.iter().map(|f| {
                            let field_name = f.ident.as_ref().unwrap();
                            let field_name_str = field_name.to_string();
                            quote! {
                                #field_name_str => {
                                    #field_name = Some(#krate::JsonDeserialize::json_deserialize(parser)?);
                                }
                            }
                        }).collect();

                        let field_unwraps: Vec<TokenStream2> = fields.named.iter().map(|f| {
                            let field_name = f.ident.as_ref().unwrap();
                            let field_name_str = field_name.to_string();
                            quote! {
                                #field_name: #field_name.ok_or_else(|| #krate::JsonError::MissingField(#field_name_str.to_string()))?
                            }
                        }).collect();

                        quote! {
                            #variant_name_str => {
                                parser.expect_object_start()?;
                                #(#field_declarations)*

                                loop {
                                    match parser.next_object_key()? {
                                        Some(key) => {
                                            match key.as_ref() {
                                                #(#field_matches)*
                                                _ => { parser.skip_value()?; }
                                            }
                                        }
                                        None => break,
                                    }
                                }

                                Ok(#name::#variant_name {
                                    #(#field_unwraps),*
                                })
                            }
                        }
                    }
                }
            }).collect();

            quote! {
                // Try string first (for unit variants)
                if parser.peek_is_string()? {
                    let variant_str = parser.parse_string()?;
                    match variant_str.as_ref() {
                        #(#variant_matches),*,
                        _ => Err(#krate::JsonError::UnknownVariant(variant_str.to_string()))
                    }
                } else {
                    // Object format: {"VariantName": ...}
                    parser.expect_object_start()?;
                    let key = parser.next_object_key()?.ok_or(#krate::JsonError::UnexpectedEnd)?;
                    let result = match key.as_ref() {
                        #(#variant_matches),*,
                        _ => Err(#krate::JsonError::UnknownVariant(key.to_string()))
                    };
                    parser.expect_object_end()?;
                    result
                }
            }
        }
        Data::Union(_) => {
            quote! { compile_error!("Unions are not supported"); }
        }
    }
}

fn get_field_name(attrs: &[syn::Attribute], default: &Ident) -> String {
    for attr in attrs {
        if attr.path().is_ident("json") {
            if let syn::Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                if let Some(name) = tokens.strip_prefix("rename = ") {
                    return name.trim_matches('"').to_string();
                }
            }
        }
    }
    default.to_string()
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}
