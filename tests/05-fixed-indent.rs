use serde_json::json;
use template_nest::{TemplateNest, TemplateNestError};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn render_simple_page_with_fixed_indent() -> Result<(), TemplateNestError> {
    let nest = TemplateNest {
        directory: "templates".into(),
        fixed_indent: true,
        ..Default::default()
    };
    let page = json!({
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE": "02-simple-component-multi-line",
        }
    });
    let page_output = json!({
        "TEMPLATE": "output/07-simple-page-fixed-indent",
    });

    assert_eq!(nest.render(&page)?, nest.render(&page_output)?,);
    Ok(())
}

#[test]
fn render_complex_page_with_fixed_indent() -> Result<(), TemplateNestError> {
    let nest = TemplateNest {
        directory: "templates".into(),
        fixed_indent: true,
        ..Default::default()
    };
    let page = json!({
        "TEMPLATE": "10-complex-page",
        "title": "Complex Page",
        "pre_body": {
            "TEMPLATE": "18-styles",
        },
        "navigation": {
            "TEMPLATE": "11-navigation",
            "banner": {
                "TEMPLATE": "12-navigation-banner",
            },
            "items": [
                { "TEMPLATE": "13-navigation-item-00-services" },
                { "TEMPLATE": "13-navigation-item-01-resources" },
            ]
        },
        "hero_section": {
            "TEMPLATE": "14-hero-section",
        },
        "main_content": [
            { "TEMPLATE": "15-isdc-card", },
            {
                "TEMPLATE": "16-vb-brand-cards",
                "cards": [
                    {
                        "TEMPLATE": "17-vb-brand-card-00",
                        "parent_classes": "p-card brand-card col-4",
                    },
                    {
                        "TEMPLATE": "17-vb-brand-card-01",
                        "parent_classes": "p-card brand-card col-4",
                    },
                    {
                        "TEMPLATE": "17-vb-brand-card-02",
                        "parent_classes": "p-card brand-card col-4",
                    },
                ]
            }
        ],
        "post_footer": {
            "TEMPLATE": "19-scripts"
        }
    });
    let page_output = json!({
        "TEMPLATE": "output/08-complex-page-fixed-indent",
    });

    assert_eq!(nest.render(&page)?, nest.render(&page_output)?,);
    Ok(())
}
