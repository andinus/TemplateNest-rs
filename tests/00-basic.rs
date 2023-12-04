use template_nest::TemplateNest;

#[test]
fn initialize() -> Result<(), String> {
    TemplateNest::new("templates")?;
    Ok(())
}
