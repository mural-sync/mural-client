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

pub async fn run() -> Result<()> {
    env::load_dotenv()?;
    let config = Config::load()?;
    info!("using configuration {:?}", &config);

    let mut last_digest = String::new();

    loop {
        info!("updating wallpaper");

        let current_digest = current_digest(&config).await?;
        if current_digest == last_digest {
            info!("the wallpaper did not change; skipping");
        } else {
            info!("setting a new wallpaper");

            last_digest = current_digest;
        }

        let delay = delay_until_next_update(&config).await?;
        std::thread::sleep(std::time::Duration::from_secs(
            delay.get_seconds() as u64 + 1,
        ));
    }
}
