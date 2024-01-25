use serde_json::json;
use std::collections::HashMap;
use template_nest::{TemplateNest, TemplateNestError};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn render_with_defaults() -> Result<(), TemplateNestError> {
    let nest = TemplateNest {
        directory: "templates".into(),
        defaults: HashMap::from([("variable".to_string(), json!("Simple Variable"))]),
        ..Default::default()
    };
    let page = json!({
        "TEMPLATE": "00-simple-page",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    });

    let page_output = json!({
        "TEMPLATE": "output/01-simple-page",
    });

    assert_eq!(nest.render(&page)?, nest.render(&page_output)?,);
    Ok(())
}
