use serde_json::json;
use template_nest::TemplateNest;

fn main() {
    let nest = TemplateNest {
        directory: "templates".into(),
        label: "NAME".to_string(),
        ..Default::default()
    };
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
