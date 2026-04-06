use serde_json::{Map, Value};

pub(crate) fn merge_openai_compatible_options(body: &mut Value, provider_options: &Value) {
    let Some(options_obj) = provider_options.as_object() else {
        return;
    };

    for (key, value) in options_obj {
        match key.as_str() {
            "reasoningEffort" => body["reasoning_effort"] = value.clone(),
            "textVerbosity" => body["verbosity"] = value.clone(),
            _ => body[key] = value.clone(),
        }
    }
}

pub(crate) fn merge_google_options(body: &mut Value, provider_options: &Value) {
    const GENERATION_CONFIG_KEYS: &[&str] = &[
        "candidateCount",
        "frequencyPenalty",
        "logprobs",
        "maxOutputTokens",
        "mediaResolution",
        "presencePenalty",
        "responseLogprobs",
        "responseMimeType",
        "responseModalities",
        "responseSchema",
        "seed",
        "speechConfig",
        "stopSequences",
        "temperature",
        "thinkingConfig",
        "topK",
        "topP",
    ];

    let Some(body_obj) = body.as_object_mut() else {
        return;
    };
    let Some(options_obj) = provider_options.as_object() else {
        return;
    };

    let generation_config = body_obj
        .entry("generationConfig".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let generation_config_obj = generation_config
        .as_object_mut()
        .expect("generationConfig must be an object");
    let mut root_entries: Vec<(String, Value)> = Vec::new();

    for (key, value) in options_obj {
        if GENERATION_CONFIG_KEYS.contains(&key.as_str()) {
            generation_config_obj.insert(key.clone(), value.clone());
        } else {
            root_entries.push((key.clone(), value.clone()));
        }
    }

    for (key, value) in root_entries {
        body_obj.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_openai_compatible_maps_reasoning_fields() {
        let mut body = json!({});
        merge_openai_compatible_options(
            &mut body,
            &json!({
                "reasoningEffort": "high",
                "textVerbosity": "low",
                "store": false,
            }),
        );

        assert_eq!(body["reasoning_effort"], json!("high"));
        assert_eq!(body["verbosity"], json!("low"));
        assert_eq!(body["store"], json!(false));
    }

    #[test]
    fn merge_google_places_thinking_config_under_generation_config() {
        let mut body = json!({
            "generationConfig": {
                "maxOutputTokens": 1024
            }
        });
        merge_google_options(
            &mut body,
            &json!({
                "thinkingConfig": {
                    "includeThoughts": true,
                    "thinkingLevel": "high"
                },
                "cachedContent": "abc"
            }),
        );

        assert_eq!(body["generationConfig"]["thinkingConfig"]["thinkingLevel"], json!("high"));
        assert_eq!(body["cachedContent"], json!("abc"));
    }

}
