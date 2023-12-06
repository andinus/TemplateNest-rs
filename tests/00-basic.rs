use template_nest::{TemplateNest, TemplateNestError};

#[test]
fn initialize() -> Result<(), TemplateNestError> {
    TemplateNest::new("templates")?;
    Ok(())
}
