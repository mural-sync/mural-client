use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

mod config;
pub(crate) use config::Config;

mod env;

mod error;
pub(crate) use error::Error;

pub(crate) mod prelude;
use prelude::*;

async fn delay_until_next_update(config: &Config) -> Result<jiff::Span> {
    let interval = reqwest::get(format!("{}/api/interval", config.server_url()))
        .await
        .map_err(Error::IntervalRequest)?
        .text()
        .await
        .map_err(Error::IntervalRequest)?
        .parse::<u64>()
        .map_err(|_| Error::InvalidInterval)?;

    let current_timestamp = jiff::Timestamp::now();
    let next_timestamp = jiff::Timestamp::from_second(
        ((current_timestamp.as_second() as f64 / interval as f64) + 1.0).floor() as i64
            * interval as i64,
    )
    .expect("calculation should always succeed");
    Ok(next_timestamp - current_timestamp)
}

async fn current_digest(config: &Config) -> Result<String> {
    // TODO: handle non-200 response
    reqwest::get(format!(
        "{}/api/pool/{}/digest",
        config.server_url(),
        config.pool_name()
    ))
    .await
    .map_err(Error::DigestRequest)?
    .text()
    .await
    .map_err(Error::DigestRequest)
}

fn find_wallpaper_path(wallpapers_path: &Path, digest: &str) -> Result<Option<PathBuf>> {
    Ok(std::fs::read_dir(wallpapers_path)
        .unwrap()
        .collect::<Result<Vec<std::fs::DirEntry>, _>>()
        .unwrap()
        .iter()
        .map(|dir_entry| dir_entry.path())
        .find(|wallpaper_path| {
            wallpaper_path
                .file_stem()
                .map(|file_stem| file_stem == OsStr::new(digest))
                .unwrap_or(false)
        }))
}

async fn download_current_wallpaper(
    wallpapers_path: &Path,
    config: &Config,
    digest: &str,
) -> Result<PathBuf> {
    info!("downloading current wallpaper");
    // TODO: handle non-200 response
    let wallpaper_response = reqwest::get(format!(
        "{}/api/pool/{}/wallpaper",
        config.server_url(),
        config.pool_name()
    ))
    .await
    .unwrap();

    let content_type = wallpaper_response
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let extension = content_type.split("/").nth(1).unwrap();
    let wallpaper_path = wallpapers_path.join(format!("{}.{}", digest, extension));

    let image_content = wallpaper_response.bytes().await.unwrap();

    std::fs::write(&wallpaper_path, image_content).unwrap();

    Ok(wallpaper_path)
}

fn set_wallpaper(wallpaper_path: &Path) -> Result<()> {
    // TODO: handle non-zero exit status
    let _exit_status = std::process::Command::new("swww")
        .arg("img")
        .arg(wallpaper_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    Ok(())
}

pub async fn run() -> Result<()> {
    env::load_dotenv()?;
    let config = Config::load()?;

    let data_home_path = xdg::BaseDirectories::with_prefix("mural-client")
        .unwrap()
        .get_data_home();
    let wallpapers_path = data_home_path.join("wallpapers");

    let mut last_digest = String::new();

    loop {
        info!("updating wallpaper");

        let current_digest = current_digest(&config).await?;
        if current_digest == last_digest {
            info!("the wallpaper did not change; skipping");
        } else {
            let wallpaper_path = match find_wallpaper_path(&wallpapers_path, &current_digest)? {
                Some(wallpaper_path) => wallpaper_path,
                None => {
                    download_current_wallpaper(&wallpapers_path, &config, &current_digest).await?
                }
            };
            info!("setting a new wallpaper");
            set_wallpaper(&wallpaper_path)?;
            last_digest = current_digest;
        }

        let delay = delay_until_next_update(&config).await?;
        std::thread::sleep(std::time::Duration::from_secs(
            delay.get_seconds() as u64 + 1,
        ));
    }
}
