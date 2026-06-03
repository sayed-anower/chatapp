use fr_rust::prelude::{
    *, 
};
use actix_web::{
    App, HttpServer, web::Data as AppData
};
use chatapp::app_config;
#[fr_rust::main]
async fn main() -> MainRlt {
    // Load environment variables
    load_env();
    /* SHARED STATES */
    // Email
    let email_config = EmailConfig {
        smtp_host: env_var("SMTP_HOST"),
        smtp_port: env_var("SMTP_PORT").parse().expect("SMTP_PORT must be a valid integer"),
        smtp_user: env_var("SMTP_USER"),
        smtp_pass: env_var("SMTP_PASS"),
        from_name: env_var("FROM_NAME"),
        from_email: env_var("FROM_EMAIL"),
    };
    let email_service = EmailService::new(email_config).unwrap();
    
    // Database
    let database_url = env_var("DATABASE_URL");
    let pool = DbPool::new(database_url);

    // Redis
    let redis_url = env_var("REDIS_URL");
    let redis = RedisManager::new(&redis_url).unwrap();
    // Crypto
    let key = env_var("AES_KEY");
    let key_bytes: &[u8; 32] = key.as_bytes().try_into().expect("AES_KEY must be exactly 32 bytes");
    let crypto_service = CryptoService::new(key_bytes).unwrap();
    // Otp verification
    let otp_config = OtpConfig {
        secret: env_var("KEY"),
        crypto: crypto_service.clone(),
        redis: redis.clone(),
        ttl_secs: 300 
    };
    let otp_service = OtpService::new(otp_config);
    // Web Socket
    let ws_config = WsConfig {
        server: 1,
        redis: redis.clone()
    };
    let ws = WsManager::new(ws_config);
    /* IP & PORTS */
    let ip = env_var_or_default("IP", "0.0.0.0");
    let port = env_var_or_default("PORT", "8080");
    let address = format!("{}:{}", ip, port);
    /* START SERVER */
    println!("Starting server at http://{}", address);
    HttpServer::new(move || App::new()
    .app_data(AppData::new(email_service.clone()))
    .app_data(AppData::new(pool.clone()))
    .app_data(AppData::new(redis.clone()))
    .app_data(AppData::new(crypto_service.clone()))
    .app_data(AppData::new(otp_service.clone()))
    .app_data(AppData::new(ws.clone()))
    .configure(app_config)
    ).bind(address)?.run().await
}