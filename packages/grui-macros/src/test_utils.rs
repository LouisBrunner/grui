use proc_macro2::TokenStream;

/// Pretty-print a token stream by converting to string and normalizing.
///
/// Tests in this crate expect stable spacing and removal of trailing commas
/// before closing delimiters, so `pretty` and `prettyprint` both delegate to
/// `normalize`.
pub fn pretty(item: TokenStream) -> String {
    let s = item.to_string();
    normalize(&s)
}

/// Normalize a string representation of tokens:
/// - Remove commas that are immediately followed (possibly after whitespace)
///   by a closing delimiter `)`, `}` or `]`.
/// - Collapse consecutive whitespace to single spaces and trim.
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ',' {
            // look ahead for spaces then a closing bracket
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == ')' || chars[j] == '}' || chars[j] == ']') {
                // skip the comma and the whitespace up to the closer
                i = j;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }

    // collapse consecutive whitespace to single spaces and trim
    out.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}
