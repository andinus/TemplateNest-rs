use serde_json::json;
use template_nest::{TemplateNest, TemplateNestOption};

fn main() {
    let nest = TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        label: "NAME".to_string(),
        ..Default::default()
    })
    .unwrap();

    let simple_page = json!({
        "NAME": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "NAME":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    });
    println!("{}", nest.render(&simple_page).unwrap());
}
