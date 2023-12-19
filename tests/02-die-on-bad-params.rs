use std::collections::HashMap;
use template_nest::{filling, Filling, TemplateNest, TemplateNestError};

#[test]
fn die_on_page_with_bad_params() {
    let nest = TemplateNest {
        directory: "templates".into(),
        die_on_bad_params: true,
        ..Default::default()
    };
    let page = filling!(
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  "Simple Component",
        "a_bad_param": "Bad Param"
    );

    match nest.render(&page) {
        Err(TemplateNestError::BadParams(_)) => {},
        Err(_) => {
            panic!("Must return TemplateNestError::BadParams on bad params error.")
        }
        Ok(_) => {
            panic!("All variables in template hash must be valid.")
        }
    }
}

/// Testing with a bad parameter but with the same number of keys as the
/// template file.
#[test]
fn die_on_page_with_bad_params_01() {
    let nest = TemplateNest {
        directory: "templates".into(),
        die_on_bad_params: true,
        ..Default::default()
    };
    let page = filling!(
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "a_bad_param": "Bad Param"
    );

    match nest.render(&page) {
        Err(TemplateNestError::BadParams(_)) => {},
        Err(_) => {
            panic!("Must return TemplateNestError::BadParams on bad params error.")
        }
        Ok(_) => {
            panic!("All variables in template hash must be valid.")
        }
    }
}

#[test]
fn live_on_page_with_bad_params() {
    let nest = TemplateNest {
        directory: "templates".into(),
        die_on_bad_params: false,
        ..Default::default()
    };
    let page = filling!(
        "TEMPLATE": "00-simple-page",
        "variable": "Simple Variable",
        "simple_component":  "Simple Component",
        "a_bad_param": "Bad Param"
    );

    match nest.render(&page) {
        Err(TemplateNestError::BadParams(_)) => {
            panic!("Must not return error if die_on_bad_params is false.")
        },
        _ => {}
    }
}
