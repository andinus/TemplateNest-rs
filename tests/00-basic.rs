use template_nest::{TemplateNest, TemplateNestError, TemplateNestOption};

#[test]
fn initialize() -> Result<(), TemplateNestError> {
    TemplateNest::new(TemplateNestOption {
        directory: "templates".into(),
        ..Default::default()
    })?;
    Ok(())
}
