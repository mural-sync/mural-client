use crate::prelude::*;

pub fn load_dotenv() -> Result<()> {
    const TARGET: &str = "mural_client::load_dotenv";

    info!(target: TARGET, "loading a .env file");

    match dotenvy::dotenv() {
        Ok(file_path) => {
            info!(target: TARGET, "successfully loaded '{}'", file_path.display());
        }
        Err(dotenvy::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            info!(target: TARGET, "could not find a .env file; skipping");
        }
        Err(dotenvy::Error::Io(e)) => return Err(Error::DotenvyIo(e)),
        Err(dotenvy::Error::LineParse(line_content, line_number)) => {
            return Err(Error::DotenvyParse {
                line_content,
                line_number,
            });
        }
        Err(dotenvy::Error::EnvVar(_)) => {
            unreachable!("this error is never returned from dotenvy::dotenv")
        }
        Err(_) => todo!(),
    }

    Ok(())
}
