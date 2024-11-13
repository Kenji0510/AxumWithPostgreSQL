use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
    debug_handler,
};
use chrono::format;
use serde_json::json;

use crate::{
    error::AppError, model::NoteModel, schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema}, AppState
};

pub async fn health_checker_handler() -> impl IntoResponse {
    println!("--> {:12} - Accessed /api/healthchecker", "HANDLER");
    const MESSAGE: &str = "Simple health checker service is running!";

    let json_response = json!({
        "status": "success",
        "message": MESSAGE,
    });

    Json(json_response)
}

#[debug_handler]
pub async fn note_list_handler(
    //Query(opts): Query<FilterOptions>,
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
//) -> Result<Json<serde_json::Value>, StatusCode> {
//) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
) -> Result<impl IntoResponse, AppError> {
    println!("--> {:12} - Accessed /api/notes", "HANDLER");
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query_result = sqlx::query_as!(
        NoteModel,
        "SELECT * FROM notes ORDER by id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await;

    if query_result.is_err() {
        // let error_response = serde_json::json!({
        //     "status": "failed",
        //     "message": "Failed to fetch notes",
        // });
        // return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        return Err(AppError::DatabaseError.into());
        //return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let notes = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": notes.len(),
        "notes": notes,
    });

    Ok(Json(json_response))
    //Ok(Json(serde_json::json!({"status": "OK"})))
}

pub async fn create_note_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateNoteSchema>,
//) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
) -> Result<impl IntoResponse, AppError> {
    println!("--> {:12} - Accessed /api/notes/", "HANDLER");

    let query_result = sqlx::query_as!(
        NoteModel,
        "INSERT INTO notes (title,content,category) VALUES ($1, $2, $3) RETURNING *",
        body.title.to_string(),
        body.content.to_string(),
        body.category.to_owned().unwrap_or("".to_string())
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(note) => {
            let note_response = json!({
                "status": "success",
                "data": json!({
                    "note": note,
                })
            });
            
            return Ok((StatusCode::CREATED, Json(note_response)));
        }
        Err(err) => {
            if err.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                // let error_response = serde_json::json!({
                //     "status": "failed",
                //     "message": "Note with that title already exists",
                // });

                // return Err((StatusCode::CONFLICT, Json(error_response)));
                return Err(AppError::Conflict.into());
            }
            // return Err((
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     Json(json!({
            //         "status": "failed",
            //         "message": format!("{:?}", err),
            //     }))
            // ));
            return Err(AppError::DatabaseError.into());
        }
    }
}

pub async fn get_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
//) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
) -> Result<impl IntoResponse, AppError> {
    println!("--> {:12} - Accessed /api/notes", "HANDLER");
    let query_result = sqlx::query_as!(
        NoteModel,
        "SELECT * FROM notes WHERE id = $1", id
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "note": note,
                })
            });
            return Ok(Json(note_response));
        }
        Err(_) => {
            // let error_response = serde_json::json!({
            //     "status": "failed",
            //     "message": format!("Note with ID: {} not found", id),
            // });
            // return Err((StatusCode::NOT_FOUND, Json(error_response)));
            return Err(AppError::NotFound.into());
        }
    }
}

pub async fn edit_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, AppError> {
    println!("--> {:12} - Accessed /api/notes/:{}", "HANDLER", id.to_string());
    let query_result = sqlx::query_as!(
        NoteModel,
        "SELECT * FROM notes WHERE id = $1",
        id
    )
    .fetch_one(&data.db)
    .await;

    if query_result.is_err() {
        return Err(AppError::NotFound.into());
    }

    let now = chrono::Utc::now();
    let note = query_result.unwrap();

    let query_result = sqlx::query_as!(
        NoteModel,
        "UPDATE notes SET title = $1, content = $2, category = $3, published = $4, updated_at = $5 WHERE id = $6 RETURNING *",
        body.title.to_owned().unwrap_or(note.title),
        body.content.to_owned().unwrap_or(note.content),
        body.category.to_owned().unwrap_or(note.category.unwrap()),
        body.published.unwrap_or(note.published.unwrap()),
        now,
        id
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "note": note
                })
            });
            return Ok(Json(note_response));
        }
        Err(_) => {
            return Err(AppError::DatabaseError.into());
        }
    }
}

pub async fn delete_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows_affected = sqlx::query!(
        "DELETE FROM notes WHERE id = $1",
        id
    )
    .execute(&data.db)
    .await
    .unwrap()
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound.into());
    }

    Ok(StatusCode::NO_CONTENT)
}