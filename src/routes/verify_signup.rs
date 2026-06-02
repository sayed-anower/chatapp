use fr_rust::prelude::*;
use fr_rust::redis::AsyncCommands;
use actix_web::{post, web::{
    Data as AppData,
    Json
}, App};
use crate::routes::TempSignup;
use serde::{Deserialize, Serialize};
use futures_util::StreamExt; 

#[derive(Serialize, Deserialize, Clone)]
pub struct VerifyOtp {
    pub email: String,
    pub otp: String,
}

#[post("/verify-signup")]
pub async fn verify_signup(
    redis: AppData<RedisManager>,
    pool: AppData<DbPool>,
    otp_service: AppData<OtpService>,
    payload: Json<VerifyOtp>,
) -> Rsp {
    let data = payload.into_inner();
    // Redis connection
    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let redis_key = format!("signup:{}", data.email);

    // Fetch pending user data from Redis
    let signup_json: Option<String> = conn.get(&redis_key).await.unwrap_or(None);
    let signup_data: Option<TempSignup> = signup_json
    .and_then(|json_str| serde_json::from_str(&json_str).ok());
    
    if let Some(user) = signup_data {
        if otp_service.verify_otp(&user.email, &data.otp).await.unwrap() {
            // Save verified user in DB
            let _ = pool.execute(
                "INSERT INTO users (name, email, pwd) VALUES ($1, $2, $3)",
                &[&user.name, &user.email, &user.pwd]
            ).await;
            
            // Clean up Redis
            let _: usize = conn.del(&redis_key).await.unwrap();

            http_ok("Signup successful!")
        } else {
            http_bad("Invalid or expired OTP.")
        }
    } else {
        http_bad("Session expired or email not found. Please sign up again.")
    }
}