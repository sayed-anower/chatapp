use fr_rust::prelude::{
    *, ServiceConfig
};
use crate::{
    routes::{
        index_file,
        signup,
        login,
        forgotten_pwd,
        verify_fpwd,
        change_pwd,
        verify_signup
    },
};
use crate::ws::ws_handler::ws_handler;

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