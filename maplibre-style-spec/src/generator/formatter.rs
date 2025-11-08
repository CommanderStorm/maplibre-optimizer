pub fn to_upper_camel_case(name: &str) -> String {
    let name = prefilter_names(name);
    let result = name
        .split(|c: char| !c.is_alphanumeric()) // split on non-alphanumeric
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => {
                    let rest = chars.as_str();
                    // Lowercase only if the token is fully uppercase (e.g., "XML")
                    if s.chars().all(|c| c.is_uppercase()) {
                        first.to_uppercase().collect::<String>() + &rest.to_lowercase()
                    } else {
                        first.to_uppercase().collect::<String>() + rest
                    }
                }
                None => String::new(),
            }
        })
        .collect::<String>();

    debug_assert_ne!(
        result, "",
        "{name} should not result in an empty string after conversion to snake case"
    );
    rustize(result)
}

pub fn to_snake_case(name: &str) -> String {
    let name = prefilter_names(name);
    let mut result = String::new();
    let mut prev_was_lower = false;
    let mut prev_was_underscore = false;

    for (i, c) in name.chars().enumerate() {
        if c.is_alphanumeric() {
            if c.is_uppercase() {
                // Insert underscore before uppercase if previous was lowercase
                // or previous was not underscore and next is lowercase (e.g., "XMLHttp")
                if (prev_was_lower || (!prev_was_underscore && has_next_lower(&name, i))) && i != 0
                {
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

    debug_assert_ne!(
        result, "",
        "{name} should not result in an empty string after conversion to snake case"
    );
    rustize(result)
}

/// Helper: check if the next char is lowercase
fn has_next_lower(s: &str, idx: usize) -> bool {
    s.chars()
        .nth(idx + 1)
        .map(|c| c.is_lowercase())
        .unwrap_or(false)
}

fn prefilter_names(name: impl ToString) -> String {
    let mut name = name.to_string();
    if name.starts_with("!") {
        name = format!("not {}", prefilter_names(name[1..].trim_start()));
    }
    match name.as_str() {
        "%" => "Percentage".to_string(),
        "*" => "Star".to_string(),
        "+" => "Plus".to_string(),
        "-" => "Minus".to_string(),
        "/" => "Slash".to_string(),
        "<" => "Less".to_string(),
        "<=" => "LessEqual".to_string(),
        ">" => "Greater".to_string(),
        ">=" => "GreaterEqual".to_string(),
        "=" | "==" => "Equal".to_string(),
        "abs" => "Absolute".to_string(),
        "acos" => "Arccosine".to_string(),
        "^" => "Power".to_string(),
        _ => name,
    }
}

/// replace rust names with r# prefix if they are reserved keywords
fn rustize(name: String) -> String {
    if name.as_str() == "Default" {
        return "DefaultStruct".to_string();
    }
    if matches!(name.as_str(), "type") {
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
        assert_eq!(to_upper_camel_case("alreadyCamel"), "AlreadyCamel");
        assert_eq!(
            to_upper_camel_case("ColorRamp ColorRamp"),
            "ColorRampColorRamp"
        );
        assert_eq!(
            to_upper_camel_case("color_ramp color_ramp"),
            "ColorRampColorRamp"
        );
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
        assert_eq!(to_upper_camel_case("default"), "DefaultStruct");
        // not reserved
        assert_eq!(to_upper_camel_case("type"), "Type");
        assert_eq!(to_snake_case("default"), "default");
    }

    #[test]
    fn test_weird_names_snake_case() {
        assert_eq!(to_snake_case("!"), "not");
        assert_eq!(to_snake_case("!="), "not_equal");
        assert_eq!(to_snake_case("!has"), "not_has");
        assert_eq!(to_snake_case("%"), "percentage");
    }
    #[test]
    fn test_weird_names_upper_camel_case() {
        assert_eq!(to_upper_camel_case("!"), "Not");
        assert_eq!(to_upper_camel_case("!="), "NotEqual");
        assert_eq!(to_upper_camel_case("!has"), "NotHas");
        assert_eq!(to_upper_camel_case("%"), "Percentage");
    }
}
