use git2::Repository;
use std::path::{Path, PathBuf};
use url::Url;

// https://github.com/ancient-software/husky-packages.io.git
pub struct Index {
    url: Url,
    local: PathBuf,
    repository: Repository,
}

impl Index {
    pub fn clone(url: Url, local: impl AsRef<Path>) -> anyhow::Result<Self> {
        let local = local.as_ref().to_path_buf();
        let repository = Repository::clone(&url.to_string(), &local)?;
        Ok(Self {
            url,
            local,
            repository,
        })
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn local(&self) -> &PathBuf {
        &self.local
    }
    pub fn repository(&self) -> &Repository {
        &self.repository
    }
}
