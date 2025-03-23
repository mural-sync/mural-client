use std::path::PathBuf;

use clap::{Arg, Command, value_parser};

pub fn get_command() -> Command {
    Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::new("config-dir")
                .long("config-dir")
                .value_parser(value_parser!(PathBuf))
                .help("provide a custom config directory"),
        )
}
