// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Path, Ident, Item, Type};


#[proc_macro_derive(QueryParams)]
pub fn derive_query_params(input: TokenStream) -> TokenStream {
    if let Item::Struct(ast) = parse_macro_input!(input) {
        let ident: Ident = ast.ident;
        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
        let mut query_generator: TokenStream2 = TokenStream2::new();
        let option_path: Path = parse_quote!(Option);
        let vec_path: Path = parse_quote!(Vec);
        for field in ast.fields {
            let field_ident: Ident = field.ident.unwrap();
            match field.ty {
                Type::Array(tarray) => {
                    if let Type::Path(_) = *tarray.elem {
                        query_generator = parse_array(&field_ident, query_generator);
                    } else {
                        query_generator = unsupported_field_type_error(&field_ident, query_generator);
                    }
                },
                Type::Path(tpath) => {
                    for seg in tpath.path.segments {
                        if seg.ident == option_path.segments[0].ident {  // Option
                            query_generator = parse_option(&field_ident, query_generator);
                        } else if seg.ident == vec_path.segments[0].ident {  // Vec
                            query_generator = parse_vector(&field_ident, query_generator);
                        } else {  // Others
                            query_generator = parse_impl_display(&field_ident, query_generator);
                        }
                    }
                },
                _ => {
                    query_generator = unsupported_field_type_error(&field_ident, query_generator);
                },
            }
        }
        let expanded: TokenStream2 = quote! {
            impl #impl_generics QueryParams for #ident #ty_generics #where_clause {
                fn to_query_params(&self) -> String {
                    let mut query: String = "?".to_string();
                    #query_generator
                    query.pop();
                    query
                }
            }
        };
        expanded.into()
    } else {
        let expanded: TokenStream2 = quote! {
            compile_error!("This derive macro can only be used for struct.");
        };
        expanded.into()
    }
}


#[inline]
fn parse_array(field_ident: &Ident, mut query_generator: TokenStream2) -> TokenStream2 {
    query_generator = quote! {
        #query_generator
        // query: String
        let mut val: String = self.#field_ident.iter()
            .fold(String::new(), |acc, v| format!("{}{},", acc, v));
        val.pop();
        query += &format!("{}={}&", stringify!(#field_ident), val);
    };
    query_generator
}


#[inline]
fn parse_option(field_ident: &Ident, mut query_generator: TokenStream2) -> TokenStream2 {
    query_generator = quote! {
        #query_generator
        // query: String
        if let Some(val) = self.#field_ident {
            query += &format!("{}={}&", stringify!(#field_ident), val);
        }
    };
    query_generator
}


#[inline]
fn parse_vector(field_ident: &Ident, mut query_generator: TokenStream2) -> TokenStream2 {
    query_generator = quote! {
        #query_generator
        // query: String
        let mut val: String = self.#field_ident.iter()
            .fold(String::new(), |acc, v| format!("{}{},", acc, v));
        val.pop();
        query += &format!("{}={}&", stringify!(#field_ident), val);
    };
    query_generator
}


#[inline]
fn parse_impl_display(field_ident: &Ident, mut query_generator: TokenStream2) -> TokenStream2 {
    query_generator = quote! {
        #query_generator
        // query: String
        query += &format!("{}={}&", stringify!(#field_ident), self.#field_ident);
    };
    query_generator
}


#[inline]
fn unsupported_field_type_error(field_ident: &Ident, mut query_generator: TokenStream2) -> TokenStream2 {
    println!("The type of the field {} does not supported.", field_ident);
    query_generator = quote! {
        #query_generator
        compile_error!("Unsupported field type detected.");
    };
    query_generator
}
