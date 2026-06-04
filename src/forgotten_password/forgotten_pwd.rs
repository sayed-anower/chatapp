use fr_rust::prelude::*;
use actix_web::{post, web::Json, web::Data as AppData};
use deadpool_redis::redis::AsyncCommands;
use serde::Deserialize;

// Payloads for Forgotten Password Actions
#[derive(Deserialize)]
pub struct ForgotPasswordPayload {
    pub email: String,
    pub new_pwd: String,
}

#[post("/forgotten-password")]
pub async fn forgotten_password(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    payload: Json<ForgotPasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    let hashed_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();
    let query = "UPDATE users SET password = $1 WHERE email = $2;";

    match pool.execute(query, &[&hashed_pwd.hash, &data.email]).await {
        Ok(_) => http_ok("Your account password has been successfully reset!"),
        Err(_) => http_bad("Failed to complete password reset workflow in the system database."),
    }
}
#[post("/forgotten-password2")]
pub async fn forgotten_password_2(
    otp_service: AppData<OtpService>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    redis: AppData<RedisManager>,
    payload: Json<ForgotPasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    // Generate Verification Token OTP
    let otp = otp_service.generate_otp(&data.email, 6, 300).await.unwrap();
    let hashed_new_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();

    // Cache the intent to write over old data once authenticated via /verify-otp
    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let _: Result<(), _> = conn.set_ex(format!("forgot_pwd:{}", data.email), hashed_new_pwd.hash, 300).await;

    let email_data = EmailData {
        to: data.email,
        subject: "Your Password Reset OTP Code".to_string(),
        body: format!("Your temporary recovery security verification code is: {}. Enter this to save your new password.", otp),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("Password reset code triggered. Please inspect your inbox."),
        Err(_) => http_bad("Failed to emit recovery email data payload."),
    }
}
#[post("/forgotten-password3")]
pub async fn forgotten_password_3(
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    redis: AppData<RedisManager>,
    payload: Json<ForgotPasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    // Generate link tracking token
    let token = linkv_service.generate_token(&data.email, 300).unwrap();
    let hashed_new_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();

    // Stash the hash update into Redis cache
    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let _: Result<(), _> = conn.set_ex(format!("forgot_pwd:{}", data.email), hashed_new_pwd.hash, 300).await;

    let recovery_url = format!("https://example.com/verify?v={}", token);
    let email_data = EmailData {
        to: data.email,
        subject: "Secure Recovery Password Link Request".to_string(),
        body: format!("Click on the following address parameter route to finish changing your access token credentials: {}", recovery_url),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("Recovery magic interface code successfully distributed via link address!"),
        Err(_) => http_bad("Failed to send link vector payload."),
    }
}
#[post("/forgotten-password4")]
pub async fn forgotten_password_4(
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    redis: AppData<RedisManager>,
    payload: Json<ForgotPasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    // Generate link tracking token
    let token = linkv_service.generate_token(&data.email, 300).unwrap();
    let hashed_new_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();

    // Stash the hash update into Redis cache
    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let _: Result<(), _> = conn.set_ex(format!("forgot_pwd:{}", data.email), hashed_new_pwd.hash, 300).await;

    let recovery_url = format!("https://example.com/verify-two?v={}", token);
    let email_data = EmailData {
        to: data.email,
        subject: "Secure Recovery Password Link Request".to_string(),
        body: format!("Click on the following address parameter route to finish changing your access token credentials: {}", recovery_url),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("Recovery magic interface code successfully distributed via link address!"),
        Err(_) => http_bad("Failed to send link vector payload."),
    }
}
