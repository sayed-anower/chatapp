use fr_rust::prelude::*;
use crate::{
    routes::{
        index_file,
        signup,
        login,
        forgottten_pwd,
        verify_fpwd,
        change_pwd,
        verify_signup
    },
    ws::{
        ws_handler
    }
};

// App Configuration
pub fn app_config(cfg: &mut ServiceConfig) {
    cfg
       .service(index_file)
       .service(signup)
       .service(verify_signup)
       .service(login)
       .service(forgotten_pwd)
       .service(verify_fpwd)
       .service(change_pwd)
       .service(ws_handler);
}