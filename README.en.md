[日本語](README.md) | English | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md)

> [!WARNING]
> iOS users currently cannot obtain a `refresh_token`. This is caused by an update to the messaging apps, not by `colmsg`, and there is currently no complete workaround. Android users can follow the [Japanese instructions](doc/how_to_get_refresh_token.md#android%E3%82%A2%E3%83%97%E3%83%AA%E3%81%AE%E5%A0%B4%E5%90%88).

<div align="center">
  <h1><strong>colmsg</strong></h1>
  <img src="https://user-images.githubusercontent.com/3148511/158018437-09822a33-8767-4e03-ba90-e0f69594c493.jpeg" width="32px" alt="Sakurazaka46 Message logo"><img src="https://user-images.githubusercontent.com/3148511/158018441-dd7cb9eb-bf31-4938-830d-1ef293a2afba.jpg" width="32px" alt="Hinatazaka46 Message logo"><img src="https://user-images.githubusercontent.com/3148511/158018442-ae54e926-760d-4b47-b0a0-7255485e1f28.jpg" width="32px" alt="Nogizaka46 Message logo">

  Save messages from the Sakurazaka46 Message, Hinatazaka46 Message, Nogizaka46 Message, Asuka Saito Message, Mai Shiraishi Message, and yodel apps to your computer.

  ![Demo](https://user-images.githubusercontent.com/3148511/158026220-90735546-2401-40ca-a9e6-89d2176ad3b4.gif)
</div>

## Overview

See [Installation](#installation) for how to install `colmsg`.

First, obtain a `refresh_token`. The instructions are [here (Japanese)](doc/how_to_get_refresh_token.md).

Then run the following command. Replace each placeholder with the `refresh_token` from its corresponding app: Sakurazaka46 Message, Hinatazaka46 Message, Nogizaka46 Message, Asuka Saito Message, Mai Shiraishi Message, or yodel. You only need to provide tokens for apps to which you subscribe.

This saves the full message history for all subscribed members.

```fish
colmsg --s_refresh_token <s_refresh_token> --h_refresh_token <h_refresh_token> --n_refresh_token <n_refresh_token> --a_refresh_token <a_refresh_token> --m_refresh_token <m_refresh_token> --y_refresh_token <y_refresh_token>
```

On Windows, use `colmsg.exe` instead of `colmsg`.

## Features

* ✅ No device rooting required
* ✅ Works with both Android and iOS apps
* ✅ Runs on Windows, macOS, and Linux
* ✅ Offers several ways to filter and save messages
* ✅ Supports the following app versions:
  * Sakurazaka46 Message: 1.12.01.169
  * Hinatazaka46 Message: 2.13.01.169
  * Nogizaka46 Message: 1.8.01.169
  * Asuka Saito Message: 1.1.01.169
  * Mai Shiraishi Message: 3.4.3.426
  * yodel: 4.1.1.455

## Usage

The overview shows the basic usage. Since a `refresh_token` is sensitive, avoid entering it directly in the terminal. We recommend setting it in a configuration file. See [Configuration file](#configuration-file). The examples below assume that the tokens are configured there.

You can use options to choose what to save.

Save messages from specific members:

```fish
colmsg -n 菅井友香 -n 佐々木久美
```

Save messages from a specific group:

```fish
colmsg -g sakurazaka
```

Save specific message types:

```fish
colmsg -k picture -k video
```

Save messages from a specific date onward:

```fish
colmsg -F '2020/01/01 00:00:00'
```

You can combine options. Run `colmsg --help` for details.

## Details

* When run after messages have already been saved, `colmsg` fetches messages after the latest saved message.
* Messages are saved in a directory structure like this:
  ```text
  colmsg/
  ├── 日向坂46 一期生
  │   └── 佐々木久美
  │       └── 1_0_20191231235959.txt
  ├── 乃木坂46
  │   └── 秋元真夏
  │       └── 2_1_20200101000000.jpg
  └── 櫻坂46 一期生
      └── 菅井友香
          ├── 3_2_20200101000001.mp4
          └── 4_3_20200101000002.mp4
  ```
* File names use the format `<sequence>_<type>_<date>.<extension>`. Sequence numbers sort messages chronologically. The type number is:
  * 0: Text
  * 1: Picture
  * 2: Video
  * 3: Voice
  * 4: Link
* Run `colmsg --download-dir` to see the default download directory for your platform.
* Already-saved messages are not overwritten.

## Configuration file

You can set default options in a configuration file. Run `colmsg --config-dir` to see its default directory. You can also set the configuration file path with `COLMSG_CONFIG_PATH`:

```fish
set -gx COLMSG_CONFIG_PATH /path/to/colmsg.conf
```

### Format

The configuration file is a list of command-line arguments. Run `colmsg --help` to see available options. Lines beginning with `#` are comments.

Example:

```text
# Sakurazaka46 refresh token
--s_refresh_token s_refresh_token

# Hinatazaka46 refresh token
--h_refresh_token h_refresh_token

# Nogizaka46 refresh token
--n_refresh_token n_refresh_token

# Asuka Saito refresh token
--a_refresh_token a_refresh_token

# Mai Shiraishi refresh token
--m_refresh_token m_refresh_token

# yodel refresh token
--y_refresh_token y_refresh_token

# Save only media files
-k picture -k video -k voice
```

## Installation

### Windows

Download the Windows archive from the [releases page](https://github.com/proshunsuke/colmsg/releases) and extract it with a tool such as [7-Zip](https://www.7-zip.org/). The archive contains `colmsg.exe`, which you can run from PowerShell or another terminal.

### macOS

Install with Homebrew:

```fish
brew tap proshunsuke/colmsg
brew install colmsg
```

### Arch Linux

Install from the [AUR](https://aur.archlinux.org/packages/colmsg/):

```fish
yay -S colmsg
```

### Binaries

Prebuilt binaries for other architectures are available on the [releases page](https://github.com/proshunsuke/colmsg/releases).

## Development

Run tests with `make test`. To test all features, run `make test-all-features`.
Format Rust code with `make fmt`; check formatting with `make fmt-check`.

For local API development, you can start the OpenAPI mock servers:

```fish
make server/kh
make server/n46
```

Set `S_BASE_URL`, `H_BASE_URL`, and `N_BASE_URL` to route requests to the mock servers. For example:

```fish
env S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/ --help
```

## TODO

* [ ] Provide examples
* [ ] Parallelize message saving
* [ ] Extract the API client into a crate

## License

`colmsg` is distributed under the MIT License. See [LICENSE](LICENSE.txt) for details.
