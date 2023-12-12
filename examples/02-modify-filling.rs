use std::collections::HashMap;
use template_nest::TemplateNest;
use template_nest::{filling, filling_list, filling_text, Filling};

/// Example to demonstrate modifying the template hash after it has been
/// created.
fn main() {
    let nest = TemplateNest::new("templates").unwrap();
    let mut simple_page = filling!(
        "TEMPLATE": "00-simple-page",
    );

    // Modify the template hash to define this variable.
    simple_page
        .insert("variable".to_string(), filling_text!("Simple Variable"))
        .unwrap();

    // A component that goes in simple_page.
    let mut simple_component = filling_list!([
        {
            "TEMPLATE": "01-simple-component",
            "variable": "This is a variable",
        },
    ]);

    // Add more elements to the component.
    simple_component
        .push(filling!(
            "TEMPLATE": "01-simple-component",
            "variable": "This is another variable"
        ))
        .unwrap();

    // Modify the template hash to add "simple_component".
    simple_page
        .insert("simple_component".to_string(), simple_component)
        .unwrap();

    println!("{}", nest.render(&simple_page).unwrap());
}
