use fr_rust::prelude::*;
use actix_web::{post, web::Json, web::Data as AppData};
use serde::{Deserialize};
use serde_json::{
    json,
};

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub pwd: String,
}

#[post("/login")]
pub async fn login(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    jwt: AppData<Jwt>,
    payload: Json<LoginPayload>,
) -> Rsp {
    let credentials = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&credentials.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");
            
            // Securely evaluate argon2/bcrypt hash match asynchronously
            match crypto.verify_hash(&credentials.pwd, &db_hash).await {
                Ok(true) => {
                    let token = jwt.generate_token(&credentials.email).unwrap();
                    http_ok_json(json!({ "token": token, "message": "Login complete" }))
                },
                _ => http_bad("Invalid credentials structural validation."),
            }
        },
        _ => http_bad("User account does not exist."),
    }
}
#[post("/login2")]
pub async fn login2(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    otp_service: AppData<OtpService>,
    email_service: AppData<EmailService>,
    payload: Json<LoginPayload>,
) -> Rsp {
    let credentials = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&credentials.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");
            
            match crypto.verify_hash(&credentials.pwd, &db_hash).await {
                Ok(true) => {
                // Generate secondary OTP challenge verification block
                    let otp = otp_service.generate_otp(&credentials.email, 6, 300).await.unwrap();
                    
                    let email_payload = EmailData {
                        to: credentials.email,
                        subject: "2FA Login Challenge Code".to_string(),
                        body: format!("Your secure login entry phase security code is: {}", otp),
                    };
                    
                    let _ = email_service.send_email(&email_payload).await;
                    http_ok("Verification OTP dispatched to registered destination info.")
                },
                _ => http_bad("Invalid credentials."),
            }
        },
        _ => http_bad("User account does not exist."),
    }
}

#[post("/login3")]
pub async fn login3(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    payload: Json<LoginPayload>,
) -> Rsp {
    let credentials = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&credentials.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");
            
            match crypto.verify_hash(&credentials.pwd, &db_hash).await {
                Ok(true) => {
                    let token = linkv_service.generate_token(&credentials.email, 300).unwrap();
                    let magic_link = format!("https://example.com/verify?v={}", token);
                    
                    let email_payload = EmailData {
                        to: credentials.email,
                        subject: "Magic Login Entry Node Link".to_string(),
                        body: format!("Click here to access your session instantly: {}", magic_link),
                    };
                    
                    let _ = email_service.send_email(&email_payload).await;
                    http_ok("Magic login access hyperlink dispatched.")
                },
                _ => http_bad("Invalid credentials."),
            }
        },
        _ => http_bad("User account does not exist."),
    }
}

#[post("/login4")]
pub async fn login4 (
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    linkv_service: AppData<LinkV>,
    email_service: AppData<EmailService>,
    payload: Json<LoginPayload>,
) -> Rsp {
    let credentials = payload.into_inner();

    let query = "SELECT password FROM users WHERE email = $1 LIMIT 1;";
    match pool.query_opt(query, &[&credentials.email]).await {
        Ok(Some(row)) => {
            let db_hash: String = row.get("password");
            
            match crypto.verify_hash(&credentials.pwd, &db_hash).await {
                Ok(true) => {
                    let token = linkv_service.generate_token(&credentials.email, 300).unwrap();
                    let magic_link = format!("https://example.com/verify-two?v={}", token);
                    
                    let email_payload = EmailData {
                        to: credentials.email,
                        subject: "Magic Login Entry Node Link".to_string(),
                        body: format!("Click here to access your session instantly: {}", magic_link),
                    };
                    
                    let _ = email_service.send_email(&email_payload).await;
                    http_ok("Magic login access hyperlink dispatched.")
                },
                _ => http_bad("Invalid credentials."),
            }
        },
        _ => http_bad("User account does not exist."),
    }
}
