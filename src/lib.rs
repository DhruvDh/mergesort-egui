#![warn(clippy::all)]

mod app;
pub use app::LearningApp;

use dotenvy;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

lazy_static! {
    static ref SUPABASE_URL: String = {
        dotenvy::dotenv().ok();
        std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set")
    };
    static ref SUPABASE_ANON_KEY: String = {
        dotenvy::dotenv().ok();
        std::env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY must be set")
    };
}

// Add these structures after other struct definitions
#[derive(Debug, Serialize)]
struct SupabaseOTPRequest {
    email: String,
    create_user: bool,
}

#[derive(Debug, Serialize)]
struct SupabaseVerifyRequest {
    email: String,
    token: String,
    #[serde(rename = "type")]
    auth_type: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub signed_in: bool,
    pub email: Option<String>,
    pub access_token: Option<String>,
}

lazy_static! {
    pub static ref AUTH_STATE: Mutex<AuthState> = Mutex::new(AuthState::default());
}

// Add this function to initialize auth state from storage
pub fn initialize_auth_state(storage: &dyn eframe::Storage) {
    if let Some(stored_auth) = eframe::get_value(storage, "auth_state") {
        let mut auth_state = AUTH_STATE.lock().unwrap();
        *auth_state = stored_auth;
    }
}

// Add this function to save auth state to storage
pub fn save_auth_state(storage: &mut dyn eframe::Storage) {
    let auth_state = AUTH_STATE.lock().unwrap();
    eframe::set_value(storage, "auth_state", &*auth_state);
}

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
    email: String,
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

#[derive(Debug, Deserialize)]
struct SupabaseAuthResponse {
    access_token: String,
}

#[cfg(target_arch = "wasm32")]
pub async fn request_otp_web(email: &str) -> Result<(), String> {
    use reqwasm::http::Request;

    let request = SupabaseOTPRequest {
        email: email.to_string(),
        create_user: true,
    };

    let response = Request::post(&format!("{}/auth/v1/otp", SUPABASE_URL.as_str()))
        .header("apikey", SUPABASE_ANON_KEY.as_str())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request).map_err(|e| e.to_string())?)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let error_text = response.text().await.map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn verify_otp_web(email: &str, token: &str) -> Result<(), String> {
    use reqwasm::http::Request;

    let request = SupabaseVerifyRequest {
        email: email.to_string(),
        token: token.to_string(),
        auth_type: "email".to_string(),
    };

    let response = Request::post(&format!("{}/auth/v1/verify", SUPABASE_URL.as_str()))
        .header("apikey", SUPABASE_ANON_KEY.as_str())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request).map_err(|e| e.to_string())?)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let auth_response: SupabaseAuthResponse =
            response.json().await.map_err(|e| e.to_string())?;

        // Verify we got a valid access token
        if auth_response.access_token.is_empty() {
            return Err("Invalid authentication response".to_string());
        }

        // Update auth state
        let mut auth_state = AUTH_STATE.lock().unwrap();
        auth_state.signed_in = true;
        auth_state.email = Some(email.to_string());
        auth_state.access_token = Some(auth_response.access_token);

        Ok(())
    } else {
        let error_text = response.text().await.map_err(|e| e.to_string())?;
        Err(format!("Failed to verify OTP: {}", error_text))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn request_otp_native(email: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let request = SupabaseOTPRequest {
        email: email.to_string(),
        create_user: true,
    };
    let response = client
        .post(format!("{}/auth/v1/otp", SUPABASE_URL.as_str()))
        .header("apikey", SUPABASE_ANON_KEY.as_str())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error_text = response.text().await.map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn verify_otp_native(email: &str, token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let request = SupabaseVerifyRequest {
        email: email.to_string(),
        token: token.to_string(),
        auth_type: "email".to_string(),
    };

    let response = client
        .post(format!("{}/auth/v1/verify", SUPABASE_URL.as_str()))
        .header("apikey", SUPABASE_ANON_KEY.as_str())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let auth_response: SupabaseAuthResponse =
            response.json().await.map_err(|e| e.to_string())?;

        // Verify we got a valid access token
        if auth_response.access_token.is_empty() {
            return Err("Invalid authentication response".to_string());
        }

        // Update auth state
        let mut auth_state = AUTH_STATE.lock().unwrap();
        auth_state.signed_in = true;
        auth_state.email = Some(email.to_string());
        auth_state.access_token = Some(auth_response.access_token);

        Ok(())
    } else {
        let error_text = response.text().await.map_err(|e| e.to_string())?;
        Err(format!("Failed to verify OTP: {}", error_text))
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use reqwasm::http::Request;
    use wasm_bindgen_futures::spawn_local;

    pub(crate) fn make_anthropic_request(
        user_message: String,
        chat_history: Vec<ChatMessage>,
        callback: impl Fn(Result<String, String>) + 'static,
    ) {
        let url = "https://dhruvdh-anthropic-s-50.deno.dev/";

        let auth_state = AUTH_STATE.lock().unwrap();
        let email = match &auth_state.email {
            Some(email) => email.clone(),
            None => {
                callback(Err("Not authenticated".to_string()));
                return;
            }
        };
        drop(auth_state);

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
            email,
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
        chat_history: Vec<ChatMessage>,
        callback: impl Fn(Result<String, String>) + Send + Sync + 'static,
    ) {
        let url = "https://dhruvdh-anthropic-s-50.deno.dev/";

        let auth_state = AUTH_STATE.lock().unwrap();
        let email = match &auth_state.email {
            Some(email) => email.clone(),
            None => {
                callback(Err("Not authenticated".to_string()));
                return;
            }
        };
        drop(auth_state);

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
            email,
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
