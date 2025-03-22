mod config;
pub(crate) use config::Config;

mod env;

mod error;
pub(crate) use error::Error;

pub(crate) mod prelude;
use prelude::*;

pub fn run() -> Result<()> {
    env::load_dotenv()?;

    let config = Config::load()?;
    info!("using configuration {:?}", &config);

    Ok(())
}
