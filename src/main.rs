use clap::{Arg, Command};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::process::{Command as ProcessCommand, Stdio};
use std::error::Error;

/// Copies the provided text to the system clipboard.
/// Uses platform-specific commands:
/// - macOS: `pbcopy`
/// - Windows: `clip`
/// - Linux: assumes `xclip` is installed.
fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
    if cfg!(target_os = "macos") {
        let mut process = ProcessCommand::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()?;
        process.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
        process.wait()?;
    } else if cfg!(target_os = "windows") {
        let mut process = ProcessCommand::new("clip")
            .stdin(Stdio::piped())
            .spawn()?;
        process.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
        process.wait()?;
    } else {
        // Assume Linux and that xclip is installed.
        let mut process = ProcessCommand::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()?;
        process.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
        process.wait()?;
    }
    Ok(())
}

/// Reads text from the system clipboard.
/// Uses platform-specific commands:
/// - macOS: `pbpaste`
/// - Windows: PowerShell's `Get-Clipboard`
/// - Linux: assumes `xclip` is installed.
fn read_from_clipboard() -> Result<String, Box<dyn Error>> {
    let output = if cfg!(target_os = "macos") {
        ProcessCommand::new("pbpaste").output()?
    } else if cfg!(target_os = "windows") {
        ProcessCommand::new("powershell")
            .args(&["-Command", "Get-Clipboard"])
            .output()?
    } else {
        // Assume Linux with xclip installed.
        ProcessCommand::new("xclip")
            .args(&["-selection", "clipboard", "-o"])
            .output()?
    };

    if !output.status.success() {
        return Err("Error: Unable to read from clipboard".into());
    }
    let text = String::from_utf8(output.stdout)?;
    Ok(text)
}

/// Extracts variables from the given content.
/// Variables must be in the format {{variable}}, and are treated case-insensitively.
fn extract_variables(content: &str) -> HashSet<String> {
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}").unwrap();
    let mut found_vars = HashSet::new();
    for cap in re.captures_iter(content) {
        found_vars.insert(cap[1].to_lowercase());
    }
    found_vars
}

/// Processes the content by substituting all occurrences of variables with their values.
/// If a substitution for a variable is missing, the variable remains unchanged.
fn process_content(content: &str, substitutions: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let var_name = caps[1].to_lowercase();
        substitutions.get(&var_name).cloned().unwrap_or_else(|| caps[0].to_string())
    })
    .to_string()
}

/// Parses a substitution string of the form key=value.
/// Returns an error if the format is invalid or if the value is missing.
fn parse_substitution(sub: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = sub.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid substitution format '{}'. Use key=value.", sub));
    }
    let key = parts[0].to_lowercase();
    let value = parts[1];
    if value.is_empty() {
        return Err(format!("Missing value for variable '{}'", key));
    }
    Ok((key, value.to_string()))
}

/// Given a set of variables and an optional positional substitution value,
/// returns a substitution map if exactly one variable is found when a positional
/// value is provided. Otherwise, returns an error.
fn get_single_substitution(variables: &HashSet<String>, pos_value: Option<&str>) -> Result<HashMap<String, String>, String> {
    let mut subs = HashMap::new();
    if let Some(val) = pos_value {
        if variables.len() != 1 {
            return Err("Error: More than one variable found in the prompt file. Please use the -s flag to specify values for each variable.".to_string());
        }
        let var = variables.iter().next().unwrap().clone();
        subs.insert(var, val.to_string());
    }
    Ok(subs)
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("cbsub")
        .version("1.0")
        .about("Substitutes variables in a prompt file and copies the result to the clipboard")
        .arg(Arg::new("file")
             .help("The prompt file to process")
             .required(true)
             .index(1))
        // The positional argument for a single substitution.
        .arg(Arg::new("value")
             .help("Substitution value for a single variable (when exactly one exists)")
             .required(false)
             .index(2))
        // New clipboard flag: if set, use clipboard contents as substitution.
        .arg(Arg::new("clipboard")
             .short('c')
             .long("clipboard")
             .help("Use clipboard content as the substitution value (for a file with a single variable)")
             .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("substitution")
             .short('s')
             .long("substitution")
             .help("Substitute a variable in key=value format (can be used multiple times)")
             .action(clap::ArgAction::Append))
        .arg(Arg::new("preview")
             .short('p')
             .help("Preview the processed text without copying to the clipboard")
             .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("list")
             .short('l')
             .help("List the variables found in the prompt file")
             .action(clap::ArgAction::SetTrue))
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let content = fs::read_to_string(file_path)
        .map_err(|_| format!("Error: Could not read file '{}'", file_path))?;

    // Extract variables from the file.
    let variables = extract_variables(&content);

    // If -l flag is provided, list the variables and exit.
    if matches.get_flag("list") {
        if variables.is_empty() {
            println!("No variables found in the prompt file.");
        } else {
            println!("Found variables:");
            for var in &variables {
                println!(" - {{{}}}", var);
            }
        }
        return Ok(());
    }

    let mut substitutions: HashMap<String, String> = HashMap::new();

    // Check for clipboard flag.
    if matches.get_flag("clipboard") {
        if matches.get_one::<String>("value").is_some() || matches.contains_id("substitution") {
            eprintln!("Error: Cannot use the --clipboard flag with other substitution methods.");
            return Err("Ambiguous substitution".into());
        }
        let clip_value = read_from_clipboard()?;
        let single_subs = get_single_substitution(&variables, Some(&clip_value))?;
        substitutions.extend(single_subs);
    }
    // Build substitutions from -s flags.
    else if let Some(subs) = matches.get_many::<String>("substitution") {
        for sub in subs {
            let (key, value) = parse_substitution(sub)?;
            substitutions.insert(key, value);
        }
    }
    // Handle positional substitution.
    else if let Some(pos_value) = matches.get_one::<String>("value") {
        if !substitutions.is_empty() {
            eprintln!("Error: Cannot use positional substitution and -s flag together.");
            return Err("Ambiguous substitution".into());
        }
        if variables.is_empty() {
            eprintln!("Error: No variables found in the prompt file to substitute.");
            return Err("No variables found".into());
        }
        let single_subs = get_single_substitution(&variables, Some(pos_value))?;
        substitutions.extend(single_subs);
    }

    // If no substitution is provided but variables exist, list them and exit.
    if substitutions.is_empty() {
        if !variables.is_empty() {
            println!("Found variables:");
            for var in &variables {
                println!(" - {}", var);
            }
            return Ok(());
        } else {
            // No variables found and no substitutions provided; copy file content as-is.
            copy_to_clipboard(&content)?;
            println!("File content copied to clipboard.");
            return Ok(());
        }
    }

    // Perform substitutions.
    let result = process_content(&content, &substitutions);

    // If the preview flag (-p) is set, display the result instead of copying.
    if matches.get_flag("preview") {
        println!("{}", result);
    } else {
        copy_to_clipboard(&result)?;
        println!("Processed content copied to clipboard.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_extract_variables_empty() {
        let content = "This is a test with no variables.";
        let vars = extract_variables(content);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_extract_variables() {
        let content = "Hello {{name}}, your code is {{code}}. Again, hi {{name}}!";
        let vars = extract_variables(content);
        assert_eq!(vars.len(), 2);
        assert!(vars.contains("name"));
        assert!(vars.contains("code"));
    }

    #[test]
    fn test_process_content_complete() {
        let content = "Hello {{name}}, your code is {{code}}.";
        let mut subs = HashMap::new();
        subs.insert("name".to_string(), "Alice".to_string());
        subs.insert("code".to_string(), "9876".to_string());
        let result = process_content(content, &subs);
        assert_eq!(result, "Hello Alice, your code is 9876.");
    }

    #[test]
    fn test_process_content_partial() {
        let content = "Hello {{name}}, your code is {{code}}.";
        let mut subs = HashMap::new();
        subs.insert("name".to_string(), "Alice".to_string());
        // No substitution for "code": it should remain unchanged.
        let result = process_content(content, &subs);
        assert_eq!(result, "Hello Alice, your code is {{code}}.");
    }

    #[test]
    fn test_case_insensitivity() {
        let content = "Hello {{Name}}, your code is {{CoDe}}.";
        let mut subs = HashMap::new();
        subs.insert("name".to_string(), "Alice".to_string());
        subs.insert("code".to_string(), "9876".to_string());
        let result = process_content(content, &subs);
        assert_eq!(result, "Hello Alice, your code is 9876.");
    }

    #[test]
    fn test_parse_substitution_valid() {
        let sub = "code=1234";
        let result = parse_substitution(sub);
        assert!(result.is_ok());
        let (k, v) = result.unwrap();
        assert_eq!(k, "code");
        assert_eq!(v, "1234");
    }

    #[test]
    fn test_parse_substitution_missing_value() {
        let sub = "name=";
        let result = parse_substitution(sub);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing value for variable 'name'");
    }

    #[test]
    fn test_parse_substitution_invalid_format() {
        let sub = "invalid";
        let result = parse_substitution(sub);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_single_substitution_success() {
        let content = "Hello {{name}}!";
        let vars = extract_variables(content);
        let result = get_single_substitution(&vars, Some("Alice"));
        assert!(result.is_ok());
        let subs = result.unwrap();
        assert_eq!(subs.get("name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_get_single_substitution_failure() {
        let content = "Hello {{name}} and {{code}}!";
        let vars = extract_variables(content);
        let result = get_single_substitution(&vars, Some("Alice"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error: More than one variable found in the prompt file. Please use the -s flag to specify values for each variable.");
    }
}
