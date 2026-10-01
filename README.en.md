[日本語](README.md) | English | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md)

<div align="center">
  <h1><strong>colmsg</strong></h1>
  <img src="https://user-images.githubusercontent.com/3148511/158018437-09822a33-8767-4e03-ba90-e0f69594c493.jpeg" width="32px" alt="Sakurazaka46 Message logo"><img src="https://user-images.githubusercontent.com/3148511/158018441-dd7cb9eb-bf31-4938-830d-1ef293a2afba.jpg" width="32px" alt="Hinatazaka46 Message logo"><img src="https://user-images.githubusercontent.com/3148511/158018442-ae54e926-760d-4b47-b0a0-7255485e1f28.jpg" width="32px" alt="Nogizaka46 Message logo">

  Save messages from the Sakurazaka46 Message, Hinatazaka46 Message, Nogizaka46 Message, Asuka Saito Message, Mai Shiraishi Message, and yodel apps to your computer.

  ![Demo](doc/demo/colmsg.gif)
</div>

## Getting started

Install `colmsg` by following [Installation](#installation).
On Windows, use `colmsg.exe` instead of `colmsg`.

Complete the login account setup in the app beforehand.  
Log in once for each service.

```sh
colmsg login sakurazaka
```

Available service names are `sakurazaka`, `hinatazaka`, `nogizaka`, `asukasaito`, `maishiraishi`, and `yodel`.
Save the full message history for all subscribed members.

```sh
colmsg
```

## Features

* ✅ No device rooting required
* ✅ The workflow can be completed entirely with colmsg
* ✅ Runs on Windows, macOS, and Linux
* ✅ Offers several ways to filter and save messages

## Message saving options

You can use options to choose what to save.

Save messages from specific members:

```sh
colmsg -n 菅井友香 -n 佐々木久美
```

Save messages from a specific group:

```sh
colmsg -g sakurazaka
```

Save specific message types:

```sh
colmsg -k picture -k video
```

Save messages from a specific date onward:

```sh
colmsg -F '2020/01/01 00:00:00'
```

By default, messages for 4 members are saved concurrently per service. Use `--jobs` (`-j`) to set a different concurrency.

```sh
colmsg --jobs 2
```

You can combine options. Run `colmsg --help` for details.

## Login

The browser used for login is detected automatically. To select a Chromium-based browser such as Chrome, Brave, or Edge, or if automatic detection fails, specify the browser executable path with `--browser`.

```sh
colmsg login sakurazaka --browser '/Applications/Brave Browser.app/Contents/MacOS/Brave Browser'
```

You can also set the `COLMSG_BROWSER` environment variable; `--browser` takes precedence.

After a successful login, credentials are saved in `auth/<service>.json` under the configuration directory.
You can check the saved authentication method, account, and expiry.

```sh
colmsg auth status
```

## Details

* When run after messages have already been saved, `colmsg` fetches messages after the latest saved message.
* Messages are saved in a directory structure like this:
  ```text
  colmsg/
  ├── 日向坂46 一期生
  │   └── 佐々木久美
  │       └── 1_0_20191231235959_佐々木久美.txt
  ├── 乃木坂46
  │   └── 秋元真夏
  │       └── 2_1_20200101000000_秋元真夏.jpg
  └── 櫻坂46 一期生
      └── 菅井友香
          ├── 3_2_20200101000001_菅井友香.mp4
          └── 4_3_20200101000002_菅井友香.mp4
  ```
* File names use the format `<sequence>_<type>_<date>_<poster-name>.<extension>`.
  * The sequence number represents the chronological order of messages. The portion before the poster's name retains its order, so sorting files lexicographically in a file browser displays messages chronologically.
  * If the poster cannot be identified, `unknown` is used.
  * The type numbers are:
    * 0: Text
    * 1: Picture
    * 2: Video
    * 3: Voice
    * 4: Link
* Run `colmsg --download-dir` to see the download directory resolved from the configuration file and command-line options.
* Already-saved messages are not overwritten.

## Configuration file

You can set default options in a configuration file. Run `colmsg --config-path` to see the path to the configuration file currently used. You can also set the configuration file path with `COLMSG_CONFIG_PATH`:

```sh
export COLMSG_CONFIG_PATH="/path/to/colmsg.conf"
```

### Format

The configuration file is a simple list of command-line arguments. Run `colmsg --help` to see available options and values. Use `#` to add comments.

Example:

```text
# Save only Sakurazaka messages
-g sakurazaka

# Save only media files
-k picture -k video -k voice

# Set concurrent member saves per service to 6
--jobs 6
```

## Legacy refresh_token authentication (deprecated)

The `refresh_token` method is **deprecated**. For new setups and migration, use Cookie authentication with `colmsg login <service>`.

Only when using the legacy method, consult the [old acquisition instructions (Japanese)](doc/how_to_get_refresh_token.md) and put the tokens for each service in the configuration file.

Legacy configuration example (deprecated; include only the services you need):

```text
# Set s_refresh_token
--s_refresh_token s_refresh_token

# Set h_refresh_token
--h_refresh_token h_refresh_token

# Set n_refresh_token
--n_refresh_token n_refresh_token

# Set a_refresh_token
--a_refresh_token a_refresh_token

# Set m_refresh_token
--m_refresh_token m_refresh_token

# Set y_refresh_token
--y_refresh_token y_refresh_token
```

## Installation

### Windows

Download the ZIP archive containing the prebuilt Windows executable from the [releases page](https://github.com/proshunsuke/colmsg/releases).
Extract it with a tool such as [7-Zip](https://sevenzip.osdn.jp/).
The extracted executable is `colmsg.exe`.
Run it from [PowerShell](https://docs.microsoft.com/ja-jp/powershell/) or another terminal.

### macOS

Install with Homebrew:

```sh
brew tap proshunsuke/colmsg
brew install colmsg
```

### Arch Linux

Install from the [AUR](https://aur.archlinux.org/packages/colmsg/):

```sh
yay -S colmsg
```

### Binaries

Prebuilt binaries for other architectures are available on the [releases page](https://github.com/proshunsuke/colmsg/releases).

## Development

Run tests with `make test`. To test all features, run `make test-all-features`.
Format Rust code with `make fmt`; check formatting with `make fmt-check`.

For local API development, you can start the OpenAPI mock servers:

```sh
make server/kh
make server/n46
```

Set `S_BASE_URL`, `H_BASE_URL`, and `N_BASE_URL` to route requests to the mock servers. For example:

```sh
S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/ --help
```

## License

`colmsg` is distributed under the MIT License. See [LICENSE](LICENSE.txt) for details.
