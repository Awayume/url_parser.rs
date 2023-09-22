// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Path, Ident, Item, Type, TypeArray, TypePath, TypePtr,
    TypeReference, TypeSlice, TypeTuple,
};


#[proc_macro_derive(QueryParams)]
pub fn derive_query_params(input: TokenStream) -> TokenStream {
    if let Item::Struct(ast) = parse_macro_input!(input) {
        let ident: Ident = ast.ident;
        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
        let mut query_generator: TokenStream2 = TokenStream2::new();
        for field in ast.fields {
            let field_ident: Ident = field.ident.unwrap();
            match field.ty {
                Type::Array(tarray) => query_generator = parse_type_array(&field_ident, tarray, query_generator),
                Type::Path(tpath) => query_generator = parse_type_path(&field_ident, tpath, query_generator),
                Type::Ptr(tptr) => query_generator = parse_type_ptr(&field_ident, tptr, query_generator),
                Type::Reference(tref) => query_generator = parse_type_reference(&field_ident, tref, query_generator),
                Type::Slice(tslice) => query_generator = parse_type_slice(&field_ident, tslice, query_generator),
                Type::Tuple(ttuple) => query_generator = parse_type_tuple(&field_ident, ttuple, query_generator),
                _ => query_generator = unsupported_field_type_error(&field_ident, query_generator),
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
fn parse_type_array(field_ident: &Ident, tarray: TypeArray, mut query_generator: TokenStream2) -> TokenStream2 {
    match *tarray.elem {
        Type::Path(_) => query_generator = parse_array(&field_ident, query_generator),
        Type::Ptr(tptr) => todo!(),
        Type::Reference(tref) => todo!(),
        _ => query_generator = unsupported_field_type_error(&field_ident, query_generator),
    }
    query_generator
}


#[inline]
fn parse_type_path(field_ident: &Ident, tpath: TypePath, mut query_generator: TokenStream2) -> TokenStream2 {
    let option_path: Path = parse_quote!(Option);
    let vec_path: Path = parse_quote!(Vec);
    for seg in tpath.path.segments {
        if seg.ident == option_path.segments[0].ident {  // Option
            query_generator = parse_option(&field_ident, query_generator);
        } else if seg.ident == vec_path.segments[0].ident {  // Vec
            query_generator = parse_vector(&field_ident, query_generator);
        } else {  // Others
            query_generator = parse_impl_display(&field_ident, query_generator);
        }
    }
    query_generator
}


#[inline]
fn parse_type_ptr(field_ident: &Ident, tptr: TypePtr, mut query_generator: TokenStream2) -> TokenStream2 {
    todo!();
}


#[inline]
fn parse_type_reference(field_ident: &Ident, tref: TypeReference, mut query_generator: TokenStream2) -> TokenStream2 {
    todo!();
}


#[inline]
fn parse_type_slice(field_ident: &Ident, tslice: TypeSlice, mut query_generator: TokenStream2) -> TokenStream2 {
    todo!();
}


#[inline]
fn parse_type_tuple(field_ident: &Ident, ttuple: TypeTuple, mut query_generator: TokenStream2) -> TokenStream2 {
    todo!();
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
