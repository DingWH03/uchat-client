use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub action: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub action: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub action: String,
    pub receiver: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ServerResponse {
    #[serde(rename = "auth_response")]
    AuthResponse {
        status: String,
        message: String,
    },
    #[serde(rename = "receive_message")]
    ReceiveMessage {
        sender: String,
        message: String,
        timestamp: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}