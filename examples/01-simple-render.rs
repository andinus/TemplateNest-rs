use std::collections::HashMap;
use template_nest::{filling, Filling, TemplateNest, TemplateNestError};

fn main() -> Result<(), TemplateNestError> {
    let nest = TemplateNest::new("templates")?;
    let simple_page = filling!(
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    );
    println!("{}", nest.render(&simple_page)?);
    Ok(())
}
