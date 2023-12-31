// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input,
    parse_quote,
    GenericArgument,
    Ident,
    Item,
    Path,
    PathArguments,
    Type,
    TypeArray,
    TypePath,
    TypePtr,
    TypeReference,
    TypeSlice,
    TypeTuple,
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
            if let Type::Path(tpath) = *tptr.elem {
                parse_slice_ptr(field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        Type::Reference(tref) => {
            if let Type::Path(tpath) = *tref.elem {
                parse_slice(field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_type_path(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    let option_path: Path = parse_quote!(Option);
    let vec_path: Path = parse_quote!(Vec);
    if tpath.path.segments[0].ident == option_path.segments[0].ident {
        // Option
        parse_option(field_ident, tpath, query_generator)
    } else if tpath.path.segments[0].ident == vec_path.segments[0].ident {
        // Vec
        parse_vector(field_ident, tpath, query_generator)
    } else {
        // Others
        parse_impl_display(field_ident, query_generator)
    }
}


fn parse_type_ptr(field_ident: &Ident, tptr: TypePtr, query_generator: TokenStream2) -> TokenStream2 {
    match *tptr.elem {
        Type::Array(tarray) => {
            match *tarray.elem {
                Type::Path(tpath) => parse_ptr_slice(field_ident, tpath, query_generator),
                Type::Ptr(tptr) => {
                    if let Type::Path(tpath) = *tptr.elem {
                        parse_ptr_slice_ptr(field_ident, tpath, query_generator)
                    } else {
                        unsupported_field_type_error(field_ident, query_generator)
                    }
                }
                Type::Reference(tref) => {
                    if let Type::Path(tpath) = *tref.elem {
                        parse_ptr_slice(field_ident, tpath, query_generator)
                    } else {
                        unsupported_field_type_error(field_ident, query_generator)
                    }
                }
                _ => unsupported_field_type_error(field_ident, query_generator),
            }
        }
        Type::Path(tpath) => {
            if is_option(&tpath) {
                match get_type_argument(&tpath).unwrap() {
                    Type::Path(tpath) => {
                        if is_option(&tpath) || is_vec(&tpath) {
                            unsupported_field_type_error(field_ident, query_generator)
                        } else {
                            quote! {
                                #query_generator
                                // query: String
                                unsafe {
                                    if !self.#field_ident.is_null() {
                                        if let Some(val) = *self.#field_ident {
                                            let val: String = val.to_string();
                                            if !val.is_empty() {
                                                query += &format!("{}={}&", stringify!(#field_ident), val);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Type::Ptr(tptr) => {
                        if let Type::Path(tpath) = *tptr.elem {
                            if is_option(&tpath) || is_vec(&tpath) {
                                unsupported_field_type_error(field_ident, query_generator)
                            } else {
                                quote! {
                                    #query_generator
                                    // query: String
                                    if !self.#field_ident.is_null() {
                                        unsafe {
                                            if let Some(val) = *self.#field_ident {
                                                if !val.is_null() {
                                                    let val: String = (*val).to_string();
                                                    if !val.is_empty() {
                                                        query += &format!("{}={}&", stringify!(#field_ident), val);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            unsupported_field_type_error(field_ident, query_generator)
                        }
                    }
                    Type::Reference(tref) => {
                        if let Type::Path(tpath) = *tref.elem {
                            if is_option(&tpath) || is_vec(&tpath) {
                                unsupported_field_type_error(field_ident, query_generator)
                            } else {
                                quote! {
                                    #query_generator
                                    // query: String
                                    if !self.#field_ident.is_null() {
                                        if let Some(val) = *self.#field_ident {
                                            let val: String = val.to_string();
                                            if !val.is_empty() {
                                                query += &format!("{}={}&", stringify!(#field_ident), val);
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            unsupported_field_type_error(field_ident, query_generator)
                        }
                    }
                    _ => unsupported_field_type_error(field_ident, query_generator),
                }
            } else if is_vec(&tpath) {
                match get_type_argument(&tpath).unwrap() {
                    Type::Path(tpath) => parse_ptr_slice(field_ident, tpath.clone(), query_generator),
                    Type::Ptr(tptr) => {
                        if let Type::Path(tpath) = *tptr.elem {
                            parse_ptr_slice_ptr(&field_ident, tpath, query_generator)
                        } else {
                            unsupported_field_type_error(&field_ident, query_generator)
                        }
                    }
                    Type::Reference(tref) => {
                        if let Type::Path(tpath) = *tref.elem {
                            parse_ptr_slice(&field_ident, tpath, query_generator)
                        } else {
                            unsupported_field_type_error(&field_ident, query_generator)
                        }
                    }
                    _ => unsupported_field_type_error(field_ident, query_generator),
                }
            } else {
                quote! {
                    #query_generator
                    // query: String
                    if !self.#field_ident.is_null() {
                        unsafe {
                            let val: String = (*self.#field_ident).to_string();
                            if !val.is_empty() {
                                query += &format!("{}={}&", stringify!(#field_ident), *self.#field_ident);
                            }
                        }
                    }
                }
            }
        }
        Type::Reference(tref) => {
            if let Type::Slice(tslice) = *tref.elem {
                if let Type::Path(tpath) = *tslice.elem {
                    parse_ptr_slice(field_ident, tpath, query_generator)
                } else {
                    unsupported_field_type_error(field_ident, query_generator)
                }
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        Type::Tuple(ttuple) => {
            if ttuple.elems.iter().all(|ty: &Type| -> bool {
                match ty {
                    Type::Path(tpath) => !(is_option(&tpath) || is_vec(&tpath)),
                    Type::Ptr(tptr) => {
                        if let Type::Path(tpath) = *tptr.elem.clone() {
                            !(is_option(&tpath) || is_vec(&tpath))
                        } else {
                            false
                        }
                    }
                    Type::Reference(tref) => {
                        if let Type::Path(tpath) = *tref.elem.clone() {
                            !(is_option(&tpath) || is_vec(&tpath))
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }) {
                let mut tuple_apd_query_generator: TokenStream2 = quote! {
                    let mut values: String = Default::default();
                };
                for (i, elm) in ttuple.elems.iter().enumerate() {
                    let idx: TokenStream2 = i.to_string().parse().unwrap();
                    if let Type::Ptr(_) = elm {
                        tuple_apd_query_generator = quote! {
                            #tuple_apd_query_generator
                            unsafe {
                                let value = (*self.#field_ident).#idx;
                                if !value.is_null() {
                                    values += &format!("{}," *value);
                                }
                            }
                        };
                    } else {
                        tuple_apd_query_generator = quote! {
                            #tuple_apd_query_generator
                            unsafe {
                                values += &format!("{},", (*self.#field_ident).#idx);
                            }
                        };
                    }
                }
                quote! {
                    #query_generator
                    // query: String
                    if !self.#field_ident.is_null() {
                        #tuple_apd_query_generator
                        values.pop();
                        if values.len() != 0 {
                            query += &format!("{}={}&", stringify!(#field_ident), values);
                        }
                    }
                }
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_ptr_slice(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    if is_option(&tpath) || is_vec(&tpath) {
        unsupported_field_type_error(field_ident, query_generator)
    } else {
        quote! {
            #query_generator
            // query: String
            if !self.#field_ident.is_null() {
                let mut val: String;
                unsafe {
                    val = (*self.#field_ident).iter()
                        .fold(String::new(), |acc, v| format!("{}{},", acc, v));
                }
                val.pop();
                if !val.is_empty() {
                    query += &format!("{}={}&", stringify!(#field_ident), val);
                }
            }
        }
    }
}


fn parse_slice_ptr(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    if is_option(&tpath) || is_vec(&tpath) {
        unsupported_field_type_error(field_ident, query_generator)
    } else {
        quote! {
            #query_generator
            // query: String
            let mut values: String = Default::default();
            for v in self.#field_ident.clone() {
                if !v.is_null() {
                    unsafe {
                        let val: String = v.as_ref().unwrap().to_string();
                        if !val.is_empty() {
                            values += &format!("{},", val);
                        }
                    }
                }
            }
            values.pop();
            if !values.is_empty() {
                query += &format!("{}={}&", stringify!(#field_ident), values);
            }
        }
    }
}


fn parse_ptr_slice_ptr(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    if is_option(&tpath) || is_vec(&tpath) {
        unsupported_field_type_error(field_ident, query_generator)
    } else {
        quote! {
            #query_generator
            // query: String
            if !self.#field_ident.is_null() {
                let mut values: String = Default::default();
                unsafe {
                    for v in *self.#field_ident {
                        if !v.is_null() {
                            let val: String = v.as_ref().unwrap().to_string();
                            if !val.is_empty() {
                                values += &format!("{},", val);
                            }
                        }
                    }
                }
                values.pop();
                if !values.is_empty() {
                    query += &format!("{}={}&", stringify!(#field_ident), values);
                }
            }
        }
    }
}


fn parse_type_reference(field_ident: &Ident, tref: TypeReference, query_generator: TokenStream2) -> TokenStream2 {
    match *tref.elem {
        Type::Array(tarray) => parse_type_array(field_ident, tarray, query_generator),
        Type::Path(tpath) => parse_type_path(field_ident, tpath, query_generator),
        Type::Slice(tslice) => parse_type_slice(field_ident, tslice, query_generator),
        Type::Tuple(ttuple) => parse_type_tuple(field_ident, ttuple, query_generator),
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_type_slice(field_ident: &Ident, tslice: TypeSlice, query_generator: TokenStream2) -> TokenStream2 {
    match *tslice.elem {
        Type::Path(tpath) => {
            parse_slice(field_ident, tpath, query_generator)
        }
        Type::Ptr(tptr) => {
            if let Type::Path(tpath) = *tptr.elem {
                parse_slice_ptr(field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        Type::Reference(tref) => {
            if let Type::Path(tpath) = *tref.elem {
                parse_slice(field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_type_tuple(field_ident: &Ident, ttuple: TypeTuple, query_generator: TokenStream2) -> TokenStream2 {
    if ttuple.elems.iter().all(|ty: &Type| -> bool {
        match ty {
            Type::Path(tpath) => !(is_option(&tpath) || is_vec(&tpath)),
            Type::Ptr(tptr) => {
                if let Type::Path(tpath) = *tptr.elem.clone() {
                    !(is_option(&tpath) || is_vec(&tpath))
                } else {
                    false
                }
            }
            Type::Reference(tref) => {
                if let Type::Path(tpath) = *tref.elem.clone() {
                    !(is_option(&tpath) || is_vec(&tpath))
                } else {
                    false
                }
            }
            _ => false,
        }
    }) {
        let mut tuple_apd_query_generator: TokenStream2 = quote! {
            let mut values: String = Default::default();
        };
        for (i, val) in ttuple.elems.iter().enumerate() {
            let idx: TokenStream2 = i.to_string().parse().unwrap();
            if let Type::Ptr(_) = val {
                tuple_apd_query_generator = quote! {
                    #tuple_apd_query_generator
                    if !self.#field_ident.#idx.is_null() {
                        unsafe {
                            values += &format!("{},", *self.#field_ident.#idx);
                        }
                    }
                };
            } else {
                tuple_apd_query_generator = quote! {
                    #tuple_apd_query_generator
                    // query: String
                    values += &format!("{},", self.#field_ident.#idx);
                };
            }
        }
        quote! {
            #query_generator
            // query: String
            #tuple_apd_query_generator
            if values.len() != 0 {
                values.pop();
                query += &format!("{}={}&", stringify!(#field_ident), values);
            }
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
            if !val.is_empty() {
                query += &format!("{}={}&", stringify!(#field_ident), val);
            }
        }
    }
}


fn parse_option(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    match get_type_argument(&tpath).unwrap() {
        Type::Path(tpath) => {
            if is_option(&tpath) || is_vec(&tpath) {
                unsupported_field_type_error(field_ident, query_generator)
            } else {
                quote! {
                    #query_generator
                    // query: String
                    if let Some(val) = &self.#field_ident {
                        let val: String = val.to_string();
                        if !val.is_empty() {
                            query += &format!("{}={}&", stringify!(#field_ident), val);
                        }
                    }
                }
            }
        }
        Type::Ptr(tptr) => {
            if let Type::Path(tpath) = *tptr.elem {
                if is_option(&tpath) || is_vec(&tpath) {
                    unsupported_field_type_error(field_ident, query_generator)
                } else {
                    quote! {
                        #query_generator
                        // query: String
                        if let Some(val) = &self.#field_ident {
                            if !val.is_null() {
                                unsafe {
                                    let val: String = (val.as_ref().unwrap()).to_string();
                                    if !val.is_empty() {
                                        query += &format!("{}={}&", stringify!(#field_ident), val);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        Type::Reference(tref) => {
            if let Type::Path(tpath) = *tref.elem {
                if is_option(&tpath) || is_vec(&tpath) {
                    unsupported_field_type_error(field_ident, query_generator)
                } else {
                    quote! {
                        #query_generator
                        // query: String
                        if let Some(val) = &self.#field_ident {
                            let val: String = val.to_string();
                            if !val.is_empty() {
                                query += &format!("{}={}&", stringify!(#field_ident), val);
                            }
                        }
                    }
                }
            } else {
                unsupported_field_type_error(field_ident, query_generator)
            }
        }
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_vector(field_ident: &Ident, tpath: TypePath, query_generator: TokenStream2) -> TokenStream2 {
    match get_type_argument(&tpath).unwrap() {
        Type::Path(tpath) => parse_slice(field_ident, tpath.clone(), query_generator),
        Type::Ptr(tptr) => {
            if let Type::Path(tpath) = *tptr.elem.clone() {
                parse_slice_ptr(&field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(&field_ident, query_generator)
            }
        }
        Type::Reference(tref) => {
            if let Type::Path(tpath) = *tref.elem.clone() {
                parse_slice(&field_ident, tpath, query_generator)
            } else {
                unsupported_field_type_error(&field_ident, query_generator)
            }
        }
        _ => unsupported_field_type_error(field_ident, query_generator),
    }
}


fn parse_impl_display(field_ident: &Ident, query_generator: TokenStream2) -> TokenStream2 {
    quote! {
        #query_generator
        // query: String
        let val: String = self.#field_ident.to_string();
        if !val.is_empty() {
            query += &format!("{}={}&", stringify!(#field_ident), val);
        }
    }
}


fn unsupported_field_type_error(field_ident: &Ident, query_generator: TokenStream2) -> TokenStream2 {
    println!("The type of the field {} does not supported.", field_ident);
    quote! {
        #query_generator
        compile_error!("Unsupported field type detected.");
    }
}
