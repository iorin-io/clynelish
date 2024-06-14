use axum::{
    extract::{Json, Extension, Path},
    response::IntoResponse,
    http::StatusCode,
};
use sqlx::{query_as, query};
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::db::AppState;
use crate::models::transaction::Transaction;

pub async fn create_transaction(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    Json(new_transaction): Json<Transaction>
) -> impl IntoResponse {
    let db_pool = state.lock().await.db_pool.clone();

    match query!(
        "INSERT INTO Transactions (account_id, child_category_id, transaction_amount, transaction_type, transaction_date, transaction_description) VALUES (?, ?, ?, ?, ?, ?)",
        new_transaction.account_id,
        new_transaction.child_category_id,
        new_transaction.transaction_amount,
        new_transaction.transaction_type,
        new_transaction.transaction_date,
        new_transaction.transaction_description
    )
    .execute(&db_pool)
    .await
    {
        Ok(result) => {
            let transaction_id = result.last_insert_id();
            match query_as!(
                Transaction,
                "SELECT transaction_id, account_id, child_category_id, transaction_amount, transaction_type, transaction_date, transaction_description FROM Transactions WHERE transaction_id = ?",
                transaction_id
            )
            .fetch_one(&db_pool)
            .await {
                Ok(transaction) => (StatusCode::CREATED, Json(transaction)).into_response(),
                Err(e) => {
                    eprintln!("Failed to fetch transaction after creation: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to create transaction: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_transaction(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    Path(transaction_id): Path<i32>,
) -> impl IntoResponse {
    let db_pool = state.lock().await.db_pool.clone();

    match query_as!(
        Transaction,
        "SELECT transaction_id, account_id, child_category_id, transaction_amount, transaction_type, transaction_date, transaction_description FROM Transactions WHERE transaction_id = ?",
        transaction_id
    )
    .fetch_one(&db_pool)
    .await
    {
        Ok(transaction) => (StatusCode::OK, Json(transaction)).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn update_transaction(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    Path(transaction_id): Path<i32>,
    Json(transaction): Json<Transaction>
) -> impl IntoResponse {
    let db_pool = state.lock().await.db_pool.clone();

    match query!(
        "UPDATE Transactions SET transaction_amount = ?, transaction_type = ?, transaction_date = ?, transaction_description = ? WHERE transaction_id = ?",
        transaction.transaction_amount,
        transaction.transaction_type,
        transaction.transaction_date,
        transaction.transaction_description,
        transaction_id
    )
    .execute(&db_pool)
    .await
    {
        Ok(_) => {
            match query_as!(
                Transaction,
                "SELECT transaction_id, account_id, child_category_id, transaction_amount, transaction_type, transaction_date, transaction_description FROM Transactions WHERE transaction_id = ?",
                transaction_id
            )
            .fetch_one(&db_pool)
            .await {
                Ok(updated_transaction) => (StatusCode::OK, Json(updated_transaction)).into_response(),
                Err(e) => {
                    eprintln!("Failed to fetch transaction after update: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to update transaction: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_transaction(
    Extension(state): Extension<Arc<Mutex<AppState>>>,
    Path(transaction_id): Path<i32>,
) -> impl IntoResponse {
    let db_pool = state.lock().await.db_pool.clone();

    match query!(
        "DELETE FROM Transactions WHERE transaction_id = ?",
        transaction_id
    )
    .execute(&db_pool)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
