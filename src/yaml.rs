//! YAML workflow parser.

use crate::Workflow;
use anyhow::{Context, Result};
use std::path::Path;

/// Parse a workflow from YAML string.
///
/// # Example
///
/// ```rust
/// use fgp_workflow::parse_yaml;
///
/// let yaml = r#"
/// name: my-workflow
/// steps:
///   - service: gmail
///     method: gmail.inbox
///     params:
///       limit: 5
///     output: emails
/// "#;
///
/// let workflow = parse_yaml(yaml).unwrap();
/// assert_eq!(workflow.name, "my-workflow");
/// assert_eq!(workflow.steps.len(), 1);
/// ```
pub fn parse_yaml(yaml: &str) -> Result<Workflow> {
    let workflow: Workflow = serde_yaml::from_str(yaml).context("Failed to parse workflow YAML")?;

    validate(&workflow)?;

    Ok(workflow)
}

/// Load and parse a workflow from a YAML file.
///
/// # Arguments
/// * `path` - Path to the YAML file
///
/// # Example
///
/// ```rust,no_run
/// use fgp_workflow::yaml::load_file;
///
/// let workflow = load_file("workflow.yaml")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn load_file(path: impl AsRef<Path>) -> Result<Workflow> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read workflow file: {}", path.display()))?;

    parse_yaml(&content)
        .with_context(|| format!("Failed to parse workflow file: {}", path.display()))
}

/// Validate a workflow.
fn validate(workflow: &Workflow) -> Result<()> {
    if workflow.name.is_empty() {
        anyhow::bail!("Workflow name cannot be empty");
    }

    if workflow.steps.is_empty() {
        anyhow::bail!("Workflow must have at least one step");
    }

    for (i, step) in workflow.steps.iter().enumerate() {
        if step.service.is_empty() {
            anyhow::bail!("Step {} has empty service name", i);
        }
        if step.method.is_empty() {
            anyhow::bail!("Step {} has empty method name", i);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_workflow() {
        let yaml = r#"
name: test-workflow
description: A test workflow
steps:
  - service: gmail
    method: gmail.inbox
    params:
      limit: 10
    output: emails
"#;

        let workflow = parse_yaml(yaml).unwrap();
        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.description, Some("A test workflow".to_string()));
        assert_eq!(workflow.steps.len(), 1);
        assert_eq!(workflow.steps[0].service, "gmail");
        assert_eq!(workflow.steps[0].method, "gmail.inbox");
    }

    #[test]
    fn test_parse_multi_step_workflow() {
        let yaml = r#"
name: email-to-browser
steps:
  - service: gmail
    method: gmail.search
    params:
      query: "is:unread"
    output: emails
  - service: browser
    method: browser.open
    params:
      url: "{{ emails.0.link }}"
"#;

        let workflow = parse_yaml(yaml).unwrap();
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].output, Some("emails".to_string()));
    }

    #[test]
    fn test_validate_empty_name() {
        let yaml = r#"
name: ""
steps:
  - service: test
    method: test.action
"#;

        let result = parse_yaml(yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("name cannot be empty"));
    }

    #[test]
    fn test_validate_no_steps() {
        let yaml = r#"
name: empty-workflow
steps: []
"#;

        let result = parse_yaml(yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one step"));
    }
}
