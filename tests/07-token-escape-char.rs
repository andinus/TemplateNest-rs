use std::collections::HashMap;
use std::fs;
use template_nest::{filling, Filling, TemplateNest, TemplateNestError};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn render_with_escaped_variable() -> Result<(), TemplateNestError> {
    let nest = TemplateNest {
        directory: "templates".into(),
        token_escape_char: "\\".to_string(),
        ..Default::default()
    };
    let page = filling!(
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE": "01-simple-component-token-escape",
        }
    );

    let mut output_file = nest.directory.clone();
    output_file.push("output/09-simple-page-token-escape.html");

    assert_eq!(nest.render(&page)?, fs::read_to_string(output_file)?.trim());
    Ok(())
}

/// Test if we can handle files where token is at the beginning of the file.
#[test]
fn render_with_escaped_variable_at_start() -> Result<(), TemplateNestError> {
    let nest = TemplateNest {
        directory: "templates".into(),
        token_escape_char: "\\".to_string(),
        ..Default::default()
    };
    let page = filling!(
        "TEMPLATE": "03-var-at-begin",
        "variable": "Simple Variable",
    );

    let mut output_file = nest.directory.clone();
    output_file.push("output/10-var-at-begin.html");

    assert_eq!(nest.render(&page)?, fs::read_to_string(output_file)?.trim());
    Ok(())
}
