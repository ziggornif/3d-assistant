use once_cell::sync::Lazy;
use std::sync::RwLock;
use tera::Tera;

/// Global Tera instance for template rendering
static TEMPLATES: Lazy<RwLock<Tera>> = Lazy::new(|| {
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to load templates: {}", e);
            panic!("Template parsing error: {}", e);
        }
    };
    RwLock::new(tera)
});

/// Initialize templates from the given directory
pub fn init_templates(template_dir: &str) -> Result<(), tera::Error> {
    let pattern = format!("{}/**/*", template_dir);
    let tera = Tera::new(&pattern)?;

    let mut templates = TEMPLATES.write().unwrap();
    *templates = tera;

    tracing::info!("Templates loaded from: {}", template_dir);
    Ok(())
}

/// Render a template with the given context
pub fn render(template_name: &str, context: &tera::Context) -> Result<String, tera::Error> {
    let templates = TEMPLATES.read().unwrap();
    templates.render(template_name, context)
}

/// Reload templates (useful for development)
#[allow(dead_code)]
pub fn reload_templates(template_dir: &str) -> Result<(), tera::Error> {
    init_templates(template_dir)
}
