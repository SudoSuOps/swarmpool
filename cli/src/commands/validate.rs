//! validate command - Validate snapshot against schema
//!
//! Debug tool for checking snapshots before publishing.
//! Invalid snapshots never leave the box.

use anyhow::{Context, Result};
use colored::Colorize;

use crate::schema::{self, SchemaType};

pub async fn execute(file: String, schema: String) -> Result<()> {
    println!("{}", "Validating Snapshot".cyan().bold());
    println!();

    // Parse schema type
    let schema_type = SchemaType::from_str(&schema)
        .context(format!("Unknown schema type: {}. Use: genesis, job, claim, proof, epoch", schema))?;

    println!("  {} {}", "File:".bright_black(), file);
    println!("  {} {:?}", "Schema:".bright_black(), schema);
    println!();

    // Read and parse file
    let content = std::fs::read_to_string(&file)
        .context("Failed to read file")?;

    let data: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse JSON")?;

    // Validate
    let result = schema::validate_snapshot(&data, schema_type);

    if result.valid {
        println!("{}", "✅ VALID".green().bold());
        println!();
        println!("  {}", "Snapshot passes schema validation.".bright_black());
        println!("  {}", "Safe to publish.".bright_black());
    } else {
        println!("{}", "❌ INVALID".red().bold());
        println!();
        println!("  {}", "Schema validation errors:".yellow());
        for error in &result.errors {
            println!("    {} {}", "•".red(), error);
        }
        println!();
        println!("  {}", "Fix these errors before publishing.".bright_black());
        
        // Return error to set exit code
        anyhow::bail!("Schema validation failed");
    }

    Ok(())
}
