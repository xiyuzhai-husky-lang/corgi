//! # Husky package manager server
//!
//! ## Endpoints：
//!
//! PUT /api/v1/crates/new
//! DELETE /api/v1/crates/{crate_name}/{version}/yank
//! PUT /api/v1/crates/{crate_name}/{version}/unyank
//! GET /api/v1/crates/{crate_name}/owners
//! PUT /api/v1/crates/{crate_name}/owners
//! DELETE /api/v1/crates/{crate_name}/owners
//! GET /api/v1/crates
//! /me

mod config;
mod index;
mod storage;

use crate::config::Config;
use crate::index::Index;
use axum::body::Bytes;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Router};
use std::env::set_var;
use std::net::SocketAddr;
use std::sync::Arc;
use tempdir::TempDir;
use tokio::io::AsyncWriteExt;

async fn test() -> &'static str {
    "Hello, world!"
}

const CORGI_LOG_ENV: &'static str = "CORGI_LOG";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Config {
        ip,
        port,
        log_level,
        index_url,
        local_repo,
        packages_dir,
    } = Config::from_command_line()?;

    set_var(CORGI_LOG_ENV, &log_level);
    pretty_env_logger::init_custom_env(CORGI_LOG_ENV);

    log::info!("Cloning {} to {}", index_url, local_repo.display());
    let index = Index::clone(index_url, local_repo)?;

    let addr = SocketAddr::from((ip, port));

    let owners = Router::new()
        .route("/", get(test))
        .route("/", delete(test))
        .route("/", put(test));

    let crates = Router::new()
        .route("/new", post(new_crate))
        .route("/:crate_name/:version/yank", delete(test))
        .route("/:crate_name/:version/unyank", put(test))
        .nest("/:crate_name/owners", owners)
        .route("/", get(test));

    let router = Router::new()
        .nest("/api/v1/crates", crates)
        .route("/me", post(test))
        .layer(Extension(Arc::new(index)));

    log::info!("Server started on {}.", port);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn new_crate(
    mut req: Request<Bytes>,
    Extension(index): Extension<Arc<Index>>,
) -> impl IntoResponse {
    // 0. Get the metadata and file data of the package
    let body = req.body_mut();
    let (len, remaining) = body.split_at(4);
    let json_len = u32::from_le_bytes(len.try_into().unwrap());
    let (json, remaining) = remaining.split_at(json_len as usize);
    let new_crate = serde_json::from_slice::<crates_io::NewCrate>(json).unwrap();

    let (len, remaining) = remaining.split_at(4);
    let file_len = u32::from_le_bytes(len.try_into().unwrap());
    let (package_buffer, _remaining) = remaining.split_at(file_len as usize);

    let dir = TempDir::new("corgi")?;

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(dir.path().join("package.zip"))
        .await?;

    file.write_all(package_buffer).await?;

    // 1. validation

    // 2. move package to static directory

    // 3. update index and push to github.com

    // if let Some(body) = &req.body {
    //     // Get the metadata of the package
    //     let (len, remaining) = body.split_at(4);
    //     let json_len = u32::from_le_bytes(len.try_into().unwrap());
    //     let (json, remaining) = remaining.split_at(json_len as usize);
    //     let new_crate = serde_json::from_slice::<crates_io::NewCrate>(json).unwrap();
    //     // Get the `.crate` file
    //     let (len, remaining) = remaining.split_at(4);
    //     let file_len = u32::from_le_bytes(len.try_into().unwrap());
    //     let (file, _remaining) = remaining.split_at(file_len as usize);
    //
    //     // Write the `.crate`
    //     let dst = self
    //         .dl_path
    //         .join(&new_crate.name)
    //         .join(&new_crate.vers)
    //         .join("download");
    //     t!(fs::create_dir_all(dst.parent().unwrap()));
    //     t!(fs::write(&dst, file));
    //
    //     let deps = new_crate
    //         .deps
    //         .iter()
    //         .map(|dep| {
    //             let (name, package) = match &dep.explicit_name_in_toml {
    //                 Some(explicit) => (explicit.to_string(), Some(dep.name.to_string())),
    //                 None => (dep.name.to_string(), None),
    //             };
    //             serde_json::json!({
    //                 "name": name,
    //                 "req": dep.version_req,
    //                 "features": dep.features,
    //                 "default_features": true,
    //                 "target": dep.target,
    //                 "optional": dep.optional,
    //                 "kind": dep.kind,
    //                 "registry": dep.registry,
    //                 "package": package,
    //             })
    //         })
    //         .collect::<Vec<_>>();
    //
    //     let line = create_index_line(
    //         serde_json::json!(new_crate.name),
    //         &new_crate.vers,
    //         deps,
    //         &cksum(file),
    //         new_crate.features,
    //         false,
    //         new_crate.links,
    //         None,
    //     );
    //
    //     write_to_index(&self.registry_path, &new_crate.name, line, false);
    //
    //     self.ok(&req)
    // } else {
    //     Response {
    //         code: 400,
    //         headers: vec![],
    //         body: b"The request was missing a body".to_vec(),
    //     }
    // }
    ""
}
