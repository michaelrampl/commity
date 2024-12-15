use minijinja::{value::Value, Environment};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;

/// Represents a field value based on the field for rendering the commit message in `render_message`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldValue {
    Text(String),
    Boolean(bool),
}

/// Renders a message using a template and provided data.
///
/// # Arguments
///
/// * `data` - A `HashMap` containing key (`String`) - value (`FieldValue`) pairs to populate the template.
/// * `template` - A string containing the `minijinja` template to be rendered.
///
/// # Errors
///
/// Returns an error if the template cannot be rendered.
///
/// # Returns
///
/// A `String` containing the rendered output.

pub fn render_message(
    data: HashMap<String, FieldValue>,
    template: &String,
) -> Result<String, Box<dyn Error>> {
    // Convert the HashMap into a MiniJinja value using `Value::from_serializable`
    let data: Value = Value::from_serialize(&data);

    // Create a MiniJinja environment
    let mut env = Environment::new();

    // Add the template to the environment
    env.add_template("template", template)?;

    // Render the template
    let tmpl = env.get_template("template")?;
    let rendered = tmpl.render(data)?;

    Ok(rendered)
}
