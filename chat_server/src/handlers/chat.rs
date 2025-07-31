use axum::response::IntoResponse;

pub async fn list_chat_handler() -> impl IntoResponse{
    "chat_list"
}

pub async fn create_chat_handler() -> impl IntoResponse{
    "chat_create"
}

pub async fn update_chat_handler() -> impl IntoResponse{
    "chat_update"
}

pub async fn delete_chat_handler() -> impl IntoResponse{
    "chat_delete"
}