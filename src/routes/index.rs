use fr_rust::prelude::{
    FileRlt,
    send_file,
};
use actix_web::{get};

#[get("/")]
pub async fn index_file() -> FileRlt {
    send_file("./static/index.html").await
}
