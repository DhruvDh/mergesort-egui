#![warn(clippy::all)]

mod app;
pub use app::LearningApp;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Default)]
struct PendingState {
    response: Option<String>,
    error: Option<String>,
}

lazy_static! {
    static ref PENDING_STATE: Mutex<PendingState> = Mutex::new(PendingState::default());
}

// Define a structure for the request payload
#[derive(Serialize, Deserialize)]
struct AnthropicRequest {
    messages: Vec<AnthropicMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    system: String,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
    cacheable: bool,
}

// Define a structure for the response
#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentItem>,
}

#[derive(Deserialize)]
struct ContentItem {
    #[serde(default)]
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

// Add ChatMessage to the imports at the top
use crate::app::ChatMessage;

// Add these at the top, after the existing imports
const INSTRUCTIONS: &str = include_str!("mergesort-instructions.md");
const LESSON: &str = include_str!("mergesort-lesson.md");

// Add this new function
fn get_system_message() -> String {
    INSTRUCTIONS.replace("{{LESSON_CONTENT}}", LESSON)
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use reqwasm::http::Request;
    use wasm_bindgen_futures::spawn_local;

    pub(crate) fn make_anthropic_request(
        user_message: String,
        chat_history: Vec<ChatMessage>, // Add chat_history parameter
        callback: impl Fn(Result<String, String>) + 'static,
    ) {
        let url = "https://dhruvdh-anthropic-s-50.deno.dev/";

        let mut messages = Vec::new();

        // Add all previous messages
        for msg in chat_history {
            messages.push(AnthropicMessage {
                role: if msg.from_user { "user" } else { "assistant" }.to_string(),
                content: msg.content,
                cacheable: false,
            });
        }

        // Add the new message
        messages.push(AnthropicMessage {
            role: "user".to_string(),
            content: user_message,
            cacheable: false,
        });

        let request_payload = AnthropicRequest {
            messages,
            max_tokens: Some(2048),
            temperature: Some(0.0),
            system: get_system_message(),
        };

        let payload = match serde_json::to_string(&request_payload) {
            Ok(p) => p,
            Err(e) => {
                callback(Err(format!("Failed to serialize request: {}", e)));
                return;
            }
        };

        spawn_local(async move {
            let response = Request::post(url)
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status() == 200 {
                        match resp.text().await {
                            Ok(body) => match serde_json::from_str::<AnthropicResponse>(&body) {
                                Ok(data) => {
                                    // Extract text from content items
                                    let text = data
                                        .content
                                        .into_iter()
                                        .filter(|item| item.content_type == "text")
                                        .map(|item| item.text)
                                        .collect::<Vec<_>>()
                                        .join("");
                                    callback(Ok(text))
                                }
                                Err(e) => callback(Err(format!("Failed to parse response: {}", e))),
                            },
                            Err(e) => callback(Err(format!("Failed to get response text: {}", e))),
                        }
                    } else {
                        callback(Err(format!("Server error: {}", resp.status())))
                    }
                }
                Err(e) => callback(Err(format!("Network error: {}", e))),
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::sync::Arc;

    pub(crate) fn make_anthropic_request(
        user_message: String,
        chat_history: Vec<ChatMessage>, // Add chat_history parameter
        callback: impl Fn(Result<String, String>) + Send + Sync + 'static,
    ) {
        let url = "https://dhruvdh-anthropic-s-50.deno.dev/";
        let callback = Arc::new(callback);

        let mut messages = Vec::new();

        // Add all previous messages
        for msg in chat_history {
            messages.push(AnthropicMessage {
                role: if msg.from_user { "user" } else { "assistant" }.to_string(),
                content: msg.content,
                cacheable: false,
            });
        }

        // Add the new message
        messages.push(AnthropicMessage {
            role: "user".to_string(),
            content: user_message,
            cacheable: false,
        });

        let request_payload = AnthropicRequest {
            messages,
            max_tokens: Some(2048),
            temperature: Some(0.0),
            system: get_system_message(),
        };

        let payload = match serde_json::to_string(&request_payload) {
            Ok(p) => p,
            Err(e) => {
                callback(Err(format!("Failed to serialize request: {}", e)));
                return;
            }
        };

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let response = client
                .post(url)
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.text().await {
                            Ok(body) => match serde_json::from_str::<AnthropicResponse>(&body) {
                                Ok(data) => {
                                    // Extract text from content items
                                    let text = data
                                        .content
                                        .into_iter()
                                        .filter(|item| item.content_type == "text")
                                        .map(|item| item.text)
                                        .collect::<Vec<_>>()
                                        .join("");
                                    callback(Ok(text))
                                }
                                Err(e) => callback(Err(format!("Failed to parse response: {}", e))),
                            },
                            Err(e) => callback(Err(format!("Failed to get response text: {}", e))),
                        }
                    } else {
                        callback(Err(format!("Server error: {}", resp.status())))
                    }
                }
                Err(e) => callback(Err(format!("Network error: {}", e))),
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) use web::make_anthropic_request;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::make_anthropic_request;
