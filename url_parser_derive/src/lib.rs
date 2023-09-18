// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Path, Ident, Item, Type};


#[proc_macro_derive(QueryParams)]
pub fn derive_query_params(input: TokenStream) -> TokenStream {
    match parse_macro_input!(input) {
        Item::Struct(ast) => {
            let ident: Ident = ast.ident;
            let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
            let mut query_generator: TokenStream2 = TokenStream2::new();
            let option_path: Path = parse_quote!(Option);
            let vec_path: Path = parse_quote!(Vec);
            for field in ast.fields {
                let field_ident: Ident = field.ident.unwrap();
                if let Type::Path(tpath) = field.ty {
                    for seg in tpath.path.segments {
                        println!("{:?}", seg.ident);
                        if seg.ident == option_path.segments[0].ident {  // Option
                            query_generator = quote! {
                                #query_generator
                                // query: String
                                if let Some(val) = self.#field_ident {
                                    query += &format!("{}={}&", stringify!(#field_ident), val);
                                }
                            };
                        } else if seg.ident == vec_path.segments[0].ident {  // Vec
                            query_generator = quote! {
                                #query_generator
                                // query: String
                                let mut val: String = self.#field_ident.iter()
                                    .fold(String::new(), |acc, v| format!("{}{},", acc, v));
                                val.pop();
                                query += &format!("{}={}&", stringify!(#field_ident), val);
                            };
                        } else {  // Others
                            query_generator = quote! {
                                #query_generator
                                // query: String
                                query += &format!("{}={}&", stringify!(#field_ident), self.#field_ident);
                            };
                        }
                    }
                 } else {
                    println!("The type of the field {} does not supported.", field_ident);
                    query_generator = quote! {
                        #query_generator
                        compile_error!("Unsupported field type detected.");
                    };
                }
            }
            let expanded: TokenStream2 = quote! {
                impl #impl_generics url_parser::QueryParams for #ident #ty_generics #where_clause {
                    fn to_query_params(&self) -> String {
                        let mut query: String = "?".to_string();
                        #query_generator
                        query.pop();
                        query
                    }
                }
            };
            expanded.into()
        },
        _ => {
            let expanded: TokenStream2 = quote! {
                compile_error!("This derive macro can only be used for struct.");
            };
            expanded.into()
        },
    }
}
