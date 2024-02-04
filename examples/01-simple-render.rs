use serde_json::json;
use template_nest::{TemplateNest, TemplateNestError, TemplateNestOption};

fn main() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        ..Default::default()
    })?;
    let simple_page = json!({
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    });
    println!("{}", nest.render(&simple_page)?);
    Ok(())
}
