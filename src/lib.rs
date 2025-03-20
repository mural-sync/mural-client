use std::path::{Path, PathBuf};

const SERVER_URL: &str = "http://localhost:8080";
const POOL: &str = "firewatch";

#[cfg(target_os = "linux")]
fn set_wallpaper<P: AsRef<Path>>(file_path: P) -> Result<(), anyhow::Error> {
    tracing::info!("setting the wallpaper");

    let file_path = file_path.as_ref().to_path_buf();

	std::process::Command::new("swww")
    	.arg("img")
    	.arg(file_path)
    	.spawn().unwrap();

    Ok(())
}

async fn get_current_digest(pool_name: &str) -> Result<String, anyhow::Error> {
    tracing::info!("getting current digest");
    Ok(reqwest::get(format!(
        "{}/pool/digest?pool_name={}",
        SERVER_URL, pool_name
    ))
    .await?
    .text()
    .await?)
}

async fn get_local_wallpaper(
    base_dirs: &xdg::BaseDirectories,
    digest: &str,
) -> Result<Option<PathBuf>, anyhow::Error> {
    let wallpapers_path = base_dirs.get_data_home().join("wallpapers");

    for file_path in std::fs::read_dir(wallpapers_path)? {
        let file_path = file_path?.path();
        let file_stem = file_path.file_stem().unwrap().to_string_lossy().to_string();
        if file_stem == digest {
            tracing::info!("the wallpaper is already available locally");
            return Ok(Some(file_path));
        }
    }

    return Ok(None);
}

async fn update_wallpaper(
    base_dirs: &xdg::BaseDirectories,
    pool_name: &str,
    last_digest: &str,
) -> Result<String, anyhow::Error> {
    tracing::info!("updating wallpaper");

    let digest = get_current_digest(pool_name).await?;

    if digest == last_digest {
        tracing::info!("wallpaper did not change");
        return Ok(digest);
    }

    let file_path = match get_local_wallpaper(&base_dirs, &digest).await? {
        Some(file_path) => file_path,
        None => {
            tracing::info!("downloading the wallpaper");
            let wallpaper_response =
                reqwest::get(format!("{}/pool/wallpaper?pool_name={}", SERVER_URL, POOL)).await?;

            let content_type = wallpaper_response
                .headers()
                .get("Content-Type")
                .unwrap()
                .to_str()?
                .to_string();
            let file_type = content_type.split("/").nth(1).unwrap();

            let image_content = wallpaper_response.bytes().await?;

            let file_path = base_dirs
                .get_data_home()
                .join("wallpapers")
                .join(format!("{}.{}", digest, file_type));
            std::fs::write(&file_path, image_content)?;
            file_path
        }
    };

    set_wallpaper(file_path)?;

    Ok(digest)
}

pub async fn run() -> Result<(), anyhow::Error> {
    let base_dirs = xdg::BaseDirectories::with_prefix("mural_client")?;
    std::fs::create_dir_all(base_dirs.get_data_home().join("wallpapers"))?;

    let mut last_digest = String::new();

    loop {
        last_digest = update_wallpaper(&base_dirs, POOL, &last_digest).await?;

        std::thread::sleep(std::time::Duration::from_secs(600));
    }
}
