use indoc::formatdoc;
use regex::Regex;

use std::sync::LazyLock;

pub fn symbol_name_is_legal(symbol_name: &str) -> bool {
    static LEGAL_NAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());
    LEGAL_NAME_REGEX.is_match(symbol_name)
}

pub fn literalize(
    symbol_name: &str,
    file: impl Iterator<Item = u8>,
) -> impl Iterator<Item = u8> {
    let (lower, _upper) = file.size_hint();
    let bytes_per_line = 12;
    let mut file_content = String::with_capacity(
        lower * "0xFF, ".len() + lower / bytes_per_line * "    ".len(),
    );

    for (idx, byte) in file.enumerate() {
        let chunk = smol_str::format_smolstr!(
            "0x{byte:02x},{}",
            if (idx + 1) % bytes_per_line == 0 {
                "\n    "
            } else {
                " "
            }
        );
        file_content.push_str(&chunk);
    }
    if file_content.ends_with('\n') {
        file_content.pop();
    }

    #[rustfmt::skip]
    let formatted = formatdoc!("
        #pragma once
        #include <stdint.h>
        static const uint8_t {symbol_name}[] = {{
            {file_content}
        }};
    ");
    formatted.into_bytes().into_iter()
}
