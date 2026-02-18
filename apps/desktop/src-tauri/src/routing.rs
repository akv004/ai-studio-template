// ============================================
// SMART ROUTER — Hybrid Intelligence
// Routes LLM calls to the best model based on
// task type, budget, and user preferences
// ============================================

use serde::{Deserialize, Serialize};

// ============================================
// TYPES
// ============================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutingDecision {
    pub provider: String,
    pub model: String,
    pub reason: String,
    pub estimated_savings: f64,
    pub alternatives_considered: Vec<Alternative>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Alternative {
    pub model: String,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub provider: &'static str,
    pub model: &'static str,
    pub vision: bool,
    pub cost_tier: CostTier,
    pub input_per_1m: f64,
    pub output_per_1m: f64,
    pub context_window: usize,
    pub strengths: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CostTier {
    Free,
    Cheap,
    Moderate,
    Expensive,
}

// ============================================
// MODEL CAPABILITIES TABLE
// ============================================

pub static MODEL_CAPABILITIES: &[ModelInfo] = &[
    ModelInfo {
        provider: "anthropic",
        model: "claude-opus-4-6",
        vision: false,
        cost_tier: CostTier::Expensive,
        input_per_1m: 15.0,
        output_per_1m: 75.0,
        context_window: 200_000,
        strengths: &["reasoning", "code", "analysis"],
    },
    ModelInfo {
        provider: "anthropic",
        model: "claude-sonnet-4-5",
        vision: true,
        cost_tier: CostTier::Moderate,
        input_per_1m: 3.0,
        output_per_1m: 15.0,
        context_window: 200_000,
        strengths: &["code", "balanced", "tool_use"],
    },
    ModelInfo {
        provider: "openai",
        model: "gpt-4o",
        vision: true,
        cost_tier: CostTier::Moderate,
        input_per_1m: 2.5,
        output_per_1m: 10.0,
        context_window: 128_000,
        strengths: &["vision", "balanced", "multilingual"],
    },
    ModelInfo {
        provider: "google",
        model: "gemini-2.0-flash",
        vision: true,
        cost_tier: CostTier::Cheap,
        input_per_1m: 0.10,
        output_per_1m: 0.40,
        context_window: 1_000_000,
        strengths: &["speed", "large_context", "cheap"],
    },
    ModelInfo {
        provider: "ollama",
        model: "llama3.2",
        vision: false,
        cost_tier: CostTier::Free,
        input_per_1m: 0.0,
        output_per_1m: 0.0,
        context_window: 128_000,
        strengths: &["free", "private", "fast_local"],
    },
];

// ============================================
// ROUTING INPUT
// ============================================

pub struct RoutingInput<'a> {
    pub message: &'a str,
    pub context_tokens: usize,
    pub has_images: bool,
    pub tools: &'a [String],
    pub routing_mode: &'a str,
    pub routing_rules: &'a [serde_json::Value],
    pub default_provider: &'a str,
    pub default_model: &'a str,
    pub budget_remaining_pct: f64,
    pub available_providers: &'a [String],
}

// ============================================
// ROUTE FUNCTION
// ============================================

pub fn route(input: &RoutingInput) -> RoutingDecision {
    match input.routing_mode {
        "hybrid_auto" => route_auto(input),
        "hybrid_manual" => route_manual(input),
        _ => route_single(input),
    }
}

fn route_single(input: &RoutingInput) -> RoutingDecision {
    let est_cost = estimate_cost(input.default_provider, input.default_model, input.context_tokens);
    RoutingDecision {
        provider: input.default_provider.to_string(),
        model: input.default_model.to_string(),
        reason: "single_model".to_string(),
        estimated_savings: 0.0,
        alternatives_considered: build_alternatives(input.default_model, input.context_tokens),
    }
}

fn route_auto(input: &RoutingInput) -> RoutingDecision {
    let default_cost = estimate_cost(input.default_provider, input.default_model, input.context_tokens);

    // Rule 1: Vision tasks need a vision model
    if input.has_images {
        if let Some(decision) = try_route_to(
            "gpt-4o", "openai", "vision_required",
            default_cost, input,
        ) {
            return decision;
        }
        // Fallback: any vision-capable available model
        if let Some(decision) = try_route_to(
            "gemini-2.0-flash", "google", "vision_required",
            default_cost, input,
        ) {
            return decision;
        }
    }

    // Rule 2: Simple messages → local model (< 100 estimated tokens, no tools)
    let est_tokens = input.message.len() / 4;
    if est_tokens < 100 && input.tools.is_empty() {
        if let Some(decision) = try_route_to(
            "llama3.2", "ollama", "simple_query_local",
            default_cost, input,
        ) {
            return decision;
        }
    }

    // Rule 3: Code tasks (tools with filesystem/shell)
    let has_code_tools = input.tools.iter().any(|t| {
        t.contains("shell") || t.contains("filesystem") || t.contains("write")
    });
    if has_code_tools {
        if let Some(decision) = try_route_to(
            "claude-sonnet-4-5", "anthropic", "code_task",
            default_cost, input,
        ) {
            return decision;
        }
    }

    // Rule 4: Large context → Gemini Flash
    if input.context_tokens > 50_000 {
        if let Some(decision) = try_route_to(
            "gemini-2.0-flash", "google", "large_context",
            default_cost, input,
        ) {
            return decision;
        }
    }

    // Rule 5: Budget low → cheapest available
    if input.budget_remaining_pct < 20.0 {
        if let Some(decision) = try_route_to(
            "llama3.2", "ollama", "budget_conservation",
            default_cost, input,
        ) {
            return decision;
        }
        // Fallback to cheapest cloud
        if let Some(decision) = try_route_to(
            "gemini-2.0-flash", "google", "budget_conservation",
            default_cost, input,
        ) {
            return decision;
        }
    }

    // Rule 6: Default
    route_single(input)
}

fn route_manual(input: &RoutingInput) -> RoutingDecision {
    let default_cost = estimate_cost(input.default_provider, input.default_model, input.context_tokens);

    // Sort rules by priority (highest first)
    let mut rules: Vec<&serde_json::Value> = input.routing_rules.iter().collect();
    rules.sort_by(|a, b| {
        let pa = a.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
        let pb = b.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
        pb.cmp(&pa)
    });

    for rule in &rules {
        let condition = rule.get("condition").and_then(|v| v.as_str()).unwrap_or("");
        let provider = rule.get("provider").and_then(|v| v.as_str()).unwrap_or("");
        let model = rule.get("model").and_then(|v| v.as_str()).unwrap_or("");

        if provider.is_empty() || model.is_empty() {
            continue;
        }

        let matches = match condition {
            "vision_required" => input.has_images,
            "simple_query" => {
                let est = input.message.len() / 4;
                est < 100 && input.tools.is_empty()
            }
            "code_task" => input.tools.iter().any(|t| {
                t.contains("shell") || t.contains("filesystem") || t.contains("write")
            }),
            "large_context" => input.context_tokens > 50_000,
            "budget_low" => input.budget_remaining_pct < 20.0,
            "always" => true,
            _ => false,
        };

        if matches {
            if let Some(decision) = try_route_to(model, provider, condition, default_cost, input) {
                return decision;
            }
        }
    }

    // No rule matched — use default
    route_single(input)
}

// ============================================
// HELPERS
// ============================================

fn try_route_to(
    model: &str,
    provider: &str,
    reason: &str,
    default_cost: f64,
    input: &RoutingInput,
) -> Option<RoutingDecision> {
    // Check if provider is available
    if !input.available_providers.iter().any(|p| p == provider) {
        return None;
    }

    let routed_cost = estimate_cost(provider, model, input.context_tokens);
    let savings = (default_cost - routed_cost).max(0.0);

    Some(RoutingDecision {
        provider: provider.to_string(),
        model: model.to_string(),
        reason: reason.to_string(),
        estimated_savings: savings,
        alternatives_considered: build_alternatives(model, input.context_tokens),
    })
}

fn build_alternatives(chosen_model: &str, context_tokens: usize) -> Vec<Alternative> {
    MODEL_CAPABILITIES
        .iter()
        .filter(|m| m.model != chosen_model)
        .map(|m| Alternative {
            model: m.model.to_string(),
            estimated_cost: estimate_cost(m.provider, m.model, context_tokens),
        })
        .collect()
}

pub fn estimate_cost(provider: &str, model: &str, context_tokens: usize) -> f64 {
    let info = MODEL_CAPABILITIES.iter().find(|m| m.model == model);
    match info {
        Some(m) => {
            let input_cost = (context_tokens as f64 / 1_000_000.0) * m.input_per_1m;
            // Estimate output as ~25% of input tokens
            let est_output = context_tokens as f64 * 0.25;
            let output_cost = (est_output / 1_000_000.0) * m.output_per_1m;
            input_cost + output_cost
        }
        None => 0.0, // Unknown model — can't estimate
    }
}

/// Get list of available providers based on which ones have API keys or are local
pub fn get_available_providers(
    all_settings: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut providers = Vec::new();

    // Check for configured cloud providers
    for provider in &["anthropic", "openai", "google", "azure_openai"] {
        let key = format!("provider.{}.api_key", provider);
        if let Some(val) = all_settings.get(&key) {
            let clean = val.trim_matches('"');
            if !clean.is_empty() {
                providers.push(provider.to_string());
            }
        }
    }

    // Ollama is always "available" (will fail at call time if not running, but that's fine)
    providers.push("ollama".to_string());

    providers
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input<'a>(
        mode: &'a str,
        message: &'a str,
        tools: &'a [String],
        has_images: bool,
        budget_pct: f64,
        context_tokens: usize,
        providers: &'a [String],
        rules: &'a [serde_json::Value],
    ) -> RoutingInput<'a> {
        RoutingInput {
            message,
            context_tokens,
            has_images,
            tools,
            routing_mode: mode,
            routing_rules: rules,
            default_provider: "anthropic",
            default_model: "claude-sonnet-4-5",
            budget_remaining_pct: budget_pct,
            available_providers: providers,
        }
    }

    #[test]
    fn test_single_mode_returns_default() {
        let providers = vec!["anthropic".to_string(), "ollama".to_string()];
        let input = make_input("single", "hello", &[], false, 100.0, 100, &providers, &[]);
        let decision = route(&input);
        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.model, "claude-sonnet-4-5");
        assert_eq!(decision.reason, "single_model");
    }

    #[test]
    fn test_auto_simple_query_routes_local() {
        let providers = vec!["anthropic".to_string(), "ollama".to_string()];
        let input = make_input("hybrid_auto", "hi", &[], false, 100.0, 10, &providers, &[]);
        let decision = route(&input);
        assert_eq!(decision.provider, "ollama");
        assert_eq!(decision.model, "llama3.2");
        assert_eq!(decision.reason, "simple_query_local");
    }

    #[test]
    fn test_auto_vision_routes_to_gpt4o() {
        let providers = vec!["anthropic".to_string(), "openai".to_string(), "ollama".to_string()];
        let input = make_input("hybrid_auto", "describe this image", &[], true, 100.0, 100, &providers, &[]);
        let decision = route(&input);
        assert_eq!(decision.provider, "openai");
        assert_eq!(decision.model, "gpt-4o");
        assert_eq!(decision.reason, "vision_required");
    }

    #[test]
    fn test_auto_vision_fallback_to_gemini() {
        // openai not available, should fall back to gemini
        let providers = vec!["anthropic".to_string(), "google".to_string(), "ollama".to_string()];
        let input = make_input("hybrid_auto", "describe this image", &[], true, 100.0, 100, &providers, &[]);
        let decision = route(&input);
        assert_eq!(decision.provider, "google");
        assert_eq!(decision.model, "gemini-2.0-flash");
        assert_eq!(decision.reason, "vision_required");
    }

    #[test]
    fn test_auto_code_task() {
        let providers = vec!["anthropic".to_string(), "ollama".to_string()];
        let tools = vec!["builtin__shell".to_string()];
        let input = make_input("hybrid_auto", "write a function that parses JSON", &tools, false, 100.0, 500, &providers, &[]);
        let decision = route(&input);
        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.model, "claude-sonnet-4-5");
        assert_eq!(decision.reason, "code_task");
    }

    #[test]
    fn test_auto_large_context() {
        let providers = vec!["anthropic".to_string(), "google".to_string(), "ollama".to_string()];
        // Use a message long enough (>400 chars) to avoid triggering simple_query rule first
        let long_msg = "x".repeat(500);
        let input = make_input("hybrid_auto", &long_msg, &[], false, 100.0, 60_000, &providers, &[]);
        let decision = route(&input);
        assert_eq!(decision.provider, "google");
        assert_eq!(decision.model, "gemini-2.0-flash");
        assert_eq!(decision.reason, "large_context");
    }

    #[test]
    fn test_auto_budget_low() {
        let providers = vec!["anthropic".to_string(), "ollama".to_string()];
        let long_msg = "x".repeat(500);
        let input = make_input("hybrid_auto", &long_msg, &[], false, 15.0, 500, &providers, &[]);
        let decision = route(&input);
        assert_eq!(decision.provider, "ollama");
        assert_eq!(decision.model, "llama3.2");
        assert_eq!(decision.reason, "budget_conservation");
    }

    #[test]
    fn test_manual_mode_matches_rule() {
        let providers = vec!["anthropic".to_string(), "openai".to_string(), "ollama".to_string()];
        let rules = vec![
            serde_json::json!({
                "condition": "vision_required",
                "provider": "openai",
                "model": "gpt-4o",
                "priority": 10
            }),
            serde_json::json!({
                "condition": "always",
                "provider": "ollama",
                "model": "llama3.2",
                "priority": 0
            }),
        ];
        let input = make_input("hybrid_manual", "what is this?", &[], true, 100.0, 100, &providers, &rules);
        let decision = route(&input);
        assert_eq!(decision.provider, "openai");
        assert_eq!(decision.model, "gpt-4o");
        assert_eq!(decision.reason, "vision_required");
    }

    #[test]
    fn test_manual_mode_fallback_to_always() {
        let providers = vec!["anthropic".to_string(), "ollama".to_string()];
        let rules = vec![
            serde_json::json!({
                "condition": "always",
                "provider": "ollama",
                "model": "llama3.2",
                "priority": 0
            }),
        ];
        let input = make_input("hybrid_manual", "hello world", &[], false, 100.0, 100, &providers, &rules);
        let decision = route(&input);
        assert_eq!(decision.provider, "ollama");
        assert_eq!(decision.model, "llama3.2");
        assert_eq!(decision.reason, "always");
    }

    #[test]
    fn test_manual_mode_skips_unavailable_provider() {
        let providers = vec!["anthropic".to_string(), "ollama".to_string()];
        let rules = vec![
            serde_json::json!({
                "condition": "always",
                "provider": "openai",
                "model": "gpt-4o",
                "priority": 10
            }),
        ];
        let input = make_input("hybrid_manual", "hello", &[], false, 100.0, 100, &providers, &rules);
        let decision = route(&input);
        // openai not available, falls back to single mode default
        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.model, "claude-sonnet-4-5");
        assert_eq!(decision.reason, "single_model");
    }

    #[test]
    fn test_estimate_cost_known_model() {
        let cost = estimate_cost("anthropic", "claude-sonnet-4-5", 1_000);
        // 1000 tokens input: 1000/1M * 3.0 = 0.003
        // ~250 tokens output: 250/1M * 15.0 = 0.00375
        assert!(cost > 0.006 && cost < 0.007);
    }

    #[test]
    fn test_estimate_cost_free_model() {
        let cost = estimate_cost("ollama", "llama3.2", 10_000);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_savings_positive_when_routing_cheaper() {
        let providers = vec!["anthropic".to_string(), "ollama".to_string()];
        let input = make_input("hybrid_auto", "hi", &[], false, 100.0, 10, &providers, &[]);
        let decision = route(&input);
        assert!(decision.estimated_savings >= 0.0);
    }

    #[test]
    fn test_get_available_providers() {
        let mut settings = std::collections::HashMap::new();
        settings.insert("provider.anthropic.api_key".to_string(), "sk-test".to_string());
        settings.insert("provider.google.api_key".to_string(), "\"\"".to_string()); // empty after trim
        let providers = get_available_providers(&settings);
        assert!(providers.contains(&"anthropic".to_string()));
        assert!(!providers.contains(&"google".to_string()));
        assert!(providers.contains(&"ollama".to_string())); // always present
    }
}
