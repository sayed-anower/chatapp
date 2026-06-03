use fr_rust::prelude::{
    *, 
    redis::AsyncCommands
};
use actix_web::{web::Data as AppData, post, Json};
use serde::{Serialize, Deserialize};
// Assuming these come from your internal crates/modules
use crate::{
    utils::{if_user_exist, verification_email},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct ForgottenPwd {
    pub email: String,
    pub new_pwd: String,
}

#[post("/forgotten-pwd")]
pub async fn forgotten_pwd(
    pool: AppData<DbPool>,
    email_service: AppData<EmailService>,
    crypto: AppData<CryptoService>,
    otp_service: AppData<OtpService>,
    redis: AppData<RedisManager>,
    payload: Json<ForgottenPwd>,
) -> Rsp {
    let data = payload.into_inner();
    
    // Added 'mut' because Redis AsyncCommands require a mutable reference
    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    
    if !if_user_exist(&pool, &data.email).await {
        return http_bad("If this email exists, an OTP has been sent."); 
    }

    let otp = otp_service.generate_otp(&data.email, 6).await.unwrap();
    
    // Hash the new requested password
    let redis_key = format!("fpwd:{}", data.email);
    let hashed_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();

    // Store the user's email and their NEW HASHED password in Redis
    let pending_reset = ForgottenPwd {
        email: data.email.clone(),
        new_pwd: hashed_pwd.hash,
    };

    let fpwd_json = serde_json::to_string(&pending_reset).expect("Failed to serialize");
    
    // Fixed Redis set syntax and type specification
    let _: Result<(), _> = conn.set_ex(&redis_key, fpwd_json, 300).await;

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
