// WebSearch tool: search the web using Google Custom Search, Brave Search,
// or a DuckDuckGo instant-answer fallback.
//
// Mirrors the TypeScript WebSearch tool behaviour:
// - Accepts a query string
// - Returns a list of results with title, url, and snippet
// - Falls back to DuckDuckGo if no search API key is configured

use crate::{PermissionLevel, Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use claurst_core::WebSearchConfig;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::debug;

pub struct WebSearchTool;

#[derive(Debug, Deserialize)]
struct WebSearchInput {
    query: String,
    #[serde(default = "default_num_results")]
    num_results: usize,
}

fn default_num_results() -> usize {
    5
}

#[derive(Clone, Copy)]
enum SearchProvider {
    Google,
    Brave,
    DuckDuckGo,
}

struct SearchBackend {
    provider: SearchProvider,
    google_api_key: Option<String>,
    google_cx: Option<String>,
    provider_api_key: Option<String>,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        claurst_core::constants::TOOL_NAME_WEB_SEARCH
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns a list of relevant web pages with \
         titles, URLs, and snippets. Use this when you need current information \
         not available in your training data, or when searching for documentation, \
         examples, or news."
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "num_results": {
                    "type": "number",
                    "description": "Number of results to return (default: 5, max: 10)"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> ToolResult {
        let params: WebSearchInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid input: {}", e)),
        };

        let num_results = params.num_results.min(10).max(1);
        debug!(query = %params.query, num_results, "Web search");

        let backend = resolve_backend(&ctx.config.web_search);

        match backend.provider {
            SearchProvider::Google => match (backend.google_api_key, backend.google_cx) {
                (Some(api_key), Some(cx)) => search_google(&params.query, num_results, &api_key, &cx).await,
                _ => ToolResult::error(
                    "Google web search is configured without both api key and cx. \
                     Set config.webSearch.googleApiKey and config.webSearch.googleCx, \
                     or set GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_CX."
                ),
            },
            SearchProvider::Brave => match backend.provider_api_key {
                Some(api_key) => search_brave(&params.query, num_results, &api_key).await,
                None => ToolResult::error(
                    "Brave web search is configured without an API key. \
                     Set config.webSearch.apiKey or BRAVE_SEARCH_API_KEY."
                ),
            },
            SearchProvider::DuckDuckGo => search_duckduckgo(&params.query, num_results).await,
        }
    }
}

fn resolve_backend(config: &WebSearchConfig) -> SearchBackend {
    let provider_api_key = first_non_empty(
        config.api_key.clone(),
        std::env::var("GOOGLE_SEARCH_API_KEY").ok(),
    )
    .or_else(|| std::env::var("BRAVE_SEARCH_API_KEY").ok().filter(|value| !value.trim().is_empty()));
    let engine_id = first_non_empty(
        config.engine_id.clone(),
        std::env::var("GOOGLE_SEARCH_CX").ok(),
    );

    let normalized_provider = config.provider.as_deref().map(normalize_provider_name);
    let provider = match normalized_provider.as_deref() {
        Some("google") => SearchProvider::Google,
        Some("brave") => SearchProvider::Brave,
        Some("duckduckgo") | Some("duckduckgo-instant") | Some("ddg") => SearchProvider::DuckDuckGo,
        Some(_) | None => {
            if provider_api_key.is_some() && engine_id.is_some() {
                SearchProvider::Google
            } else if provider_api_key.is_some() {
                SearchProvider::Brave
            } else {
                SearchProvider::DuckDuckGo
            }
        }
    };

    SearchBackend {
        provider,
        google_api_key: provider_api_key.clone(),
        google_cx: engine_id,
        provider_api_key,
    }
}

fn first_non_empty(primary: Option<String>, fallback: Option<String>) -> Option<String> {
    primary
        .filter(|value| !value.trim().is_empty())
        .or_else(|| fallback.filter(|value| !value.trim().is_empty()))
}

fn normalize_provider_name(provider: &str) -> String {
    match provider.trim().to_ascii_lowercase().as_str() {
        "googlecustomsearch" | "google_custom_search" | "programmablesearch" | "programmable_search" => "google".to_string(),
        "duckduckgoinstant" | "duckduckgo_instant" => "duckduckgo".to_string(),
        other => other.to_string(),
    }
}

/// Search using the Google Custom Search JSON API.
async fn search_google(query: &str, num_results: usize, api_key: &str, cx: &str) -> ToolResult {
    let client = reqwest::Client::new();
    let url = format!(
        "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&num={}",
        urlencoding_simple(api_key),
        urlencoding_simple(cx),
        urlencoding_simple(query),
        num_results
    );

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => return ToolResult::error(format!("Search request failed: {}", e)),
    };

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        return ToolResult::error(format!("Google Search API returned status {}", status));
    }

    let data: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return ToolResult::error(format!("Failed to parse response: {}", e)),
    };

    let results = format_google_results(&data, num_results);
    ToolResult::success(results)
}

fn format_google_results(data: &Value, max: usize) -> String {
    let mut output = String::new();

    if let Some(items) = data.get("items").and_then(|items| items.as_array()) {
        for (i, item) in items.iter().take(max).enumerate() {
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("(No title)");
            let url = item.get("link").and_then(|u| u.as_str()).unwrap_or("");
            let snippet = item.get("snippet").and_then(|s| s.as_str()).unwrap_or("");

            output.push_str(&format!("{}. **{}**\n   URL: {}\n   {}\n\n", i + 1, title, url, snippet));
        }
    }

    if output.is_empty() {
        "No results found.".to_string()
    } else {
        output
    }
}

/// Search using the Brave Search API.
async fn search_brave(query: &str, num_results: usize, api_key: &str) -> ToolResult {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
        urlencoding_simple(query),
        num_results
    );

    let resp = match client
        .get(&url)
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip")
        .header("X-Subscription-Token", api_key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return ToolResult::error(format!("Search request failed: {}", e)),
    };

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        return ToolResult::error(format!("Brave Search API returned status {}", status));
    }

    let data: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return ToolResult::error(format!("Failed to parse response: {}", e)),
    };

    let results = format_brave_results(&data, num_results);
    ToolResult::success(results)
}

fn format_brave_results(data: &Value, max: usize) -> String {
    let mut output = String::new();
    let web_results = data
        .get("web")
        .and_then(|w| w.get("results"))
        .and_then(|r| r.as_array());

    if let Some(items) = web_results {
        for (i, item) in items.iter().take(max).enumerate() {
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("(No title)");
            let url = item.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let snippet = item.get("description").and_then(|s| s.as_str()).unwrap_or("");

            output.push_str(&format!("{}. **{}**\n   URL: {}\n   {}\n\n", i + 1, title, url, snippet));
        }
    }

    if output.is_empty() {
        "No results found.".to_string()
    } else {
        output
    }
}

/// Fallback: DuckDuckGo Instant Answer API.
/// Note: this doesn't return full search results, only instant answers.
async fn search_duckduckgo(query: &str, num_results: usize) -> ToolResult {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding_simple(query)
    );

    let resp = match client
        .get(&url)
        .header("User-Agent", "Claurst/1.0")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return ToolResult::error(format!("Search request failed: {}", e)),
    };

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        return ToolResult::error(format!("DuckDuckGo API returned status {}", status));
    }

    let data: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return ToolResult::error(format!("Failed to parse response: {}", e)),
    };

    let output = format_ddg_results(&data, num_results);
    ToolResult::success(output)
}

fn format_ddg_results(data: &Value, max: usize) -> String {
    let mut output = String::new();
    let mut count = 0;

    // Abstract (main answer)
    if let Some(abstract_text) = data.get("Abstract").and_then(|a| a.as_str()) {
        if !abstract_text.is_empty() {
            let source = data.get("AbstractSource").and_then(|s| s.as_str()).unwrap_or("");
            let url = data.get("AbstractURL").and_then(|u| u.as_str()).unwrap_or("");
            output.push_str(&format!("**{}**\n{}\nURL: {}\n\n", source, abstract_text, url));
            count += 1;
        }
    }

    // Related topics
    if let Some(topics) = data.get("RelatedTopics").and_then(|t| t.as_array()) {
        for topic in topics.iter().take(max.saturating_sub(count)) {
            if let Some(text) = topic.get("Text").and_then(|t| t.as_str()) {
                if !text.is_empty() {
                    let url = topic.get("FirstURL").and_then(|u| u.as_str()).unwrap_or("");
                    output.push_str(&format!("- {}\n  {}\n\n", text, url));
                }
            }
        }
    }

    if output.is_empty() {
        format!(
            "No instant answer found for '{}'. Configure config.webSearch.googleApiKey + \
             config.webSearch.googleCx, or set GOOGLE_SEARCH_API_KEY + GOOGLE_SEARCH_CX, \
             for full web search.",
            data.get("QuerySearchQuery")
                .and_then(|q| q.as_str())
                .unwrap_or("your query")
        )
    } else {
        output
    }
}

/// Minimal percent-encoding for URL query parameters.
fn urlencoding_simple(s: &str) -> String {
    let mut encoded = String::new();
    for ch in s.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                encoded.push(ch);
            }
            ' ' => encoded.push('+'),
            _ => {
                for byte in ch.to_string().as_bytes() {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_backend_prefers_google_config_when_complete() {
        let backend = resolve_backend(&WebSearchConfig {
            provider: None,
            api_key: Some("g-key".to_string()),
            engine_id: Some("cx-id".to_string()),
        });

        assert!(matches!(backend.provider, SearchProvider::Google));
    }

    #[test]
    fn resolve_backend_falls_back_to_brave_when_google_is_incomplete() {
        let backend = resolve_backend(&WebSearchConfig {
            provider: None,
            api_key: Some("b-key".to_string()),
            engine_id: None,
        });

        assert!(matches!(backend.provider, SearchProvider::Brave));
    }

    #[test]
    fn format_google_results_uses_items_array() {
        let data = json!({
            "items": [
                {
                    "title": "Example",
                    "link": "https://example.com",
                    "snippet": "Example snippet"
                }
            ]
        });

        let rendered = format_google_results(&data, 5);

        assert!(rendered.contains("Example"));
        assert!(rendered.contains("https://example.com"));
        assert!(rendered.contains("Example snippet"));
    }
}
