use fr_rust::prelude::*;

use crate::{
    utils::{
        if_user_exist,
        verification_email,
    },
};
use fr_rust::redis::AsyncCommands;

use actix_web::{post, web::{
    Data as AppData,
    Json
}};

use serde::{Deserialize, Serialize};
use futures_util::StreamExt; 

#[derive(Serialize, Deserialize, Clone)]
pub struct ForgottenPwd {
    pub email: String,
    pub new_pwd: String,
}

#[post("/forgotten-pwd")]
pub async fn forgotten_pwd<RV>(
    pool: AppData<DbPool>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    otp_service: AppData<OtpService>,
    redis: AppData<RedisManager>,
    payload: Json<ForgottenPwd>,
) -> Rsp {
    let data = payload.into_inner();
    
    // Redis connection
    let conn = redis.get_connection().await.expect("Redis Failed!");
    
    if !if_user_exist(&pool, &data.email).await {
        return http_bad("If this email exists, an OTP has been sent."); // Good practice to prevent email enumeration
    }

    let otp = otp_service.generate_otp(&data.email, 6).await.unwrap();
    
    // Hash the new requested password
    let redis_key = format!("fpwd:{}", data.email);
    let hashed_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();
    let fpwd_json: Option<String> = conn.get(&redis_key).await.unwrap_or(None);
    let fpwd_data: Option<ForgottenPwd> = fpwd_json
    .and_then(|json_str| serde_json::from_str(&json_str).ok());
    let fpwd_json = serde_json::to_string(&fpwd_data).expect("Failed to serialize");
    let _: Result<RV, _> = conn.set_ex(&redis_key, fpwd_json, 300).await;

    let email_data = EmailData {
        to: data.email,
        subject: "Password Reset OTP".to_string(),
        body: verification_email("My Company", &otp, "User", 5),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("OTP sent to your email."),
        Err(_) => http_bad("Failed to send OTP email."),
    }
}

