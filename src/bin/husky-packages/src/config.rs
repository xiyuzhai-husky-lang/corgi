use clap::Parser;
use config::Environment;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use url::Url;

/// Configuration
///
/// ./husky-packages -c config.toml
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ip: IpAddr,
    pub port: u16,
    pub log_level: String,

    pub index_url: Url,
    pub local_repo: PathBuf,

    /// Local directory for packages
    pub packages_dir: PathBuf,
}

impl Config {
    pub fn from_command_line() -> anyhow::Result<Config> {
        let Opts {
            mut configs,
            config_root_path,
        } = Opts::parse();

        if configs.is_empty() {
            anyhow::bail!(
                "Please use the command line option '-c' to set the configuration file path"
            );
        }

        let mut config = config::Config::builder();

        for conf in configs.drain(..).rev() {
            config = config.add_source(config::File::from(match &config_root_path {
                None => conf,
                Some(root) => root.join(conf),
            }));
        }

        config = config.add_source(Environment::with_prefix("HUSKY_PACKAGES").separator("__"));

        Ok(config.build()?.try_deserialize()?)
    }
}

/// husky-packages launcher
#[derive(Debug, Parser)]
#[clap(name = "husky-packages launcher")]
pub struct Opts {
    /// Config files
    #[clap(short = 'c', long)]
    pub configs: Vec<PathBuf>,

    /// Config file root path
    #[clap(short = 'r', long)]
    pub config_root_path: Option<PathBuf>,
}
