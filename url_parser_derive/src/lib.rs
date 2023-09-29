// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use std::str::FromStr;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    GenericArgument, parse_macro_input, parse_quote, Path, PathArguments, Ident, Item, Type,
    TypeArray, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
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


fn unwrap_boxed_type_path(tbox: Box<Type>) -> Result<TypePath, ()> {
    match *tbox {
        Type::Path(tpath) => Ok(tpath),
        Type::Ptr(tptr) => unwrap_boxed_type_path(tptr.elem),
        Type::Reference(tref) => unwrap_boxed_type_path(tref.elem),
        _ => Err(()),
    }
}


#[inline]
fn get_type_argument(tpath: &TypePath) -> Result<Type, ()> {
    if let PathArguments::AngleBracketed(garg) = &tpath.path.segments[0].arguments {
        if let GenericArgument::Type(ty) = &garg.args[0] {
            Ok(ty.clone())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}


#[inline]
fn is_option(tpath: &TypePath) -> bool {
    let option_path: Path = parse_quote!(Option);
    tpath.path.segments[0].ident == option_path.segments[0].ident
}


#[inline]
fn is_vec(tpath: &TypePath) -> bool {
    let vec_path: Path = parse_quote!(Vec);
    tpath.path.segments[0].ident == vec_path.segments[0].ident
}


fn parse_type_array(field_ident: &Ident, tarray: TypeArray, query_generator: TokenStream2) -> TokenStream2 {
    match *tarray.elem {
        Type::Path(tpath) => parse_slice(field_ident, tpath, query_generator),
        Type::Ptr(tptr) => {
            if let Ok(tpath) = unwrap_boxed_type_path(tptr.elem) {
                parse_slice(field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        },
        Type::Reference(tref) => {
            if let Ok(tpath) = unwrap_boxed_type_path(tref.elem) {
                parse_slice(field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        },
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_type_path(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    let option_path: Path = parse_quote!(Option);
    let vec_path: Path = parse_quote!(Vec);
    if tpath.path.segments[0].ident == option_path.segments[0].ident {  // Option
        parse_option(field_ident, tpath, query_generator)
    } else if tpath.path.segments[0].ident == vec_path.segments[0].ident {  // Vec
        parse_vector(field_ident, tpath, query_generator)
    } else {  // Others
        parse_impl_display(field_ident, query_generator)
    }
}


fn parse_type_ptr(field_ident: &Ident, tptr: TypePtr, query_generator: TokenStream2) -> TokenStream2 {
    match *tptr.elem {
        Type::Array(tarray) => parse_type_array(field_ident, tarray, query_generator),
        Type::Path(tpath) => parse_type_path(field_ident, tpath, query_generator),
        Type::Ptr(tptr) => parse_type_ptr(field_ident, tptr, query_generator),
        Type::Reference(tref) => parse_type_reference(field_ident, tref, query_generator),
        Type::Slice(tslice) => parse_type_slice(field_ident, tslice, query_generator),
        Type::Tuple(ttuple) => parse_type_tuple(field_ident, ttuple, query_generator),
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_type_reference(field_ident: &Ident, tref: TypeReference, query_generator: TokenStream2) -> TokenStream2 {
    match *tref.elem {
        Type::Array(tarray) => parse_type_array(field_ident, tarray, query_generator),
        Type::Path(tpath) => parse_type_path(field_ident, tpath, query_generator),
        Type::Ptr(tptr) => parse_type_ptr(field_ident, tptr, query_generator),
        Type::Reference(tref) => parse_type_reference(field_ident, tref, query_generator),
        Type::Slice(tslice) => parse_type_slice(field_ident, tslice, query_generator),
        Type::Tuple(ttuple) => parse_type_tuple(field_ident, ttuple, query_generator),
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_type_slice(field_ident: &Ident, tslice: TypeSlice, query_generator: TokenStream2) -> TokenStream2 {
    if let Ok(tpath) = unwrap_boxed_type_path(tslice.elem) {
        parse_slice(field_ident, tpath, query_generator)
    } else {
        unsupported_field_type_error(field_ident, query_generator)
    }
}


fn parse_type_tuple(field_ident: &Ident, ttuple: TypeTuple, query_generator: TokenStream2) -> TokenStream2 {
    if ttuple.elems.iter().all(|ty: &Type| -> bool {
        match ty {
            Type::Path(tpath) => !(is_option(&tpath) || is_vec(&tpath)),
            Type::Ptr(tptr) => {
                if let Ok(tpath) = unwrap_boxed_type_path(tptr.elem.clone()) {
                    !(is_option(&tpath) || is_vec(&tpath))
                } else {
                    false
                }
            },
            Type::Reference(tref) => {
                if let Ok(tpath) = unwrap_boxed_type_path(tref.elem.clone()) {
                    !(is_option(&tpath) || is_vec(&tpath))
                } else {
                    false
                }
            },
            _ => false,
        }
    }) {
        let size: usize = ttuple.elems.len();
        let mut template: String = "{},".to_string().repeat(size);
        template.pop();
        let mut values: TokenStream2 = TokenStream2::new();
        for i in 0..size {
            let i: TokenStream2 = TokenStream2::from_str(&i.to_string()).unwrap();
            values = quote!(#values self.#field_ident.#i,);
        }
        quote! {
            #query_generator
            // query: String
            query += &format!(#template, #values);
        }
    } else {
        unsupported_field_type_error(field_ident, query_generator)
    }
}


fn parse_slice(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    if is_option(&tpath) || is_vec(&tpath) {
        unsupported_field_type_error(field_ident, query_generator)
    } else {
        quote! {
            #query_generator
            // query: String
            let mut val: String = self.#field_ident.iter()
                .fold(String::new(), |acc, v| format!("{}{},", acc, v));
            val.pop();
            query += &format!("{}={}&", stringify!(#field_ident), val);
        }
    }
}


fn parse_option(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    match get_type_argument(&tpath).unwrap() {
        Type::Array(tarray) => todo!(),
        Type::Path(tpath) => {
            if is_option(&tpath) {
                unsupported_field_type_error(field_ident, query_generator)
            } else if is_vec(&tpath) {
                // TODO: Add type check
                quote! {
                    #query_generator
                    // query: String
                    if let Some(val) = self.#field_ident {
                        let mut val: String = self.#field_ident.iter()
                            .fold(String::new(), |acc, v| format!("{}{},", acc, v));
                        val.pop();
                        query += &format!("{}={}&", stringify!(#field_ident), val);
                    }
                }
            } else {
                quote! {
                    #query_generator
                    // query: String
                    if let Some(val) = self.#field_ident {
                        query += &format!("{}={}&", stringify!(#field_ident), val);
                    }
                }
            }
        },
        Type::Ptr(tptr) => todo!(),
        Type::Reference(tref) => todo!(),
        Type::Slice(tslice) => todo!(),
        Type::Tuple(ttuple) => todo!(),
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_vector(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    if let PathArguments::AngleBracketed(garg) = &tpath.path.segments[0].arguments {
        if let GenericArgument::Type(ty) = &garg.args[0] {
            match ty {
                Type::Path(tpath) => parse_slice(field_ident, tpath.clone(), query_generator),
                Type::Ptr(tptr) => {
                    if let Ok(tpath) = unwrap_boxed_type_path(tptr.elem.clone()) {
                        parse_slice(&field_ident, tpath, query_generator)
                    } else {
                        unsupported_field_type_error(&field_ident, query_generator)
                    }
                },
                Type::Reference(tref) => {
                    if let Ok(tpath) = unwrap_boxed_type_path(tref.elem.clone()) {
                        parse_slice(&field_ident, tpath, query_generator)
                    } else {
                        unsupported_field_type_error(&field_ident, query_generator)
                    }
                },
                _ => unsupported_field_type_error(field_ident, query_generator),
            }
        } else {
            unsupported_field_type_error(field_ident, query_generator)
        }
    } else {
        unsupported_field_type_error(field_ident, query_generator)
    }
}


fn parse_impl_display(field_ident: &Ident, query_generator: TokenStream2) -> TokenStream2 {
    quote! {
        #query_generator
        // query: String
        query += &format!("{}={}&", stringify!(#field_ident), self.#field_ident);
    }
}


fn unsupported_field_type_error(field_ident: &Ident, query_generator: TokenStream2) -> TokenStream2 {
    println!("The type of the field {} does not supported.", field_ident);
    quote! {
        #query_generator
        compile_error!("Unsupported field type detected.");
    }
}
