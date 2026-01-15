//! Execution context for workflow variables.

use anyhow::{Context as _, Result};
use handlebars::Handlebars;
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Execution context that holds variables and results.
#[derive(Debug, Default)]
pub struct Context {
    /// Named variables from step outputs
    variables: HashMap<String, Value>,

    /// Results from each step (accessed via $prev)
    results: Vec<Value>,

    /// Handlebars template engine
    #[allow(dead_code)]
    handlebars: Handlebars<'static>,
}

impl Context {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            results: Vec::new(),
            handlebars: Handlebars::new(),
        }
    }

    /// Set a variable.
    pub fn set(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// Get a variable.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Push a result onto the results stack.
    pub fn push_result(&mut self, value: Value) {
        self.results.push(value);
    }

    /// Get the previous result ($prev).
    pub fn prev(&self) -> Option<&Value> {
        self.results.last()
    }

    /// Get all results.
    pub fn results(&self) -> &[Value] {
        &self.results
    }

    /// Resolve a value, expanding any templates.
    ///
    /// Templates are marked with `__template__` key and use Handlebars syntax.
    pub fn resolve(&self, value: &Value) -> Result<Value> {
        match value {
            Value::Object(map) => {
                // Check if this is a template
                if let Some(template) = map.get("__template__") {
                    if let Value::String(template_str) = template {
                        return self.render_template(template_str);
                    }
                }

                // Recursively resolve object values
                let mut result = Map::new();
                for (k, v) in map {
                    result.insert(k.clone(), self.resolve(v)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
                let resolved: Result<Vec<Value>> = arr.iter()
                    .map(|v| self.resolve(v))
                    .collect();
                Ok(Value::Array(resolved?))
            }
            Value::String(s) => {
                // Check for inline templates {{ ... }}
                if s.contains("{{") && s.contains("}}") {
                    self.render_template(s)
                } else {
                    Ok(value.clone())
                }
            }
            _ => Ok(value.clone()),
        }
    }

    /// Render a Handlebars template string.
    fn render_template(&self, template: &str) -> Result<Value> {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(false);

        // Build context data
        let mut data = self.variables.clone();

        // Add $prev
        if let Some(prev) = self.prev() {
            data.insert("$prev".to_string(), prev.clone());
            data.insert("prev".to_string(), prev.clone());
        }

        // Add $results
        data.insert("$results".to_string(), Value::Array(self.results.clone()));
        data.insert("results".to_string(), Value::Array(self.results.clone()));

        let rendered = hb.render_template(template, &data)
            .context("Failed to render template")?;

        // Try to parse as JSON, otherwise return as string
        match serde_json::from_str::<Value>(&rendered) {
            Ok(v) => Ok(v),
            Err(_) => Ok(Value::String(rendered)),
        }
    }

    /// Get all variables as a JSON object.
    pub fn as_json(&self) -> Value {
        let mut data = Map::new();

        for (k, v) in &self.variables {
            data.insert(k.clone(), v.clone());
        }

        if let Some(prev) = self.prev() {
            data.insert("$prev".to_string(), prev.clone());
        }

        data.insert("$results".to_string(), Value::Array(self.results.clone()));

        Value::Object(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_variables() {
        let mut ctx = Context::new();
        ctx.set("email", serde_json::json!({"subject": "Test"}));

        assert!(ctx.get("email").is_some());
        assert_eq!(
            ctx.get("email").unwrap().get("subject"),
            Some(&Value::String("Test".to_string()))
        );
    }

    #[test]
    fn test_context_results() {
        let mut ctx = Context::new();
        ctx.push_result(serde_json::json!({"id": 1}));
        ctx.push_result(serde_json::json!({"id": 2}));

        assert_eq!(ctx.results().len(), 2);
        assert_eq!(ctx.prev().unwrap().get("id"), Some(&Value::from(2)));
    }

    #[test]
    fn test_template_resolution() {
        let mut ctx = Context::new();
        ctx.set("name", Value::String("Alice".to_string()));

        let template = serde_json::json!({"__template__": "Hello, {{ name }}!"});
        let resolved = ctx.resolve(&template).unwrap();

        assert_eq!(resolved, Value::String("Hello, Alice!".to_string()));
    }

    #[test]
    fn test_inline_template() {
        let mut ctx = Context::new();
        ctx.set("url", Value::String("https://example.com".to_string()));

        let value = Value::String("Visit {{ url }}".to_string());
        let resolved = ctx.resolve(&value).unwrap();

        assert_eq!(resolved, Value::String("Visit https://example.com".to_string()));
    }
}
