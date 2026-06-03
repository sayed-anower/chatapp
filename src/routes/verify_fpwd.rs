use fr_rust::prelude::*;
use deadpool_redis::redis::AsyncCommands;
use actix_web::{web::Data as AppData, post, web::Json};

use serde::{Serialize, Deserialize};
// Import the struct
use crate::routes::forgotten_pwd::ForgottenPwd; 

#[derive(Serialize, Deserialize, Clone)]
pub struct VerifyOtp {
    pub email: String,
    pub otp: String,
}

#[post("/verify-fpwd")]
pub async fn verify_fpwd(
    redis: AppData<RedisManager>,
    pool: AppData<DbPool>,
    otp_service: AppData<OtpService>,
    payload: Json<VerifyOtp>,
) -> Rsp {
    let data = payload.into_inner();
    
    // 1. Made 'conn' mutable so Redis commands can execute
    let mut conn = redis.get_connection().await.expect("Redis Failed!");
    let redis_key = format!("fpwd:{}", data.email);
    
    // 2. Safely read as Option<String> from Redis
    let fpwd_json: Option<String> = conn.get(&redis_key).await.ok();
    
    // 3. Convert the JSON String into your struct using Serde
    let fpwd_data: Option<ForgottenPwd> = fpwd_json
        .and_then(|json_str| serde_json::from_str(&json_str).ok());
    
    if let Some(fpwd) = fpwd_data {
        // 4. Verify the user input against the OTP service
        match otp_service.verify_otp(&data.email, &data.otp).await {
            Ok(true) => {
                // 5. Update password in the database
                let update_result = pool.execute(
                    "UPDATE users SET pwd = $1 WHERE email = $2;",
                    &[&fpwd.new_pwd, &data.email]
                ).await;

                if update_result.is_err() {
                    return http_bad("Database update failed.");
                }
                
                // 6. Clean up the used Redis session data
                let _: Result<(), _> = conn.del(&redis_key).await;

                http_ok("Password reset successfully!")
            }
            _ => http_bad("Invalid or expired OTP.")
        }
    } else {
        http_bad("Session expired. Please request a new password reset.")
    }
}
