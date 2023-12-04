use template_nest::TemplateNest;
use template_nest::{filling, Filling};
use std::collections::HashMap;

fn main() {
    let nest = TemplateNest::new("templates").unwrap();
    let simple_page = filling!(
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    );
    println!("{}", nest.render(&simple_page).unwrap());
}
