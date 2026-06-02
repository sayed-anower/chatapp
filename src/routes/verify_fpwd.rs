use fr_rust::prelude::*;

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
    // Redis connection
    let conn = redis.get_connection().await.expect("Redis Failed!");
    let redis_key = format!("fpwd:{}", data.email);
    
    let fpwd_data: Option<ForgottenPwd> = conn.get(&redis_key).await.unwrap_or(None);
    
    if let Some(fpwd) = fpwd_data {
        if otp_service.verify_otp(&data.email, &data.otp) {
            // Update password in database
            let _ = pool.execute(
                "UPDATE users SET pwd = $1 WHERE email = $2;",
                &[&fpwd.new_pwd, &data.email]
            ).await;
            
            // Clean up Redis
            let _ = conn.del(&redis_key).await.unwrap();

            http_ok("Password reset successfully!")
        } else {
            http_bad("Invalid or expired OTP.")
        }
    } else {
        http_bad("Session expired. Please request a new password reset.")
    }
}

