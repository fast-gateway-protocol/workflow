//! Workflow definition and builder.

use crate::step::{Step, StepBuilder};
use serde::{Deserialize, Serialize};

/// A workflow consisting of multiple steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow name
    pub name: String,

    /// Description of what this workflow does
    #[serde(default)]
    pub description: Option<String>,

    /// Steps to execute
    pub steps: Vec<Step>,
}

impl Workflow {
    /// Create a new workflow with a name.
    pub fn new(name: &str) -> WorkflowBuilder {
        WorkflowBuilder::new(name)
    }

    /// Create an empty workflow.
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            steps: Vec::new(),
        }
    }

    /// Execute this workflow.
    pub fn run(&self) -> anyhow::Result<crate::ExecutionResult> {
        crate::execute(self)
    }
}

/// Builder for creating workflows.
#[derive(Debug, Clone)]
pub struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    /// Create a new workflow builder.
    pub fn new(name: &str) -> Self {
        Self {
            workflow: Workflow {
                name: name.to_string(),
                description: None,
                steps: Vec::new(),
            },
        }
    }

    /// Set the workflow description.
    pub fn description(mut self, desc: &str) -> Self {
        self.workflow.description = Some(desc.to_string());
        self
    }

    /// Add a step to the workflow.
    pub fn add<S: Into<Step>>(mut self, step: S) -> Self {
        self.workflow.steps.push(step.into());
        self
    }

    /// Add a step builder (convenience).
    pub fn step(self, step: StepBuilder) -> Self {
        self.add(step.build())
    }

    /// Build the workflow.
    pub fn build(self) -> Workflow {
        self.workflow
    }

    /// Execute the workflow.
    pub fn run(self) -> anyhow::Result<crate::ExecutionResult> {
        self.build().run()
    }
}

impl From<WorkflowBuilder> for Workflow {
    fn from(builder: WorkflowBuilder) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder() {
        let workflow = Workflow::new("test")
            .description("Test workflow")
            .add(
                Step::call("gmail", "gmail.inbox")
                    .with_param("limit", 5)
                    .output("emails")
                    .build(),
            )
            .add(
                Step::call("browser", "browser.open")
                    .with_template_param("url", "{{ emails.0.url }}")
                    .build(),
            )
            .build();

        assert_eq!(workflow.name, "test");
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].service, "gmail");
        assert_eq!(workflow.steps[1].service, "browser");
    }
}
