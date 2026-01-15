//! # fgp-workflow
//!
//! Workflow composition for FGP daemon services.
//!
//! Chain multiple daemon calls together, passing data between steps.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use fgp_workflow::{Workflow, Step};
//!
//! let workflow = Workflow::new("my-workflow")
//!     .add(Step::call("gmail", "gmail.inbox")
//!         .with_param("limit", 5)
//!         .output("emails"))
//!     .add(Step::call("browser", "browser.open")
//!         .with_template_param("url", "{{ emails.0.url }}"));
//!
//! let result = workflow.run()?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## YAML Definition
//!
//! ```yaml
//! name: my-workflow
//! steps:
//!   - service: gmail
//!     method: gmail.inbox
//!     params:
//!       limit: 5
//!     output: emails
//!   - service: browser
//!     method: browser.open
//!     params:
//!       url: "{{ emails.0.url }}"
//! ```

mod context;
mod executor;
mod step;
mod workflow;
pub mod yaml;

pub use context::Context;
pub use executor::{execute, ExecutionResult};
pub use step::{Step, StepBuilder};
pub use workflow::{Workflow, WorkflowBuilder};
pub use yaml::parse_yaml;

/// Re-export common types
pub use serde_json::Value;
