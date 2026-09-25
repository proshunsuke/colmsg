[日本語](README.md) | [English](README.en.md) | 简体中文 | [繁體中文](README.zh-Hant.md)

> [!WARNING]
> 目前 iOS 无法获取 `refresh_token`。这是消息应用更新导致的问题，并非 `colmsg` 本身的问题，目前没有完整的解决办法。Android 用户可参考[此处的日文说明](doc/how_to_get_refresh_token.md#android%E3%82%A2%E3%83%97%E3%83%AA%E3%81%AE%E5%A0%B4%E5%90%88)。

<div align="center">
  <h1><strong>colmsg</strong></h1>
  <img src="https://user-images.githubusercontent.com/3148511/158018437-09822a33-8767-4e03-ba90-e0f69594c493.jpeg" width="32px" alt="樱坂46 Message 标志"><img src="https://user-images.githubusercontent.com/3148511/158018441-dd7cb9eb-bf31-4938-830d-1ef293a2afba.jpg" width="32px" alt="日向坂46 Message 标志"><img src="https://user-images.githubusercontent.com/3148511/158018442-ae54e926-760d-4b47-b0a0-7255485e1f28.jpg" width="32px" alt="乃木坂46 Message 标志">

  将「樱坂46 Message」「日向坂46 Message」「乃木坂46 Message」「斋藤飞鸟 Message」「白石麻衣 Message」和 yodel 应用中的消息保存到电脑。

  ![演示](https://user-images.githubusercontent.com/3148511/158026220-90735546-2401-40ca-a9e6-89d2176ad3b4.gif)
</div>

## 概述

安装方法请参阅[安装](#安装)。

首先获取 `refresh_token`。获取方法请参阅[此处的日文说明](doc/how_to_get_refresh_token.md)。

然后运行以下命令，并将占位符替换为对应应用取得的 `refresh_token`：樱坂46 Message、日向坂46 Message、乃木坂46 Message、斋藤飞鸟 Message、白石麻衣 Message 和 yodel。只需填写已订阅应用的令牌。

这会保存所有已订阅成员的完整历史消息。

```fish
colmsg --s_refresh_token <s_refresh_token> --h_refresh_token <h_refresh_token> --n_refresh_token <n_refresh_token> --a_refresh_token <a_refresh_token> --m_refresh_token <m_refresh_token> --y_refresh_token <y_refresh_token>
```

在 Windows 上，请将 `colmsg` 替换为 `colmsg.exe`。

## 功能

* ✅ 无需 root 设备
* ✅ 支持 Android 和 iOS 应用
* ✅ 可在 Windows、macOS 和 Linux 上运行
* ✅ 支持多种消息筛选和保存方式
* ✅ 支持以下应用版本：
  * 樱坂46 Message：1.12.01.169
  * 日向坂46 Message：2.13.01.169
  * 乃木坂46 Message：1.8.01.169
  * 斋藤飞鸟 Message：1.1.01.169
  * 白石麻衣 Message：3.4.3.426
  * yodel：4.1.1.455

## 使用方法

概述介绍了基本用法。由于 `refresh_token` 属于敏感信息，不建议直接在终端中输入。建议将其写入配置文件，详情请参阅[配置文件](#配置文件)。以下示例假定令牌已写入配置文件。

可通过选项选择要保存的内容。

保存指定成员的消息：

```fish
colmsg -n 菅井友香 -n 佐々木久美
```

保存指定组合的消息：

```fish
colmsg -g sakurazaka
```

保存指定类型的消息：

```fish
colmsg -k picture -k video
```

保存指定日期之后的消息：

```fish
colmsg -F '2020/01/01 00:00:00'
```

默认情况下，每个服务会并行保存4名成员的消息。可使用 `--jobs`（`-j`）更改并行数量。

```fish
colmsg --jobs 2
```

选项可以组合使用。运行 `colmsg --help` 查看详细说明。

## 详细信息

* 如果已经保存过消息，运行 `colmsg` 时会从最后一条已保存消息之后继续获取。
* 消息按以下目录结构保存：
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
* 文件名格式为 `<序号>_<类型>_<日期>_<发布者姓名>.<扩展名>`。序号和日期前缀保持不变，因此文件仍按时间顺序排列。无法确定发布者时使用 `unknown`。类型编号如下：
  * 0：文本
  * 1：图片
  * 2：视频
  * 3：语音
  * 4：链接
* 运行 `colmsg --download-dir` 可查看根据配置文件和命令行选项确定的下载目录。
* 已保存的消息不会被覆盖。

## 配置文件

可以在配置文件中预先设置默认选项。运行 `colmsg --config-path` 可查看当前使用的配置文件路径。也可以通过 `COLMSG_CONFIG_PATH` 指定配置文件路径：

```fish
set -gx COLMSG_CONFIG_PATH /path/to/colmsg.conf
```

### 格式

配置文件是命令行参数的列表。运行 `colmsg --help` 可查看可用选项。以 `#` 开头的行为注释。

示例：

```text
# 樱坂46 refresh token
--s_refresh_token s_refresh_token

# 日向坂46 refresh token
--h_refresh_token h_refresh_token

# 乃木坂46 refresh token
--n_refresh_token n_refresh_token

# 斋藤飞鸟 refresh token
--a_refresh_token a_refresh_token

# 白石麻衣 refresh token
--m_refresh_token m_refresh_token

# yodel refresh token
--y_refresh_token y_refresh_token

# 仅保存媒体文件
-k picture -k video -k voice
```

## 安装

### Windows

从[发布页面](https://github.com/proshunsuke/colmsg/releases)下载 Windows 压缩包，并使用[7-Zip](https://www.7-zip.org/)等工具解压。压缩包中包含 `colmsg.exe`，可在 PowerShell 或其他终端中运行。

### macOS

使用 Homebrew 安装：

```fish
brew tap proshunsuke/colmsg
brew install colmsg
```

### Arch Linux

从 [AUR](https://aur.archlinux.org/packages/colmsg/) 安装：

```fish
yay -S colmsg
```

### 二进制文件

其他架构的预编译程序可在[发布页面](https://github.com/proshunsuke/colmsg/releases)下载。

## 开发

使用 `make test` 运行测试。要测试全部 feature，请运行 `make test-all-features`。
使用 `make fmt` 自动格式化 Rust 代码，并使用 `make fmt-check` 检查格式。

本地开发 API 时，可启动 OpenAPI 模拟服务器：

```fish
make server/kh
make server/n46
```

设置 `S_BASE_URL`、`H_BASE_URL` 和 `N_BASE_URL`，即可将请求转发到模拟服务器。例如：

```fish
env S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/ --help
```

## 许可证

`colmsg` 根据 MIT License 发布。详情请参阅 [LICENSE](LICENSE.txt)。
