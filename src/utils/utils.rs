use fr_rust::prelude::*;

// Check if user exists in the database
pub async fn if_user_exist(pool: &DbPool, email: &str) -> bool {
    let select_opt_query = "SELECT id FROM users WHERE email = $1;";
    match pool.query_opt(select_opt_query, &[&email]).await {
        Ok(Some(_)) => true,
        _ => false,
    }
}

// Format the email body
pub fn verification_email(
    company_name: &str,
    otp: &str,
    user_name: &str,
    validity_time: i8,
) -> String {
    format!(
        "Hello {},\n\n\
        Your One-Time Password (OTP) for verification is: {}\n\n\
        This code is valid for the next {} minutes. Please do not share this code with anyone.\n\n\
        If you did not request this, please ignore this email.\n\n\
        Best regards,\n\
        {}",
        user_name, otp, validity_time, company_name
    )
}
