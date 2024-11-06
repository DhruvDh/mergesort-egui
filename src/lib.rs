#![warn(clippy::all)]

mod app;
pub use app::LearningApp;

use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use lazy_static::lazy_static;
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
    completion: String,
}

pub fn make_anthropic_request(
    user_message: String,
    callback: impl Fn(Result<String, String>) + 'static
) {
    let url = "https://dhruvdh-anthropic-s-50.deno.dev/";

    let request_payload = AnthropicRequest {
        messages: vec![AnthropicMessage {
            role: "user".to_string(),
            content: user_message,
            cacheable: false,
        }],
        max_tokens: Some(2048),
        temperature: Some(0.0),
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
                        Ok(body) => {
                            match serde_json::from_str::<AnthropicResponse>(&body) {
                                Ok(data) => callback(Ok(data.completion)),
                                Err(e) => callback(Err(format!("Failed to parse response: {}", e))),
                            }
                        }
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

