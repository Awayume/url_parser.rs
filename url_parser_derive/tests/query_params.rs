// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Formatter, Result as FormatterResult};
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
}


#[derive(QueryParams)]
struct OptionTypes<'a> {
    opt_u8: Option<u8>,
    opt_f32: Option<f32>,
    opt_bool: Option<bool>,
    opt_string: Option<String>,
    opt_str: Option<&'a str>,
    opt_char: Option<char>,
    opt_array_u8: Option<[u8; 3]>,
    opt_tuple: Option<(u8, bool, String, &'a str, char)>,
    opt_vec_u8: Option<Vec<u8>>,
}


#[derive(QueryParams)]
struct VectorTypes<'a> {
    vec_u8: Vec<u8>,
    vec_f32: Vec<f32>,
    vec_bool: Vec<bool>,
    vec_string: Vec<String>,
    vec_str: Vec<&'a str>,
    vec_char: Vec<char>,
    vec_opt_u8: Vec<Option<u8>>,
}


#[derive(QueryParams)]
struct EmptyStruct {}


struct CustomType {
    value: String,
}

impl CustomType {
    fn new(value: &str) -> Self {
        Self {value: value.to_string()}
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
        f32: 1.0,
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
    };
    assert_eq!(
        param.to_query_params(),
        concat!(
            "?u8=1&f32=1.0&bool=true&string=String&str=str&char=c&array_u8=1,2,3&array_bool=true,false,true",
            "&array_string=A,B,C&array_str=a,b,c&array_char=a,b,c&tuple=1,true,String, str,c",
        ).to_string(),
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
        opt_array_u8: Some([1, 2, 3]),
        opt_tuple: None,
        opt_vec_u8: Some(vec![1, 2, 3]),
    };
    assert_eq!(
        param1.to_query_params(),
        "?opt_u8=1&opt_bool=true&opt_str=str&opt_array_u8=1,2,3&opt_vec_u8=1,2,3".to_string(),
    );

    let param2: OptionTypes = OptionTypes {
        opt_u8: None,
        opt_f32: Some(1.0),
        opt_bool: None,
        opt_string: Some("String".to_string()),
        opt_str: None,
        opt_char: Some('c'),
        opt_array_u8: None,
        opt_tuple: Some((1, true, "String".to_string(), "str", 'c')),
        opt_vec_u8: None,
    };
    assert_eq!(
        param2.to_query_params(),
        "?opt_f32=1.0&opt_string=String&opt_char=c&opt_tuple=1,true,String,str,c".to_string(),
    );

    let param3: OptionTypes = OptionTypes {
        opt_u8: None,
        opt_f32: None,
        opt_bool: None,
        opt_string: None,
        opt_str: None,
        opt_char: None,
        opt_array_u8: None,
        opt_tuple: None,
        opt_vec_u8: None,
    };
    assert_eq!(param3.to_query_params(), "".to_string());
}


#[test]
fn vector_types() {
    let param1: VectorTypes = VectorTypes {
        vec_u8: vec![1, 2, 3],
        vec_f32: vec![1.0, 2.0, 3.0],
        vec_bool: vec![true, false],
        vec_string: vec!["St".to_string(), "ri".to_string(), "ng".to_string()],
        vec_str: vec!["st", "r"],
        vec_char: vec!['c', 'h', 'a', 'r'],
        vec_opt_u8: vec![Some(1), Some(2), Some(3)],
    };
    assert_eq!(
        param1.to_query_params(),
        concat!(
            "?vec_u8=1,2,3&vec_f32=1.0,2.0,3.0&vec_bool=true,false&vec_string=St,ri,ng",
            "&vec_str=st,r&vec_char=c,h,a,r&vec_opt_u8=1,2,3"
        ).to_string(),
    );

    let param2: VectorTypes = VectorTypes {
        vec_u8: vec![1, 2, 3],
        vec_f32: vec![1.0, 2.0, 3.0],
        vec_bool: vec![true, false],
        vec_string: vec!["St".to_string(), "ri".to_string(), "ng".to_string()],
        vec_str: vec!["st", "r"],
        vec_char: vec!['c', 'h', 'a', 'r'],
        vec_opt_u8: vec![Some(1), None, Some(3)],
    };
    assert_eq!(
        param2.to_query_params(),
        concat!(
            "?vec_u8=1,2,3&vec_f32=1.0,2.0,3.0&vec_bool=true,false&vec_string=St,ri,ng",
            "&vec_str=st,r&vec_char=c,h,a,r&vec_opt_u8=1,3",
        ).to_string(),
    );

    let param3: VectorTypes = VectorTypes {
        vec_u8: vec![1, 2, 3],
        vec_f32: vec![1.0, 2.0, 3.0],
        vec_bool: vec![true, false],
        vec_string: vec!["St".to_string(), "ri".to_string(), "ng".to_string()],
        vec_str: vec!["st", "r"],
        vec_char: vec!['c', 'h', 'a', 'r'],
        vec_opt_u8: vec![None, None, None],
    };
    assert_eq!(
        param3.to_query_params(),
        "?vec_u8=1,2,3&vec_f32=1.0,2.0,3.0&vec_bool=true,false&vec_string=St,ri,ng&vec_str=st,r&vec_char=c,h,a,r".to_string(),
    );
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
