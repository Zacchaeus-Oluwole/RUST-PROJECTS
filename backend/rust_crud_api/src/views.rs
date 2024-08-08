use axum::{response::IntoResponse, extract::Path, http::StatusCode, Extension, Json};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    models::{*, self},
    errors::CustomError,
};

pub async fn all_profiles(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let sql = "SELECT * FROM employee".to_string();
    let profile = sqlx::query_as::<_, Profile>(&sql).fetch_all(&pool).await.unwrap();

    (StatusCode::OK, Json(profile))
}

pub async fn profile(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> Result<Json<models::Profile>, CustomError> {
    let sql = "SELECT * FROM employee where id=$1".to_string();
    let profile : models::Profile = sqlx::query_as(&sql).bind(id).fetch_one(&pool).await.map_err(|_| {
        CustomError::TaskNotFound
    })?;

    Ok(Json(profile))
}

#[axum_macros::debug_handler]
pub async fn post_profile(Extension(pool): Extension<PgPool>, Json(data): Json<NewProfile>) -> Result<(StatusCode, Json<models::NewProfile>), CustomError> {
    let sql = "INSERT INTO employee (id, eid, ename, eemail, econtact) values ($1, $2, $3, $4, $5)".to_string();
    let _  = sqlx::query(&sql)
    .bind(&data.id)
    .bind(&data.eid)
    .bind(&data.ename)
    .bind(&data.eemail)
    .bind(&data.econtact)
    .execute(&pool)
    .await.map_err(|_| {
        CustomError::InternalServerError
    })?;

    Ok((StatusCode::CREATED, Json(data)))
}

pub async fn update_profile(Path(id): Path<i32>, Extension(pool): Extension<PgPool>, Json(data): Json<NewProfile>) -> Result<(StatusCode, Json<models::NewProfile>), CustomError> {
    let sql = "SELECT * FROM employee where id=$1".to_string();
    let _ : models::Profile = sqlx::query_as(&sql).bind(id).fetch_one(&pool).await.map_err(|_| {
        CustomError::TaskNotFound
    })?;

    sqlx::query("UPDATE employee SET eid=$1, ename=$2, eemail=$3, econtact=$4 WHERE id=$5")
    .bind(&data.eid)
    .bind(&data.ename)
    .bind(&data.eemail)
    .bind(&data.econtact)
    .bind(id)
    .execute(&pool)
    .await.map_err(|_| {
        CustomError::InternalServerError
    })?;

    Ok((StatusCode::OK, Json(data)))
}

pub async fn delete_profile(Path(id): Path<i32>, Extension(pool): Extension<PgPool>) -> Result<(StatusCode, Json<Value>), CustomError> {
    let sql = "SELECT * FROM employee where id=$1".to_string();
    let _ : models::Profile = sqlx::query_as(&sql)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        CustomError::TaskNotFound
    })?;

    sqlx::query("DELETE FROM employee WHERE id=$1")
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|_| {
        CustomError::TaskNotFound
    })?;

    Ok((StatusCode::OK ,Json(json!({"msg": "Profile Deleted"}))))
}
