use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::iter;

pub fn symbol_name_is_legal(symbol_name: &str) -> bool {
    lazy_static! {
        static ref LEGAL_NAME_REGEX: Regex =
            Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    }
    LEGAL_NAME_REGEX.is_match(symbol_name)
}

pub fn literalize(
    symbol_name: &str,
    file: impl Iterator<Item = u8>,
) -> impl Iterator<Item = u8> {
    const INCLUDE_GUARD: &str = "#pragma once";
    const STDINT_INCLUDE: &str = "#include <stdint.h>";
    let byte_chunks =
        Itertools::intersperse(file.map(|byte| byte.to_string()), ", ".to_owned());

    let chunks = iter::once(INCLUDE_GUARD.to_owned())
        .chain(iter::once("\n".to_owned()))
        .chain(iter::once(STDINT_INCLUDE.to_owned()))
        .chain(iter::once("\n".to_owned()))
        .chain(iter::once("static const uint8_t".to_owned()))
        .chain(iter::once(" ".to_owned()))
        .chain(iter::once(symbol_name.to_owned()))
        .chain(iter::once("[]".to_owned()))
        .chain(iter::once(" = ".to_owned()))
        .chain(iter::once("{ ".to_owned()))
        .chain(byte_chunks)
        .chain(iter::once(" }".to_owned()))
        .chain(iter::once(";".to_owned()));

    chunks.map(String::into_bytes).flat_map(Vec::into_iter)
}
