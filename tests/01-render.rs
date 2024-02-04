use serde_json::json;
use template_nest::{TemplateNest, TemplateNestError, TemplateNestOption};

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn render_simple_page() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        ..Default::default()
    })?;
    let page = json!({
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
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

#[test]
fn render_incomplete_page() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        ..Default::default()
    })?;
    let page = json!({
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
        }
    });
    let page_output = json!({
        "TEMPLATE": "output/03-incomplete-page",
    });
    assert_eq!(nest.render(&page)?, nest.render(&page_output)?);
    Ok(())
}

#[test]
fn render_complex_page() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        ..Default::default()
    })?;
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
        "TEMPLATE": "output/02-complex-page",
    });
    assert_eq!(nest.render(&page)?, nest.render(&page_output)?);

    Ok(())
}

#[test]
fn render_array_of_template_hash() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        ..Default::default()
    })?;
    let page = json!([
        {
            "TEMPLATE": "01-simple-component",
            "variable": "This is a variable",
        }, {
            "TEMPLATE": "01-simple-component",
            "variable": "This is another variable",
        }
    ]);
    let page_output = json!({
        "TEMPLATE": "output/13-render-with-array-of-template-hash",
    });
    assert_eq!(nest.render(&page)?, nest.render(&page_output)?);

    Ok(())
}
