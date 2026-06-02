use fr_rust::prelude::{
    get,
    FileRlt,
    send_file
};

#[get("/")]
pub async fn index_file() -> FileRlt {
    send_file("./static/index.html").await
}
