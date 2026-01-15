//! Workflow step definitions.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A single step in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Service to call (e.g., "gmail", "browser")
    pub service: String,

    /// Method to call (e.g., "gmail.inbox", "browser.open")
    pub method: String,

    /// Parameters to pass to the method
    #[serde(default)]
    pub params: HashMap<String, Value>,

    /// Variable name to store the result (optional)
    #[serde(default)]
    pub output: Option<String>,

    /// Description for logging/debugging
    #[serde(default)]
    pub description: Option<String>,
}

impl Step {
    /// Create a new step with service and method.
    pub fn call(service: &str, method: &str) -> StepBuilder {
        StepBuilder::new(service, method)
    }

    /// Create a step from service name only (method inferred as service.action).
    pub fn service(service: &str) -> StepBuilder {
        StepBuilder::new(service, service)
    }
}

/// Builder for creating workflow steps.
#[derive(Debug, Clone)]
pub struct StepBuilder {
    step: Step,
}

impl StepBuilder {
    /// Create a new step builder.
    pub fn new(service: &str, method: &str) -> Self {
        Self {
            step: Step {
                service: service.to_string(),
                method: method.to_string(),
                params: HashMap::new(),
                output: None,
                description: None,
            },
        }
    }

    /// Add a parameter.
    pub fn with_param<V: Into<Value>>(mut self, key: &str, value: V) -> Self {
        self.step.params.insert(key.to_string(), value.into());
        self
    }

    /// Add a parameter with template syntax (will be resolved at runtime).
    ///
    /// Templates use Handlebars syntax: `{{ variable.path }}`
    pub fn with_template_param(mut self, key: &str, template: &str) -> Self {
        // Mark as template by wrapping in special structure
        self.step.params.insert(
            key.to_string(),
            serde_json::json!({
                "__template__": template
            }),
        );
        self
    }

    /// Add all parameters from a JSON value.
    pub fn with_params(mut self, params: Value) -> Self {
        if let Value::Object(map) = params {
            for (k, v) in map {
                self.step.params.insert(k, v);
            }
        }
        self
    }

    /// Set the output variable name.
    pub fn output(mut self, name: &str) -> Self {
        self.step.output = Some(name.to_string());
        self
    }

    /// Set a description.
    pub fn description(mut self, desc: &str) -> Self {
        self.step.description = Some(desc.to_string());
        self
    }

    /// Build the step.
    pub fn build(self) -> Step {
        self.step
    }
}

impl From<StepBuilder> for Step {
    fn from(builder: StepBuilder) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_builder() {
        let step = Step::call("gmail", "gmail.inbox")
            .with_param("limit", 10)
            .output("emails")
            .build();

        assert_eq!(step.service, "gmail");
        assert_eq!(step.method, "gmail.inbox");
        assert_eq!(step.params.get("limit"), Some(&Value::from(10)));
        assert_eq!(step.output, Some("emails".to_string()));
    }

    #[test]
    fn test_template_param() {
        let step = Step::call("browser", "browser.open")
            .with_template_param("url", "{{ emails.0.link }}")
            .build();

        let url_param = step.params.get("url").unwrap();
        assert!(url_param.get("__template__").is_some());
    }
}
