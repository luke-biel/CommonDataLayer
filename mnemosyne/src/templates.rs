use handlebars::Handlebars;

pub fn init_templates(registry: &mut Handlebars<'_>) -> anyhow::Result<()> {
    registry.register_template_file("layout", "templates/layout.hbs")?;

    Ok(())
}
