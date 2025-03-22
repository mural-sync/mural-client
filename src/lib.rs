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
    let digest_response = reqwest::get(format!(
        "{}/api/pool/{}/digest",
        config.server_url(),
        config.pool_name()
    ))
    .await
    .map_err(|e| Error::DigestRequest(e.to_string()))?;

    if !digest_response.status().is_success() {
        return Err(Error::DigestRequest("response was not 200".to_string()));
    }

    digest_response
        .text()
        .await
        .map_err(|e| Error::DigestRequest(e.to_string()))
}

fn find_wallpaper_path(wallpapers_path: &Path, digest: &str) -> Result<Option<PathBuf>> {
    Ok(std::fs::read_dir(wallpapers_path)
        .map_err(|e| Error::WallpaperList {
            io_error: e,
            wallpapers_path: wallpapers_path.display().to_string(),
        })?
        .collect::<Result<Vec<std::fs::DirEntry>, _>>()
        .map_err(|e| Error::WallpaperList {
            io_error: e,
            wallpapers_path: wallpapers_path.display().to_string(),
        })?
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
    let wallpaper_response = reqwest::get(format!(
        "{}/api/pool/{}/wallpaper",
        config.server_url(),
        config.pool_name()
    ))
    .await
    .map_err(|e| Error::WallpaperRequest(e.to_string()))?;

    if !wallpaper_response.status().is_success() {
        return Err(Error::WallpaperRequest("response was not 200".to_string()));
    }

    let content_type = wallpaper_response
        .headers()
        .get("Content-Type")
        .expect("should always have a Content-Type")
        .to_str()
        .expect("content-type should always be a valid &str")
        .to_string();
    let extension = content_type
        .split("/")
        .nth(1)
        .expect("content-type should always contain a slash");
    let wallpaper_path = wallpapers_path.join(format!("{}.{}", digest, extension));

    let image_content = wallpaper_response
        .bytes()
        .await
        .map_err(|e| Error::WallpaperRequest(e.to_string()))?;

    std::fs::write(&wallpaper_path, image_content).map_err(Error::WallpaperWrite)?;

    Ok(wallpaper_path)
}

fn set_wallpaper(wallpaper_path: &Path) -> Result<()> {
    let exit_status = std::process::Command::new("swww")
        .arg("img")
        .arg(wallpaper_path)
        .spawn()
        .map_err(Error::WallpaperSetCommand)?
        .wait()
        .map_err(Error::WallpaperSetCommand)?;
    if !exit_status.success() {
        // TODO: include stderr in error message
        return Err(Error::WallpaperSet {
            exit_code: exit_status.code().unwrap_or_default(),
        });
    }

    Ok(())
}

pub async fn run() -> Result<()> {
    env::load_dotenv()?;
    let config = Config::load()?;

    let data_home_path = xdg::BaseDirectories::with_prefix("mural-client")
        .map_err(|_| Error::DataHome)?
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
