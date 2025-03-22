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

pub async fn run() -> Result<()> {
    env::load_dotenv()?;
    let config = Config::load()?;
    info!("using configuration {:?}", &config);

    loop {
        info!("updating wallpaper");

        let delay = delay_until_next_update(&config).await?;
        std::thread::sleep(std::time::Duration::from_secs(
            delay.get_seconds() as u64 + 1,
        ));
    }
}
