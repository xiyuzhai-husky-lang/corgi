//! # Husky 包管理服务器
//!
//! 后端路由：
//!
//! PUT /api/v1/crates/new
//! DELETE /api/v1/crates/{crate_name}/{version}/yank
//! PUT /api/v1/crates/{crate_name}/{version}/unyank
//! GET /api/v1/crates/{crate_name}/owners
//! PUT /api/v1/crates/{crate_name}/owners
//! DELETE /api/v1/crates/{crate_name}/owners
//! GET /api/v1/crates
//! /me

use axum::routing::{delete, get, post, put};
use axum::Router;
use std::net::SocketAddr;

async fn test() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init_custom_env("CORGI_LOG");

    let port = 3000;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let owners = Router::new()
        .route("/", get(test))
        .route("/", delete(test))
        .route("/", put(test));

    let inner = Router::new()
        .route("/new", post(test))
        .route("/:crate_name/:version/yank", delete(test))
        .route("/:crate_name/:version/unyank", put(test))
        .nest("/:crate_name/owners", owners)
        .route("/", get(test));

    let router = Router::new()
        .nest("/api/v1/crates", inner)
        .route("/me", post(test));

    log::info!("Server started on {}.", port);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
