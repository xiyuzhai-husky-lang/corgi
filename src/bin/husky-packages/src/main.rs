//! # Husky package manager server
//!
//! ## Endpoints：
//!
//! PUT /api/v1/packages/new
//! DELETE /api/v1/packages/{package_name}/{version}/yank
//! PUT /api/v1/packages/{package_name}/{version}/unyank
//! GET /api/v1/packages/{package_name}/owners
//! PUT /api/v1/packages/{package_name}/owners
//! DELETE /api/v1/packages/{package_name}/owners
//! GET /api/v1/packages
//! /me

mod config;
mod error;
mod index;
mod storage;

use crate::config::Config;
// use crate::index::Index;
use crate::error::Error;
use axum::body::Bytes;
use axum::http::Request;
use axum::routing::{delete, get, post, put};
use axum::Router;
use std::env::set_var;
use std::io::{Seek, SeekFrom};
use std::net::SocketAddr;
use tempdir::TempDir;
use tokio::io::AsyncWriteExt;

async fn test() -> &'static str {
    "Hello, world!"
}

const CORGI_LOG_ENV: &'static str = "CORGI_LOG";
const CORGI_TOML: &'static str = "Corgi.toml";

#[tokio::main]
async fn main() -> error::Result<()> {
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
    //
    // log::info!("Cloning {} to {}", index_url, local_repo.display());
    // let index = Index::clone(index_url, local_repo)?;

    let addr = SocketAddr::from((ip, port));

    let owners = Router::new()
        .route("/", get(test))
        .route("/", delete(test))
        .route("/", put(test));

    let packages = Router::new()
        .route("/new", post(new_package))
        .route("/:package_name/:version/yank", delete(test))
        .route("/:package_name/:version/unyank", put(test))
        .nest("/:package_name/owners", owners)
        .route("/", get(test));

    let router = Router::new()
        .nest("/api/v1/packages", packages)
        .route("/me", post(test));
    // .layer(Extension(Arc::new(index)));

    log::info!("Server started on {}.", port);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn new_package(body: Bytes) -> error::Result<String> {
    // 0. Get the metadata and file data of the package
    if body.len() <= 4 {
        return Error::custom("The body is too short");
    }

    let (len, remaining) = body.split_at(4);
    let json_len = u32::from_le_bytes(len.try_into().unwrap());
    let (json, remaining) = remaining.split_at(json_len as usize);
    let new_package = serde_json::from_slice::<crates_io::NewPackage>(json)?;
    log::info!("{} - {}", new_package.name, new_package.vers);

    if remaining.len() <= 4 {
        return Error::custom("The body is too short");
    }

    let (len, remaining) = remaining.split_at(4);
    let file_len = u32::from_le_bytes(len.try_into().unwrap());
    let (package_buffer, _remaining) = remaining.split_at(file_len as usize);

    let dir = TempDir::new("corgi")?;
    let package_path = dir.path().join("package.zip");
    let extracted = dir.path().join("extracted");

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&package_path)
        .await?;

    file.write_all(package_buffer).await?;

    let mut file = file.into_std().await; // async
    let ep = extracted.clone();
    tokio::task::spawn_blocking(move || {
        file.seek(SeekFrom::Start(0))?;
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(ep)?;

        Ok::<(), error::Error>(())
    })
    .await??;

    // 1. validation
    let corgi_toml = tokio::fs::read_to_string(extracted.join(CORGI_TOML)).await?;

    // 2. move package to static directory

    // 3. update index and push to github.com

    // if let Some(body) = &req.body {
    //     // Get the metadata of the package
    //     let (len, remaining) = body.split_at(4);
    //     let json_len = u32::from_le_bytes(len.try_into().unwrap());
    //     let (json, remaining) = remaining.split_at(json_len as usize);
    //     let new_crate = serde_json::from_slice::<packages_io::NewCrate>(json).unwrap();
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
    Ok("".into())
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use crates_io::NewPackage;

    #[tokio::test]
    async fn create_package() {
        let client = reqwest::Client::default();

        let mut body = BytesMut::new();

        let new_package = NewPackage {
            name: "hello".to_string(),
            vers: "0.1.0".to_string(),
            deps: vec![],
            features: Default::default(),
            authors: vec![],
            description: None,
            documentation: None,
            homepage: None,
            readme: None,
            readme_file: None,
            keywords: vec![],
            categories: vec![],
            license: None,
            license_file: None,
            repository: None,
            badges: Default::default(),
            links: None,
        };

        let json = serde_json::to_string(&new_package).unwrap();
        let len = json.len() as u32;

        body.put_u32_le(len);
        body.put_slice(json.as_bytes());

        let response = client
            .post("http://localhost:3000/api/v1/packages/new")
            .body(body.freeze())
            .send()
            .await
            .unwrap();

        println!("{}", response.status());
        println!("{:?}", response.bytes().await.unwrap());
    }
}
