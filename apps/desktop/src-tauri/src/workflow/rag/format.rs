use super::search::SearchResult;

/// Format search results as a context string with source citations.
/// Optimized for LLM injection.
pub fn format_context_with_citations(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "No relevant context found in the knowledge base.".to_string();
    }

    let mut output = String::from("Relevant context from your knowledge base:\n");

    for result in results {
        output.push_str("\n---\n");
        output.push_str(&format!(
            "[Source: {}, lines {}-{}, score: {:.2}]\n",
            result.source, result.line_start, result.line_end, result.score,
        ));
        output.push_str(&result.text);
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_empty() {
        let result = format_context_with_citations(&[]);
        assert!(result.contains("No relevant context"));
    }

    #[test]
    fn test_format_single_result() {
        let results = vec![SearchResult {
            chunk_id: 0,
            score: 0.92,
            text: "JWT tokens with 15-minute expiry".into(),
            source: "auth-service.md".into(),
            line_start: 23,
            line_end: 45,
        }];
        let output = format_context_with_citations(&results);
        assert!(output.contains("[Source: auth-service.md, lines 23-45, score: 0.92]"));
        assert!(output.contains("JWT tokens"));
    }

    #[test]
    fn test_format_multiple_results() {
        let results = vec![
            SearchResult { chunk_id: 0, score: 0.92, text: "First chunk".into(), source: "a.md".into(), line_start: 1, line_end: 10 },
            SearchResult { chunk_id: 1, score: 0.85, text: "Second chunk".into(), source: "b.md".into(), line_start: 5, line_end: 15 },
        ];
        let output = format_context_with_citations(&results);
        assert!(output.contains("[Source: a.md"));
        assert!(output.contains("[Source: b.md"));
        assert!(output.contains("First chunk"));
        assert!(output.contains("Second chunk"));
        // Should have separators
        assert!(output.matches("---").count() == 2);
    }
}
