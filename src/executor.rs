//! Workflow execution engine.

use crate::{Context, Step, Workflow};
use anyhow::{Context as _, Result};
use serde_json::Value;

/// Result of workflow execution.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Final result (last step's output)
    pub result: Value,

    /// All step results
    pub step_results: Vec<StepResult>,

    /// Final context state
    pub context: Context,

    /// Total execution time in milliseconds
    pub total_ms: f64,
}

/// Result of a single step execution.
#[derive(Debug)]
pub struct StepResult {
    /// Step index (0-based)
    pub index: usize,

    /// Step that was executed
    pub step: Step,

    /// Result of the step
    pub result: Value,

    /// Execution time in milliseconds
    pub duration_ms: f64,
}

/// Execute a workflow.
///
/// This is the main entry point for running workflows.
/// It processes each step sequentially, passing results between them.
///
/// # Arguments
/// * `workflow` - The workflow to execute
///
/// # Returns
/// * `Ok(ExecutionResult)` - All steps completed successfully
/// * `Err(...)` - A step failed (fail-fast behavior)
///
/// # Example
///
/// ```rust,no_run
/// use fgp_workflow::{Workflow, Step, execute};
///
/// let workflow = Workflow::new("example")
///     .add(Step::call("gmail", "gmail.unread").output("unread"))
///     .build();
///
/// let result = execute(&workflow)?;
/// println!("Unread count: {:?}", result.result);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn execute(workflow: &Workflow) -> Result<ExecutionResult> {
    tracing::info!(workflow = %workflow.name, steps = workflow.steps.len(), "Starting workflow");

    let start = std::time::Instant::now();
    let mut ctx = Context::new();
    let mut step_results = Vec::new();

    for (index, step) in workflow.steps.iter().enumerate() {
        let step_start = std::time::Instant::now();

        tracing::debug!(
            step = index,
            service = %step.service,
            method = %step.method,
            "Executing step"
        );

        // Resolve parameters (expand templates)
        let resolved_params = resolve_params(&ctx, &step.params)?;

        // Call the daemon (with auto-start enabled for workflows)
        let response = fgp_daemon::client::call_auto_start(
            &step.service,
            &step.method,
            resolved_params.clone(),
        )
        .with_context(|| format!("Step {} ({}.{}) failed", index, step.service, step.method))?;

        // Check response
        if !response.ok {
            let error = response.error.map(|e| e.message).unwrap_or_default();
            anyhow::bail!(
                "Step {} ({}.{}) returned error: {}",
                index,
                step.service,
                step.method,
                error
            );
        }

        let result = response.result.unwrap_or(Value::Null);
        let step_ms = step_start.elapsed().as_secs_f64() * 1000.0;

        tracing::debug!(step = index, duration_ms = step_ms, "Step completed");

        // Store result
        ctx.push_result(result.clone());

        // Store in named variable if output is specified
        if let Some(ref output_name) = step.output {
            ctx.set(output_name, result.clone());
        }

        step_results.push(StepResult {
            index,
            step: step.clone(),
            result: result.clone(),
            duration_ms: step_ms,
        });
    }

    let total_ms = start.elapsed().as_secs_f64() * 1000.0;

    tracing::info!(
        workflow = %workflow.name,
        total_ms = total_ms,
        "Workflow completed"
    );

    let final_result = ctx.prev().cloned().unwrap_or(Value::Null);

    Ok(ExecutionResult {
        result: final_result,
        step_results,
        context: ctx,
        total_ms,
    })
}

/// Resolve parameters, expanding templates.
fn resolve_params(
    ctx: &Context,
    params: &std::collections::HashMap<String, Value>,
) -> Result<Value> {
    let mut resolved = serde_json::Map::new();

    for (key, value) in params {
        resolved.insert(key.clone(), ctx.resolve(value)?);
    }

    Ok(Value::Object(resolved))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_params_simple() {
        let ctx = Context::new();
        let mut params = std::collections::HashMap::new();
        params.insert("limit".to_string(), Value::from(10));

        let resolved = resolve_params(&ctx, &params).unwrap();

        assert_eq!(resolved.get("limit"), Some(&Value::from(10)));
    }

    #[test]
    fn test_resolve_params_with_template() {
        let mut ctx = Context::new();
        ctx.set("count", Value::from(5));

        let mut params = std::collections::HashMap::new();
        params.insert(
            "message".to_string(),
            serde_json::json!({"__template__": "Found {{ count }} items"}),
        );

        let resolved = resolve_params(&ctx, &params).unwrap();

        assert_eq!(
            resolved.get("message"),
            Some(&Value::String("Found 5 items".to_string()))
        );
    }
}
