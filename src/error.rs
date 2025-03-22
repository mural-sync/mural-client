#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("requesting the current wallpaper failed: {0}")]
    WallpaperRequest(reqwest::Error),
    #[error("failed to write the downloaded wallaper to disk: {0}")]
    WallpaperWrite(std::io::Error),

    #[error("executing the command to set the wallpaper failed: {0}")]
    WallpaperSetCommand(std::io::Error),
    #[error("setting the wallpaper failed with exit code {exit_code}")]
    WallpaperSet { exit_code: i32 },

    #[error("requesting the interval failed: {0}")]
    IntervalRequest(reqwest::Error),
    #[error("the server returned an invalid interval")]
    InvalidInterval,

    #[error("requesting the current digest failed: {0}")]
    DigestRequest(reqwest::Error),

    #[error("failed load a .env file: '{line_content}' on line {line_number} is invalid")]
    DotenvyParse {
        line_content: String,
        line_number: usize,
    },
    #[error("failed to read a .env file: {0}")]
    DotenvyIo(std::io::Error),

    #[error("failed to find a config directory")]
    ConfigHome,
    #[error("failed to parse the configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("failed to read the configuration: {0}")]
    ConfigRead(std::io::Error),

    #[error("failed to find a data directory")]
    DataHome,

    #[error("failed to list locally available wallpapers in '{wallpapers_path}': {io_error}")]
    WallpaperList {
        io_error: std::io::Error,
        wallpapers_path: String,
    },

    #[error("{0}")]
    Custom(String),
}
