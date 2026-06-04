use fr_rust::prelude::*;
use crate::utils::if_user_exist;
use actix_web::{post, web::Json, web::Data as AppData};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TempSignup {
    pub name: String,
    pub email: String,
    pub pwd: String,
}

#[post("/signup")]
pub async fn signup(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    user_data: Json<TempSignup>,
) -> Rsp {
    let data = user_data.into_inner();

    if if_user_exist(&pool, &data.email).await {
        return http_bad("User already exists with this email.");
    }

    let hashed_pwd = crypto.hash_data(&data.pwd).await.unwrap();

    let query = "INSERT INTO users (name, email, password) VALUES ($1, $2, $3);";
    match pool.execute(query, &[&data.name, &data.email, &hashed_pwd.hash]).await {
        Ok(_) => http_ok("Registration successful!"),
        Err(_) => http_bad("Database insertion failed."),
    }
}

#[post("/signup2")]
pub async fn signup2(
    pool: AppData<DbPool>,
    otp_service: AppData<OtpService>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    redis: AppData<RedisManager>,
    user_data: Json<TempSignup>,
) -> Rsp {
    let data = user_data.into_inner();

    if if_user_exist(&pool, &data.email).await {
        return http_bad("User already exists with this email.");
    }

    let otp = otp_service.generate_otp(&data.email, 6, 300).await.unwrap();
    let hashed_pwd = crypto.hash_data(&data.pwd).await.unwrap();
    
    let temp_user = TempSignup {
        name: data.name.clone(),
        email: data.email.clone(),
        pwd: hashed_pwd.hash,
    };

    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let signup_json = serde_json::to_string(&temp_user).unwrap();
    let _: Result<(), _> = conn.set_ex(format!("signup:{}", data.email), signup_json, 300).await;

    let email_data = EmailData {
        to: data.email,
        subject: "Verify Account OTP".to_string(),
        body: format!("Your registration security code is: {}. Valid for 5 mins.", otp),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("OTP transmitted successfully! Please finalize via /verify-otp."),
        Err(_) => http_bad("Failed to emit signup OTP."),
    }
}

#[post("/signup3")]
pub async fn signup3(
    pool: AppData<DbPool>,
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    redis: AppData<RedisManager>,
    user_data: Json<TempSignup>,
) -> Rsp {
    let data = user_data.into_inner();

    if if_user_exist(&pool, &data.email).await {
        return http_bad("User already exists with this email.");
    }

    let token = linkv_service.generate_token(&data.email, 300).unwrap();
    let hashed_pwd = crypto.hash_data(&data.pwd).await.unwrap();
    
    let temp_user = TempSignup {
        name: data.name.clone(),
        email: data.email.clone(),
        pwd: hashed_pwd.hash,
    };

    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let signup_json = serde_json::to_string(&temp_user).unwrap();
    let _: Result<(), _> = conn.set_ex(format!("signup:{}", data.email), signup_json, 300).await;

    let verification_url = format!("https://example.com/verify?v={}", token);
    
    let email_data = EmailData {
        to: data.email,
        subject: "Complete your Signup".to_string(),
        body: format!("Click this activation node link to finalize setup: {}", verification_url),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("Activation link dispatched to email."),
        Err(_) => http_bad("Could not dispatch activation link."),
    }
}

#[post("/signup4")]
pub async fn signup4 (
    pool: AppData<DbPool>,
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    redis: AppData<RedisManager>,
    user_data: Json<TempSignup>,
) -> Rsp {
    let data = user_data.into_inner();

    if if_user_exist(&pool, &data.email).await {
        return http_bad("User already exists with this email.");
    }

    let token = linkv_service.generate_token(&data.email, 300).unwrap();
    let hashed_pwd = crypto.hash_data(&data.pwd).await.unwrap();
    
    let temp_user = TempSignup {
        name: data.name.clone(),
        email: data.email.clone(),
        pwd: hashed_pwd.hash,
    };

    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let signup_json = serde_json::to_string(&temp_user).unwrap();
    let _: Result<(), _> = conn.set_ex(format!("signup:{}", data.email), signup_json, 300).await;

    let verification_url = format!("https://example.com/verify-two?v={}", token);
    
    let email_data = EmailData {
        to: data.email,
        subject: "Complete your Signup".to_string(),
        body: format!("Click this activation node link to finalize setup: {}", verification_url),
    };

    match email_service.send_email(&email_data).await {
        Ok(_) => http_ok("Activation link dispatched to email."),
        Err(_) => http_bad("Could not dispatch activation link."),
    }
}
