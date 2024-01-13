use std::collections::HashMap;
use template_nest::{filling, filling_text, Filling, TemplateNest, TemplateNestError};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn render_with_defaults() -> Result<(), TemplateNestError> {
    let nest = TemplateNest {
        directory: "templates".into(),
        defaults: HashMap::from([("variable".to_string(), filling_text!("Simple Variable"))]),
        ..Default::default()
    };
    let page = filling!(
        "TEMPLATE": "00-simple-page",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    );

    let page_output = filling!(
        "TEMPLATE": "output/01-simple-page",
    );

    assert_eq!(nest.render(&page)?, nest.render(&page_output)?,);
    Ok(())
}
