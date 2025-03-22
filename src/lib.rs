mod config;
pub(crate) use config::Config;

mod env;

mod error;
pub(crate) use error::Error;

pub(crate) mod prelude;
use prelude::*;

pub async fn run() -> Result<()> {
    env::load_dotenv()?;
    let config = Config::load()?;
    info!("using configuration {:?}", &config);

    loop {
        info!("updating wallpaper");

        let interval = reqwest::get(format!("{}/api/interval", config.server_url()))
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let current_timestamp = jiff::Timestamp::now();
        let next_timestamp = jiff::Timestamp::from_second(
            ((current_timestamp.as_second() as f64 / interval as f64) + 1.0).floor() as i64
                * interval as i64,
        )
        .expect("calculation should always succeed");
        let span = next_timestamp - current_timestamp;

        info!("next update at {}", next_timestamp);
        info!("delaying for {} seconds", span);

        std::thread::sleep(std::time::Duration::from_secs(
            span.get_seconds() as u64 + 1,
        ));
    }

    Ok(())
}
