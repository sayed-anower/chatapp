use fr_rust::prelude::{
    *
};
use actix_web::{web::Data as AppData, post, web::Json};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginUser {
    pub email: String,
    pub pwd: String
}

#[post("/login")]
pub async fn login(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    user_data: Json<LoginUser>,
) -> Rsp {
    let data = user_data.into_inner();

    let query = "SELECT pwd FROM users WHERE email = $1;";
    
    if let Ok(Some(row)) = pool.query_opt(query, &[&data.email]).await {
        let db_hash: String = row.get("pwd");
        
        // Verify provided password against DB hash
        if crypto.verify_hash(&data.pwd, &db_hash).await.unwrap_or(false) {
            http_ok("Login successful!") // You would typically return a JWT token here
        } else {
            http_bad("Invalid credentials.")
        }
    } else {
        http_bad("User not found.")
    }
}

