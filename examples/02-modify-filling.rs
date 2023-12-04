use std::collections::HashMap;
use template_nest::TemplateNest;
use template_nest::{filling, filling_text, Filling};

/// Example to demonstrate modifying the template hash after it has been
/// created.
fn main() {
    let nest = TemplateNest::new("templates").unwrap();
    let mut simple_page = filling!(
        "TEMPLATE": "00-simple-page",
        "simple_component":  {
            "TEMPLATE":"01-simple-component",
            "variable": "Simple Variable in Simple Component"
        }
    );
    match simple_page {
        Filling::Template(ref mut map) => {
            map.insert("variable".to_string(), filling_text!("Simple Variable"));
        }
        _ => {}
    }

    println!("{}", nest.render(&simple_page).unwrap());
}
