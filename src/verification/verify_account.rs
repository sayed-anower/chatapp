use fr_rust::prelude::*;
use actix_web::{get, post, web, web::Data as AppData};
use serde::{Deserialize};
use serde_json::{
    json,
};

// Query param extractor for: /verify?v=token or /verify-two?v=token
#[derive(Deserialize)]
pub struct TokenQuery {
    v: String,
}

// JSON payload extractor for: /verify-otp
#[derive(Deserialize)]
pub struct OtpPayload {
    key: String,
    otp: String,
}

// Standard structural payload for internal communication / mock extraction
struct UserContext {
    user_id: String,
    email: String,
}

/// Validates submitted OTP via JSON -> Returns Login Token JWT
#[post("/verify-otp")]
pub async fn verify_otp_route(
    payload: web::Json<OtpPayload>,
    otp_service: AppData<OtpService>,
    jwt: AppData<Jwt>,
) -> Rsp {

    // Verify OTP explicitly against stored Redis backplane criteria
    match otp_service.verify_otp(&payload.key, &payload.otp).await {
        Ok(true) => {
            // Generate standard login JWT payload
            match jwt.generate_token(&payload.key) {
                Ok(token) => http_ok_json(json!({
                    "success": true,
                    "message": "OTP verification successful.",
                    "login_token": token
                })),
                Err(_) => http_bad("Authentication succeeded, but application session setup failed."),
            }
        },
        _ => http_bad_json(json!({
            "success": false,
            "error": "Invalid or expired OTP code code."
        })),
    }
}

/// Validates link token -> Instantly returns Login Token JWT
#[get("/verify")]
pub async fn verify_route(
    query: web::Query<TokenQuery>,
    linkv_service: AppData<LinkV>,
    jwt: AppData<Jwt>,
) -> Rsp {
    // 1. Get the real user
    let target_user = jwt.parse_token(&query.v).unwrap();

    // Validate link token
    match linkv_service.verify_token(&query.v) {
        Ok(true) => {
            // Generate standard login JWT payload (No expiration per framework docs example)
            match jwt.generate_token(&target_user.sub) {
                Ok(token) => http_ok_json(json!({
                    "success": true,
                    "message": "Logged in successfully",
                    "login_token": token
                })),
                Err(_) => http_bad("Failed to generate application access token."),
            }
        },
        _ => http_bad("Invalid or expired verification link."),
    }
}

/// Validates link token -> Generates OTP -> Emails OTP -> Returns HTML form.
#[get("/verify-two")]
pub async fn verify_two_route(
    query: web::Query<TokenQuery>,
    linkv_service: AppData<LinkV>,
    otp_service: AppData<OtpService>,
    email_service: AppData<EmailService>,
    jwt: AppData<Jwt>,
) -> Rsp {
    // 1. Get the real user
    let target_user = jwt.parse_token(&query.v).unwrap();
    // 2. Validate the link token
    match linkv_service.verify_token(&query.v) {
        Ok(true) => {
            // 3. Link verified! Now generate a 6-digit OTP
            let otp = match otp_service.generate_otp(&target_user.sub, 6, 300).await {
                Ok(code) => code,
                Err(_) => return http_bad("Failed to initialize secondary verification code."),
            };

            // 4. Send the OTP via Email
            let email_data = EmailData {
                to: target_user.sub.to_string(),
                subject: "Your Verification Code".to_string(),
                body: format!("Your secure OTP code is: {}. It expires in 5 minutes.", otp),
            };

            if email_service.send_email(&email_data).await.is_err() {
                return http_bad("Could not transmit verification message.");
            }

            // 5. Return HTML Form Page to prompt user for OTP submission
            let html_page = format!(
                r#"<!DOCTYPE html>
                <html>
                <head><title>Verify OTP</title></head>
                <body>
                    <h2>Enter the OTP sent to your email</h2>
                    <form action="/verify-otp" method="POST" enctype="application/json">
                        <input type="hidden" id="key" name="key" value="{}">
                        <input type="text" id="otp" name="otp" placeholder="Enter 6-Digit OTP" required>
                        <button type="submit">Verify & Login</button>
                    </form>
                </body>
                </html>"#, 
                target_user.sub
            );
            
            send_str(&html_page)
        },
        _ => http_bad("Link verification failed or token expired."),
    }
}