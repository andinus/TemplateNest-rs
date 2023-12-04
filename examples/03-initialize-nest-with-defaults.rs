use std::collections::HashMap;
use template_nest::TemplateNest;
use template_nest::{filling, Filling};

fn main() {
    let nest = TemplateNest {
        directory: "templates".into(),
        label: &"NAME",
        ..Default::default()
    };
    let simple_page = filling!(
        "NAME": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "NAME":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    );
    println!("{}", nest.render(&simple_page).unwrap());
}
