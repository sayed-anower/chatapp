use fr_rust::prelude::*;
use actix_web::{post, web::Json, web::Data as AppData};
use deadpool_redis::redis::AsyncCommands;
use serde::Deserialize;

// Payloads for Change Password Actions
#[derive(Deserialize)]
pub struct ChangePasswordPayload {
    pub email: String,
    pub old_pwd: String,
    pub new_pwd: String,
}

#[post("/change-password")]
pub async fn change_password(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    payload: Json<ChangePasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&data.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");

            // Verify old password match
            match crypto.verify_hash(&data.old_pwd, &db_hash).await {
                Ok(true) => {
                    // Hash and save new password
                    let hashed_new_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();
                    let update_query = "UPDATE users SET password = $1 WHERE email = $2;";
                    
                    if pool.execute(update_query, &[&hashed_new_pwd.hash, &data.email]).await.is_ok() {
                        http_ok("Password changed successfully!")
                    } else {
                        http_bad("Failed to update database record.")
                    }
                },
                _ => http_bad("Old password verification failed."),
            }
        },
        _ => http_bad("User account not found."),
    }
}
#[post("/change-password2")]
pub async fn change_password_2(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    otp_service: AppData<OtpService>,
    email_service: AppData<EmailService>,
    redis: AppData<RedisManager>,
    payload: Json<ChangePasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&data.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");

            match crypto.verify_hash(&data.old_pwd, &db_hash).await {
                Ok(true) => {
                    // Generate OTP and Hash the upcoming new password
                    let otp = otp_service.generate_otp(&data.email, 6, 300).await.unwrap();
                    let hashed_new_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();

                    // Temporarily hold the hashed update in Redis cache for 5 mins
                    let mut conn = redis.get_connection().await.expect("Redis Failed!");
                    let _: Result<(), _> = conn.set_ex(format!("pending_pwd:{}", data.email), hashed_new_pwd.hash, 300).await;

                    let email_data = EmailData {
                        to: data.email,
                        subject: "Confirm Password Change Request".to_string(),
                        body: format!("Use this code to finalize your password update: {}. Valid for 5 minutes.", otp),
                    };

                    match email_service.send_email(&email_data).await {
                        Ok(_) => http_ok("Password update initiated. Please check your email for the validation OTP!"),
                        Err(_) => http_bad("Failed to send verification email."),
                    }
                },
                _ => http_bad("Old password verification failed."),
            }
        },
        _ => http_bad("User account not found."),
    }
}
#[post("/change-password3")]
pub async fn change_password_3(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    redis: AppData<RedisManager>,
    payload: Json<ChangePasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&data.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");

            match crypto.verify_hash(&data.old_pwd, &db_hash).await {
                Ok(true) => {
                    let token = linkv_service.generate_token(&data.email, 300).unwrap();
                    let hashed_new_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();

                    let mut conn = redis.get_connection().await.expect("Redis Failed!");
                    let _: Result<(), _> = conn.set_ex(format!("pending_pwd:{}", data.email), hashed_new_pwd.hash, 300).await;

                    let confirm_url = format!("https://example.com/verify?v={}", token);
                    let email_data = EmailData {
                        to: data.email,
                        subject: "Authorize Password Modification".to_string(),
                        body: format!("Click this secure node link to authorize your new account password: {}", confirm_url),
                    };

                    match email_service.send_email(&email_data).await {
                        Ok(_) => http_ok("Authorization link has been dispatched to your email."),
                        Err(_) => http_bad("Could not dispatch authorization email."),
                    }
                },
                _ => http_bad("Old password verification failed."),
            }
        },
        _ => http_bad("User account not found."),
    }
}

#[post("/change-password4")]
pub async fn change_password_4(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    redis: AppData<RedisManager>,
    payload: Json<ChangePasswordPayload>,
) -> Rsp {
    let data = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&data.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");

            match crypto.verify_hash(&data.old_pwd, &db_hash).await {
                Ok(true) => {
                    let token = linkv_service.generate_token(&data.email, 300).unwrap();
                    let hashed_new_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();

                    let mut conn = redis.get_connection().await.expect("Redis Failed!");
                    let _: Result<(), _> = conn.set_ex(format!("pending_pwd:{}", data.email), hashed_new_pwd.hash, 300).await;

                    let confirm_url = format!("https://example.com/verify-two?v={}", token);
                    let email_data = EmailData {
                        to: data.email,
                        subject: "Authorize Password Modification".to_string(),
                        body: format!("Click this secure node link to authorize your new account password: {}", confirm_url),
                    };

                    match email_service.send_email(&email_data).await {
                        Ok(_) => http_ok("Authorization link has been dispatched to your email."),
                        Err(_) => http_bad("Could not dispatch authorization email."),
                    }
                },
                _ => http_bad("Old password verification failed."),
            }
        },
        _ => http_bad("User account not found."),
    }
}
