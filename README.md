# Mural Server

This is the client software for mural.

Mural is a program that allows you to synchronize a wallpaper slideshow across your devices.
It supports having multiple different slideshows (called pools). For example, you might have
a pool called "Games" for wallpapers related to games and a pool called "Landscapes" for
wallpapers of beautiful landscapes.

## Installation

### From Source

```bash
cargo install --git https://github.com/mural-sync/mural-client
```

### Configuration

By default, `mural-client` looks in `~/.config/mural-client` for the configuration file.
You can change this using the `MURAL_CLIENT_HOME_HOME` environment variable.

This is the default configuration file:

```toml
server_url = "http://localhost:46666" # the url of your server. see https://github.com/mural-sync/mural-server for instructions to setup a server
pool_name = "default" # the pool to use
```
