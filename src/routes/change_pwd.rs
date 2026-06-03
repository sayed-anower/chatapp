use fr_rust::prelude::{
    *, 
    redis::AsyncCommands
};
use actix_web::{web::Data as AppData, post, web::Json};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChangePwd {
    pub email: String,
    pub old_pwd: String,
    pub new_pwd: String,
}

#[post("/change-pwd")]
pub async fn change_pwd(
    pool: AppData<DbPool>,
    crypto: AppData<CryptoService>,
    payload: Json<ChangePwd>,
) -> Rsp {
    let data = payload.into_inner();
    
    let query = "SELECT pwd FROM users WHERE email = $1;";
    
    if let Ok(Some(row)) = pool.query_opt(query, &[&data.email]).await {
        let db_hash: String = row.get("pwd");
        
        // Ensure old password matches
        if crypto.verify_hash(&data.old_pwd, &db_hash).await.unwrap_or(false) {
            // Hash the new password
            let new_hashed_pwd = crypto.hash_data(&data.new_pwd).await.unwrap();
            
            // Update the DB
            let _ = pool.execute(
                "UPDATE users SET pwd = $1 WHERE email = $2;",
                &[&new_hashed_pwd.hash, &data.email]
            ).await;
            
            http_ok("Password changed successfully!")
        } else {
            http_bad("Incorrect old password.")
        }
    } else {
        http_bad("User not found.")
    }
}