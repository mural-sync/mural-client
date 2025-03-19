use std::path::Path;

const SERVER_URL: &str = "http://localhost:8080";
const POOL: &str = "firewatch";

#[cfg(target_os = "linux")]
fn set_wallpaper<P: AsRef<Path>>(file_path: P) -> Result<(), anyhow::Error> {
    let file_path = file_path.as_ref().to_path_buf();

    std::process::Command::new("swaybg")
        .arg("--image")
        .arg(file_path)
        .spawn()?;

    Ok(())
}

pub async fn run() -> Result<(), anyhow::Error> {
    let digest = reqwest::get(format!("{}/pool/digest?pool_name={}", SERVER_URL, POOL))
        .await?
        .text()
        .await?;

    let wallpaper_response = reqwest::get(format!("{}/pool/wallpaper?pool_name={}", SERVER_URL, POOL))
        .await?;

    let content_type = wallpaper_response.headers().get("Content-Type").unwrap().to_str()?.to_string();
    let file_type = content_type.split("/").nth(1).unwrap();

    let image_content = wallpaper_response.bytes().await?;
    std::fs::write(format!("{}.{}", digest, file_type), image_content)?;

    Ok(())
}
