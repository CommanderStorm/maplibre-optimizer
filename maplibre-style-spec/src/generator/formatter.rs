/// Converts a string to a valid Rust struct name (UpperCamelCase)
pub fn to_upper_camel_case(name: &str) -> String {
    let name = name
        .split(|c: char| !c.is_alphanumeric()) // split on non-alphanumeric
        .filter(|s| !s.is_empty()) // skip empty parts
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<String>();
    rustize(name)
}
pub fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_was_lower = false;
    let mut prev_was_underscore = false;

    for (i, c) in name.chars().enumerate() {
        if c.is_alphanumeric() {
            if c.is_uppercase() {
                // Insert underscore before uppercase if previous was lowercase
                // or previous was not underscore and next is lowercase (e.g., "XMLHttp")
                if (prev_was_lower || (!prev_was_underscore && has_next_lower(name, i))) && i != 0 {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
                prev_was_lower = false;
                prev_was_underscore = false;
            } else {
                result.push(c);
                prev_was_lower = true;
                prev_was_underscore = false;
            }
        } else if !prev_was_underscore && !result.is_empty() {
            result.push('_');
            prev_was_underscore = true;
            prev_was_lower = false;
        }
    }

    // trim trailing underscores
    while result.ends_with('_') {
        result.pop();
    }

    rustize(result)
}

/// Helper: check if the next char is lowercase
fn has_next_lower(s: &str, idx: usize) -> bool {
    s.chars()
        .nth(idx + 1)
        .map(|c| c.is_lowercase())
        .unwrap_or(false)
}

/// replace rust
fn rustize(name: String) -> String {
    if matches!(name.as_str(), "type" | "Default") {
        format!("r#{name}")
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_upper_camel_case() {
        assert_eq!(to_upper_camel_case("my_struct_name"), "MyStructName");
        assert_eq!(to_upper_camel_case("hello world"), "HelloWorld");
        assert_eq!(to_upper_camel_case("123abc"), "123abc");
        assert_eq!(to_upper_camel_case("__weird__name__"), "WeirdName");
        assert_eq!(to_upper_camel_case("alreadyCamel"), "Alreadycamel");
    }

    #[test]
    fn test_to_snake_case_basic() {
        assert_eq!(to_snake_case("MyStructName"), "my_struct_name");
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("Already_Snake_Case"), "already_snake_case");
        assert_eq!(to_snake_case("Dashing-Name"), "dashing_name");
        assert_eq!(to_snake_case("-small_dashing-name-"), "small_dashing_name");
        assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
    }

    #[test]
    fn test_reserved_words() {
        // reserved
        assert_eq!(to_snake_case("type"), "r#type");
        assert_eq!(to_upper_camel_case("default"), "r#Default");
        // not reserved
        assert_eq!(to_upper_camel_case("type"), "Type");
        assert_eq!(to_snake_case("default"), "default");
    }
}