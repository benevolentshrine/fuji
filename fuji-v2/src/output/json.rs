use serde::Serialize;
use crate::models::AnalysisResult;

#[derive(Serialize)]
struct JSONOutput {
    summary: JSONSummary,
    files: Vec<JSONFile>,
}

#[derive(Serialize)]
struct JSONSummary {
    files_analyzed: usize,
    files_flagged: usize,
    avg_complexity: f64,
    total_issues: usize,
}

#[derive(Serialize)]
struct JSONFile {
    path: String,
    ai_probability: f64,
    complexity: usize,
    issues: Vec<JSONIssue>,
}

#[derive(Serialize)]
struct JSONIssue {
    line: usize,
    #[serde(rename = "type")]
    issue_type: String,
    severity: String,
}

pub fn write_json(result: &AnalysisResult) -> Result<(), String> {
    let output = JSONOutput {
        summary: JSONSummary {
            files_analyzed: result.summary.files_analyzed,
            files_flagged: result.summary.files_flagged,
            avg_complexity: result.summary.avg_complexity,
            total_issues: result.summary.total_issues,
        },
        files: result.files.iter()
            .filter(|f| !f.issues.is_empty() || f.ai_probability > 30.0)
            .map(|f| JSONFile {
                path: f.path.clone(),
                ai_probability: f.ai_probability / 100.0,
                complexity: f.complexity,
                issues: f.issues.iter().map(|issue| JSONIssue {
                    line: issue.line,
                    issue_type: issue.issue_type.clone(),
                    severity: issue.severity.to_string(),
                }).collect(),
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&output)
        .map_err(|e| format!("JSON error: {}", e))?;
    println!("{}", json);
    Ok(())
}
