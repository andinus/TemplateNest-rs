use std::collections::HashMap;
use template_nest::TemplateNest;
use template_nest::{filling, Filling};

#[test]
fn render_with_show_labels() -> Result<(), String> {
    let nest = TemplateNest {
        directory: "templates".into(),
        show_labels: true,
        ..Default::default()
    };
    let page = filling!(
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    );

    let nest_no_labels = TemplateNest {
        directory: "templates".into(),
        show_labels: false,
        ..Default::default()
    };
    let page_output = filling!(
        "TEMPLATE": "output/04-simple-page-with-labels",
    );

    assert_eq!(nest.render(&page)?, nest_no_labels.render(&page_output)?,);
    Ok(())
}
