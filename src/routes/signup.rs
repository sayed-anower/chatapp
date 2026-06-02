use fr_rust::prelude::*;
 use actix_web::{post, web::{
    Data as AppData,
    Json
}, App};

use serde::{Deserialize, Serialize};

use futures_util::StreamExt; 
use crate::{
    utils::{
        if_user_exist,
        verification_email,
    },
};

use fr_rust::redis::AsyncCommands;

#[derive(Serialize, Deserialize, Clone)]
pub struct TempSignup {
    pub name: String,
    pub email: String,
    pub pwd: String,
}

#[post("/signup")]
pub async fn signup(
    pool: AppData<DbPool>,
    email_service: AppData<EmailService>,
    otp_service: AppData<OtpService>,
    crypto: AppData<CryptoService>,
    redis: AppData<RedisManager>,
    user_data: Json<TempSignup>,
) -> Rsp {
    let data = user_data.into_inner();

    if if_user_exist(&pool, &data.email).await {
        return http_bad("User already exists with this email.");
    }

    // Generate 6-digit OTP
    let otp = otp_service.generate_otp(&data.email, 6).await.unwrap();

    
    // Hash the password before temporarily saving to Redis
    let hashed_pwd = crypto.hash_data(&data.pwd).await.unwrap();
    
    let temp_user = TempSignup {
        name: data.name.clone(),
        email: data.email.clone(),
        pwd: hashed_pwd.hash,
    };

    let redis_key = format!("signup:{}", temp_user.email);
    
    // Redis connection
    let conn = redis.get_connection().await.expect("Redis Failed!");
    
    let signup_json = serde_json::to_string(&temp_user).expect("Failed to serialize");
    // Save to Redis with 300s (5m) TTL
    let _: Result<(), _> = conn.set_ex(&redis_key, fpwd_json, 300).await;

    // Send Email
    let email_data = EmailData {
        to: data.email,
        subject: "Verify Your Account".to_string(),
        body: verification_email("My Company", &otp, &data.name, 5),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("Signup initiated. Please check your email for the OTP!"),
        Err(_) => http_bad("Failed to send verification email."),
    }
}