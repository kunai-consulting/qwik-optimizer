pub(crate) fn get_event_scope_data_from_jsx_event(jsx_event: &str) -> (&'static str, usize) {
    if jsx_event.starts_with("window:on") {
        ("on-window:", 9)
    } else if jsx_event.starts_with("document:on") {
        ("on-document:", 11)
    } else if jsx_event.starts_with("on") {
        ("on:", 2)
    } else {
        ("", usize::MAX)
    }
}

pub(super) fn create_event_name(name: &str, prefix: &str) -> String {
    let mut result = String::from(prefix);

    let name = if let Some(stripped) = name.strip_prefix('-') {
        for c in stripped.chars() {
            if c.is_ascii_uppercase() {
                result.push('-');
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        return result;
    } else {
        name
    };

    for c in name.chars() {
        if c.is_ascii_uppercase() {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

pub(crate) fn jsx_event_to_html_attribute(jsx_event: &str) -> Option<String> {
    if !jsx_event.ends_with('$') {
        return None;
    }

    let (prefix, idx) = get_event_scope_data_from_jsx_event(jsx_event);
    if idx == usize::MAX {
        return None;
    }

    let name = &jsx_event[idx..jsx_event.len() - 1];

    Some(create_event_name(name, prefix))
}
