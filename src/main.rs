use fr_rust::prelude::*;
use actix_web::{
    App, HttpServer, web::Data as AppData
};
use chatapp::app_config;

#[fr_rust::main]
async fn main() -> MainRlt {
    // 1. Load Environment Variables
    load_env();

    // 2. Configure DDoS Shield
    let ddos_shield = DdosShield::builder()
        .max_requests(5)          // Max requests per window
        .window_secs(1)           // Time window (1 second)
        .ban_duration_secs(20)    // Ban duration for violators
        .block_agent("malicious-bot")
        .allow_missing_ua(false)
        .build();

    // 3. Initialize Shared Services
    let jwt_secret = env_var("JWT_SECRET");
    let jwt = Jwt::new(jwt_secret);
    
    let email_config = EmailConfig {
        smtp_host: env_var("SMTP_HOST"),
        smtp_port: env_var("SMTP_PORT").parse().expect("Invalid SMTP_PORT"),
        smtp_user: env_var("SMTP_USER"),
        smtp_pass: env_var("SMTP_PASS"),
        from_name: env_var("FROM_NAME"),
        from_email: env_var("FROM_EMAIL"),
    };
    let email_service = EmailService::new(email_config).unwrap();
    
    let pool = DbPool::new(env_var("DATABASE_URL"));
    let redis = RedisManager::new(&env_var("REDIS_URL")).unwrap();
    
    let key = env_var("AES_KEY");
    let key_bytes: &[u8; 32] = key.as_bytes().try_into().expect("AES_KEY must be 32 bytes");
    let crypto_service = CryptoService::new(key_bytes).unwrap();
    
    let otp_service = OtpService::new(OtpConfig {
        secret: env_var("KEY"),
        crypto: crypto_service.clone(),
        redis: redis.clone(),
    });
    
    let linkv_service = LinkV::new(LinkVConfig {
        secret: env_var("KEY"),
        crypto: crypto_service.clone(),
        redis: redis.clone(),
    });
    
    let ws = WsManager::new(WsConfig { server: 1, redis: redis.clone() });

    // 4. Start the HTTP Server
    let address = format!("{}:{}", env_var_or_default("IP", "0.0.0.0"), env_var_or_default("PORT", "8080"));
    println!("Starting server at http://{}", address);
    
    HttpServer::new(move || App::new()
        .app_data(AppData::new(email_service.clone()))
        .app_data(AppData::new(pool.clone()))
        .app_data(AppData::new(redis.clone()))
        .app_data(AppData::new(crypto_service.clone()))
        .app_data(AppData::new(otp_service.clone()))
        .app_data(AppData::new(linkv_service.clone()))
        .app_data(AppData::new(jwt.clone()))
        .app_data(AppData::new(ws.clone()))
        .configure(app_config)
        .wrap(ddos_shield.clone())
    )
    .bind(address)?
    .run()
    .await
}
