// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Formatter, Result as FormatterResult};
use std::ptr;

use url_parser_derive::QueryParams;
use url_parser_trait::QueryParams;


#[derive(QueryParams)]
struct BasicTypes<'a> {
    u8: u8,
    f32: f32,
    bool: bool,
    string: String,
    str: &'a str,
    char: char,
    array_u8: [u8; 3],
    array_bool: [bool; 3],
    array_string: [String; 3],
    array_str: [&'a str; 3],
    array_char: [char; 3],
    tuple: (u8, bool, String, &'a str, char),
    slice: &'a [u8],
}


#[derive(QueryParams)]
struct OptionTypes<'a> {
    opt_u8: Option<u8>,
    opt_f32: Option<f32>,
    opt_bool: Option<bool>,
    opt_string: Option<String>,
    opt_str: Option<&'a str>,
    opt_char: Option<char>,
}


#[derive(QueryParams)]
struct VectorTypes<'a> {
    vec_u8: Vec<u8>,
    vec_f32: Vec<f32>,
    vec_bool: Vec<bool>,
    vec_string: Vec<String>,
    vec_str: Vec<&'a str>,
    vec_char: Vec<char>,
}


#[derive(QueryParams)]
struct PtrTypes<'a> {
    ptr_u8: *const u8,
    array_ptr_u8: [*const u8; 3],
    ptr_array_u8: *const [u8; 3],
    tuple_ptr_u8: (*const u8,),
    ptr_tuple_u8: *const (u8,),
    slice_ptr_u8: &'a [*const u8],
    ptr_slice_u8: *const &'a [u8],
    opt_ptr_u8: Option<*const u8>,
    ptr_opt_u8: *const Option<u8>,
    vec_ptr_u8: Vec<*const u8>,
    ptr_vec_u8: *const Vec<u8>,
}


#[derive(QueryParams)]
struct EmptyStruct {}


struct CustomType {
    value: String,
}

impl CustomType {
    fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl Display for CustomType {
    fn fmt(&self, fmt: &mut Formatter) -> FormatterResult {
        write!(fmt, "{}", self.value)?;
        Ok(())
    }
}


#[derive(QueryParams)]
struct CustomTypes {
    key: CustomType,
}


#[test]
fn basic_types() {
    let param: BasicTypes = BasicTypes {
        u8: 1,
        f32: 1.5,
        bool: true,
        string: "String".to_string(),
        str: "str",
        char: 'c',
        array_u8: [1, 2, 3],
        array_bool: [true, false, true],
        array_string: ["A".to_string(), "B".to_string(), "C".to_string()],
        array_str: ["a", "b", "c"],
        array_char: ['a', 'b', 'c'],
        tuple: (1, true, "String".to_string(), "str", 'c'),
        slice: &[1, 2, 3],
    };
    assert_eq!(
        param.to_query_params(),
        concat!(
            "?u8=1&f32=1.5&bool=true&string=String&str=str&char=c&array_u8=1,2,3&array_bool=true,false,true",
            "&array_string=A,B,C&array_str=a,b,c&array_char=a,b,c&tuple=1,true,String,str,c&slice=1,2,3",
        )
        .to_string(),
    );
}


#[test]
fn option_types() {
    let param1: OptionTypes = OptionTypes {
        opt_u8: Some(1),
        opt_f32: None,
        opt_bool: Some(true),
        opt_string: None,
        opt_str: Some("str"),
        opt_char: None,
    };
    assert_eq!(
        param1.to_query_params(),
        "?opt_u8=1&opt_bool=true&opt_str=str".to_string(),
    );

    let param2: OptionTypes = OptionTypes {
        opt_u8: None,
        opt_f32: Some(1.5),
        opt_bool: None,
        opt_string: Some("String".to_string()),
        opt_str: None,
        opt_char: Some('c'),
    };
    assert_eq!(
        param2.to_query_params(),
        "?opt_f32=1.5&opt_string=String&opt_char=c".to_string(),
    );

    let param3: OptionTypes = OptionTypes {
        opt_u8: None,
        opt_f32: None,
        opt_bool: None,
        opt_string: None,
        opt_str: None,
        opt_char: None,
    };
    assert_eq!(param3.to_query_params(), "".to_string());
}


#[test]
fn vector_types() {
    let param1: VectorTypes = VectorTypes {
        vec_u8: vec![1, 2, 3],
        vec_f32: vec![1.5, 2.5, 3.5],
        vec_bool: vec![true, false],
        vec_string: vec!["St".to_string(), "ri".to_string(), "ng".to_string()],
        vec_str: vec!["st", "r"],
        vec_char: vec!['c', 'h', 'a', 'r'],
    };
    assert_eq!(
        param1.to_query_params(),
        concat!(
            "?vec_u8=1,2,3&vec_f32=1.5,2.5,3.5&vec_bool=true,false",
            "&vec_string=St,ri,ng&vec_str=st,r&vec_char=c,h,a,r",
        )
        .to_string(),
    );

    let param2: VectorTypes = VectorTypes {
        vec_u8: vec![],
        vec_f32: vec![],
        vec_bool: vec![],
        vec_string: vec![],
        vec_str: vec![],
        vec_char: vec![],
    };
    assert_eq!(param2.to_query_params(), "".to_string());
}


#[test]
fn ptr_types() {
    let ptr_slice_u8: &[u8] = &[1, 2, 3];

    let param1: PtrTypes = PtrTypes {
        ptr_u8: &1,
        array_ptr_u8: [ptr::null(), ptr::null(), ptr::null()],
        ptr_array_u8: &[1, 2, 3],
        tuple_ptr_u8: (ptr::null(),),
        ptr_tuple_u8: &(1,),
        slice_ptr_u8: &[ptr::null()],
        ptr_slice_u8: &ptr_slice_u8,
        opt_ptr_u8: Some(ptr::null()),
        ptr_opt_u8: &Some(1),
        vec_ptr_u8: vec![ptr::null()],
        ptr_vec_u8: &vec![1, 2, 3],
    };
    assert_eq!(
        param1.to_query_params(),
        "?ptr_u8=1&ptr_array_u8=1,2,3&ptr_tuple_u8=1&ptr_slice_u8=1,2,3&ptr_opt_u8=1&ptr_vec_u8=1,2,3".to_string(),
    );

    let param2: PtrTypes = PtrTypes {
        ptr_u8: ptr::null(),
        array_ptr_u8: [&1, &2, &3],
        ptr_array_u8: ptr::null(),
        tuple_ptr_u8: (&1,),
        ptr_tuple_u8: ptr::null(),
        slice_ptr_u8: &[&1, &2, &3],
        ptr_slice_u8: ptr::null(),
        opt_ptr_u8: Some(&1),
        ptr_opt_u8: ptr::null(),
        vec_ptr_u8: vec![&1, &2, &3],
        ptr_vec_u8: ptr::null(),
    };
    assert_eq!(
        param2.to_query_params(),
        "?array_ptr_u8=1,2 3&tuple_ptr_u8=1&slice_ptr_u8=1,2,3&opt_ptr_u8=1&vec_ptr_u8=1,2 3".to_string(),
    );

    let param3: PtrTypes = PtrTypes {
        ptr_u8: ptr::null(),
        array_ptr_u8: [ptr::null(), ptr::null(), ptr::null()],
        ptr_array_u8: ptr::null(),
        tuple_ptr_u8: (ptr::null(),),
        ptr_tuple_u8: ptr::null(),
        slice_ptr_u8: &[ptr::null()],
        ptr_slice_u8: ptr::null(),
        opt_ptr_u8: Some(ptr::null()),
        ptr_opt_u8: ptr::null(),
        vec_ptr_u8: vec![ptr::null()],
        ptr_vec_u8: ptr::null(),
    };
    assert_eq!(param3.to_query_params(), "".to_string());
}


#[test]
fn empty_struct() {
    let param: EmptyStruct = EmptyStruct {};
    assert_eq!(param.to_query_params(), "".to_string());
}


#[test]
fn custom_type() {
    let param: CustomTypes = CustomTypes {
        key: CustomType::new("value"),
    };
    assert_eq!(param.to_query_params(), "?key=value".to_string());
}
