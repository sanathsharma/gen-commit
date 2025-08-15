use crate::client::{AIClient, Client, Result};
use std::collections::HashMap;

pub fn group_files_by_type(modified_files: Vec<String>) -> String {
  let mut groups: HashMap<String, Vec<String>> = HashMap::new();

  // Define file type patterns
  let patterns = [
    // Frontend files
    (
      vec![".tsx", ".jsx", ".css", ".scss", ".html", ".vue"],
      "Frontend",
    ),
    // Backend files
    (
      vec![".rs", ".go", ".js", ".py", ".rb", ".php", ".java"],
      "Backend",
    ),
    // Test files
    (vec!["test.", "spec.", "/tests/", "/test/"], "Tests"),
    // Config files
    (
      vec![".toml", ".json", ".yaml", ".yml", ".config."],
      "Config",
    ),
    // Documentation
    (
      vec![".md", ".txt", "README", "LICENSE", "CHANGELOG"],
      "Docs",
    ),
  ];

  for file in modified_files {
    let mut assigned = false;

    // Check if file matches any pattern
    for (extensions, group) in &patterns {
      if extensions.iter().any(|ext| file.contains(ext)) {
        groups
          .entry(group.to_string())
          .or_default()
          .push(file.clone());
        assigned = true;
        break;
      }
    }

    // If no match found, add to Other group
    if !assigned {
      groups.entry("Other".to_string()).or_default().push(file);
    }
  }

  // Format the output
  let mut result = String::new();
  for (group, files) in groups {
    result.push_str(&format!("- {}: {}\n", group, files.join(", ")));
  }

  result
}

pub fn format_recent_commits(commits: Vec<String>) -> String {
  if commits.is_empty() {
    return "No recent commits found.".to_string();
  }

  let mut result = String::new();
  for commit in commits {
    result.push_str(&format!("- {}\n", commit));
  }

  result
}

pub async fn analyze_changes_with_ai(client: &Client, diff: &str) -> Result<String> {
  // Create system prompt for analysis
  let system_prompt = "You are an expert code analyst. Analyze git diffs and provide concise summaries of changes. \
                        Focus on identifying new functions, modified functions, tests, dependencies, and overall purpose. \
                        Format responses as bullet points. Be brief and specific.".to_string();

  // Create user prompt for the AI to analyze the changes
  let user_prompt = format!(
    "Analyze the following git diff and provide a concise summary of the changes. \
         Focus on identifying: \
         1. New functions/methods added \
         2. Functions/methods modified \
         3. Tests added or modified \
         4. Dependencies changed \
         5. Overall purpose of the changes \
         \
         Format your response as bullet points, one for each category. \
         Be brief and specific. \
         \
         Git diff: \
         {}\n",
    diff
  )
  .to_string();

  // Call the AI model to analyze the changes
  let analysis = match client {
    Client::OpenAI(c) => c.generate_response(system_prompt, user_prompt).await?,
    Client::Anthropic(c) => c.generate_response(system_prompt, user_prompt).await?,
  };

  Ok(analysis)
}
