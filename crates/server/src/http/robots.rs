use crate::http::error::{RouteError, RouteResult};
use crate::http::extract::{AuthedUser, ValidatedJson};
use crate::service;
use crate::service::error::ServiceError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use snakerobots_shared::dto::robot::{CreateRobot, RenameRobot};
use snakerobots_shared::dto::Robot;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_robots))
        .route("/", post(create_robot))
        .route("/{id}", get(get_robot))
        .route("/{id}", delete(delete_robot))
        .route("/{id}/rename", post(rename_robot))
        .route("/{id}/upload", post(upload_robot))
        .route("/{id}/download", post(download_robot))
}

async fn list_robots(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
) -> RouteResult<Json<Vec<Robot>>> {
    let robots = service::robot::list_robots(&app, user_id).await?;
    Ok(Json(robots.into_iter()
        .map(|robot| robot.into())
        .collect()))
}

async fn get_robot(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Path(robot_id): Path<String>,
) -> RouteResult<Json<Robot>> {
    let robot = service::robot::get_robot(&app, user_id, robot_id)
        .await?
        .ok_or_else(|| RouteError::not_found())?;
    Ok(Json(robot.into()))
}

async fn create_robot(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    ValidatedJson(create_robot): ValidatedJson<CreateRobot>,
) -> RouteResult<(StatusCode, Json<Robot>)> {
    let result = service::robot::create_robot(&app, user_id, &create_robot.name).await;
    match result {
        Ok(v) => Ok((StatusCode::CREATED, Json(v.into()))),
        Err(ServiceError::LimitReached(_)) => Err(RouteError::new(StatusCode::CONFLICT, "limit_reached", "You reached the limit for the number of robots")),
        Err(e) => Err(e.into())
    }
}

async fn delete_robot(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Path(robot_id): Path<String>,
) -> RouteResult<()> {
    let success = service::robot::delete_robot(&app, user_id, robot_id).await?;
    if success {
        Ok(())
    } else {
        Err(RouteError::not_found())
    }
}

async fn rename_robot(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Path(robot_id): Path<String>,
    ValidatedJson(rename_robot): ValidatedJson<RenameRobot>,
) -> RouteResult<()> {
    let success = service::robot::rename_robot(&app, user_id, robot_id, &rename_robot.name).await?;
    if success {
        Ok(())
    } else {
        Err(RouteError::not_found())
    }
}

async fn upload_robot(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Path(robot_id): Path<String>,
    code: String,
) -> RouteResult<()> {
    let success = service::robot::upload_robot(&app, user_id, robot_id, &code).await?;
    if success {
        Ok(())
    } else {
        Err(RouteError::not_found())
    }
}

async fn download_robot(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Path(robot_id): Path<String>,
) -> RouteResult<String> {
    let code = service::robot::download_robot(&app, user_id, robot_id)
        .await?
        .ok_or_else(RouteError::not_found)?;
    Ok(code)
}
