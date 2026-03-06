//! Virtual-path normalization helpers shared across host abstractions.

/// Normalizes a virtual filesystem path using Explorer UI semantics.
///
/// This helper trims whitespace, converts backslashes to `/`, resolves `.`/`..`, ensures a
/// leading slash, and returns `/` for empty or fully-collapsed paths.
pub fn normalize_virtual_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "/".to_string();
    }

    let mut out = String::new();
    for segment in trimmed.replace('\\', "/").split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            if let Some(idx) = out.rfind('/') {
                out.truncate(idx);
            }
            continue;
        }
        out.push('/');
        out.push_str(segment);
    }

    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_virtual_path;

    #[test]
    fn normalize_virtual_path_matches_expected_cases() {
        let cases = [
            ("", "/"),
            ("   ", "/"),
            ("foo/bar", "/foo/bar"),
            ("/foo//bar/", "/foo/bar"),
            ("./foo/../bar", "/bar"),
            ("\\\\foo\\\\bar", "/foo/bar"),
            ("/../../", "/"),
        ];

        for (input, expected) in cases {
            assert_eq!(normalize_virtual_path(input), expected, "input={input:?}");
        }
    }
}
