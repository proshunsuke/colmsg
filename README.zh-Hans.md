[日本語](README.md) | [English](README.en.md) | 简体中文 | [繁體中文](README.zh-Hant.md)

<div align="center">
  <h1><strong>colmsg</strong></h1>
  <img src="https://user-images.githubusercontent.com/3148511/158018437-09822a33-8767-4e03-ba90-e0f69594c493.jpeg" width="32px" alt="樱坂46 Message 标志"><img src="https://user-images.githubusercontent.com/3148511/158018441-dd7cb9eb-bf31-4938-830d-1ef293a2afba.jpg" width="32px" alt="日向坂46 Message 标志"><img src="https://user-images.githubusercontent.com/3148511/158018442-ae54e926-760d-4b47-b0a0-7255485e1f28.jpg" width="32px" alt="乃木坂46 Message 标志">

  将「樱坂46 Message」「日向坂46 Message」「乃木坂46 Message」「斋藤飞鸟 Message」「白石麻衣 Message」和 yodel 应用中的消息保存到电脑。

  ![演示](doc/demo/colmsg.gif)
</div>

## 开始使用

请参照[安装](#安装)说明安装 `colmsg`。
在 Windows 上，请将可执行文件名替换为 `colmsg.exe`。

请事先在应用中完成登录账号设置。  
请为每项服务登录一次。

```sh
colmsg login sakurazaka
```

可指定的服务名称为 `sakurazaka`、`hinatazaka`、`nogizaka`、`asukasaito`、`maishiraishi` 和 `yodel`。
保存所有已订阅成员的完整历史消息。

```sh
colmsg
```

## 功能

* ✅ 无需 root 设备
* ✅ 整个流程可在 colmsg 中完成
* ✅ 可在 Windows、macOS 和 Linux 上运行
* ✅ 支持多种消息筛选和保存方式

## 消息保存选项

可通过选项选择要保存的内容。

保存指定成员的消息：

```sh
colmsg -n 菅井友香 -n 佐々木久美
```

保存指定组合的消息：

```sh
colmsg -g sakurazaka
```

保存指定类型的消息：

```sh
colmsg -k picture -k video
```

保存指定日期之后的消息：

```sh
colmsg -F '2020/01/01 00:00:00'
```

默认情况下，每个服务会并行保存4名成员的消息。可使用 `--jobs`（`-j`）更改并行数量。

```sh
colmsg --jobs 2
```

选项可以组合使用。运行 `colmsg --help` 查看详细说明。

## 登录

登录时使用的浏览器会自动检测。如需指定 Chrome、Brave、Edge 等基于 Chromium 的浏览器，或自动检测失败，请通过 `--browser` 指定浏览器可执行文件的路径。

```sh
colmsg login sakurazaka --browser '/Applications/Brave Browser.app/Contents/MacOS/Brave Browser'
```

也可通过环境变量 `COLMSG_BROWSER` 指定，`--browser` 优先。

登录成功后，认证信息保存在配置目录中的 `auth/<service>.json`。
可以查看已保存的认证方式、账户和有效期。

```sh
colmsg auth status
```

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
* 文件名格式为 `<序号>_<类型>_<日期>_<发布者姓名>.<扩展名>`。
  * 序号表示消息的时间顺序。发布者姓名之前的部分保持原有顺序，因此在文件浏览器中按字典顺序排序，即可按时间顺序查看已保存的消息。
  * 无法确定发布者时使用 `unknown`。
  * 类型编号如下：
    * 0：文本
    * 1：图片
    * 2：视频
    * 3：语音
    * 4：链接
* 运行 `colmsg --download-dir` 可查看根据配置文件和命令行选项确定的下载目录。
* 已保存的消息不会被覆盖。

## 配置文件

可以在配置文件中预先设置默认选项。运行 `colmsg --config-path` 可查看当前使用的配置文件路径。也可以通过 `COLMSG_CONFIG_PATH` 指定配置文件路径：

```sh
export COLMSG_CONFIG_PATH="/path/to/colmsg.conf"
```

### 格式

配置文件是命令行参数的简单列表。运行 `colmsg --help` 可查看可用选项和参数值。使用 `#` 可以添加注释。

配置文件示例：

```text
# 仅处理樱坂的消息
-g sakurazaka

# 仅保存媒体文件
-k picture -k video -k voice

# 将每项服务并行保存的成员数设为 6
--jobs 6
```

## 传统refresh_token认证（已弃用）

`refresh_token` 方式 **已弃用**。新用户和迁移用户请使用 `colmsg login <service>` 注册 Cookie 认证。

仅在使用传统方式时，请参考[旧获取步骤（日文）](doc/how_to_get_refresh_token.md)，将各项服务的令牌写入配置文件。

传统配置示例（已弃用，仅填写所需服务）：

```text
# 设置 s_refresh_token
--s_refresh_token s_refresh_token

# 设置 h_refresh_token
--h_refresh_token h_refresh_token

# 设置 n_refresh_token
--n_refresh_token n_refresh_token

# 设置 a_refresh_token
--a_refresh_token a_refresh_token

# 设置 m_refresh_token
--m_refresh_token m_refresh_token

# 设置 y_refresh_token
--y_refresh_token y_refresh_token
```

## 安装

### Windows

从[发布页面](https://github.com/proshunsuke/colmsg/releases)下载包含 Windows 预编译可执行文件的 ZIP 压缩包。
使用 [7-Zip](https://sevenzip.osdn.jp/) 等工具解压。
解压后可获得可执行文件 `colmsg.exe`。
请在 [PowerShell](https://docs.microsoft.com/ja-jp/powershell/) 等终端中运行。

### macOS

使用 Homebrew 安装：

```sh
brew tap proshunsuke/colmsg
brew install colmsg
```

### Arch Linux

从 [AUR](https://aur.archlinux.org/packages/colmsg/) 安装：

```sh
yay -S colmsg
```

### 二进制文件

其他架构的预编译程序可在[发布页面](https://github.com/proshunsuke/colmsg/releases)下载。

## 开发

使用 `make test` 运行测试。要测试全部 feature，请运行 `make test-all-features`。
使用 `make fmt` 自动格式化 Rust 代码，并使用 `make fmt-check` 检查格式。

本地开发 API 时，可启动 OpenAPI 模拟服务器：

```sh
make server/kh
make server/n46
```

设置 `S_BASE_URL`、`H_BASE_URL` 和 `N_BASE_URL`，即可将请求转发到模拟服务器。例如：

```sh
S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/ --help
```

## 许可证

`colmsg` 根据 MIT License 发布。详情请参阅 [LICENSE](LICENSE.txt)。
