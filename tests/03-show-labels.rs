use serde_json::json;
use template_nest::{TemplateNest, TemplateNestError, TemplateNestOption};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn render_with_show_labels() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        show_labels: true,
        ..Default::default()
    })
    .unwrap();

    let page = json!({
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    });

    let nest_no_labels = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        show_labels: false,
        ..Default::default()
    })
    .unwrap();

    let page_output = json!({
        "TEMPLATE": "output/04-simple-page-with-labels",
    });

    assert_eq!(nest.render(&page)?, nest_no_labels.render(&page_output)?,);
    Ok(())
}

#[test]
fn render_with_show_labels_alt_delimiters() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        show_labels: true,
        comment_delimiters: ("<!--!".to_string(), "!-->".to_string()),
        ..Default::default()
    })
    .unwrap();

    let page = json!({
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    });

    let nest_no_labels = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        show_labels: false,
        ..Default::default()
    })
    .unwrap();

    let page_output = json!({
        "TEMPLATE": "output/05-simple-page-with-labels-alt-delims",
    });

    assert_eq!(nest.render(&page)?, nest_no_labels.render(&page_output)?,);
    Ok(())
}
