[日本語](README.md) | [English](README.en.md) | [简体中文](README.zh-Hans.md) | 繁體中文

> [!WARNING]
> 目前 iOS 無法取得 `refresh_token`。這是訊息 App 更新造成的問題，並非 `colmsg` 本身的問題，目前沒有完整的解決方法。Android 使用者可參考[此處的日文說明](doc/how_to_get_refresh_token.md#android%E3%82%A2%E3%83%97%E3%83%AA%E3%81%AE%E5%A0%B4%E5%90%88)。

<div align="center">
  <h1><strong>colmsg</strong></h1>
  <img src="https://user-images.githubusercontent.com/3148511/158018437-09822a33-8767-4e03-ba90-e0f69594c493.jpeg" width="32px" alt="櫻坂46 Message 標誌"><img src="https://user-images.githubusercontent.com/3148511/158018441-dd7cb9eb-bf31-4938-830d-1ef293a2afba.jpg" width="32px" alt="日向坂46 Message 標誌"><img src="https://user-images.githubusercontent.com/3148511/158018442-ae54e926-760d-4b47-b0a0-7255485e1f28.jpg" width="32px" alt="乃木坂46 Message 標誌">

  將「櫻坂46 Message」「日向坂46 Message」「乃木坂46 Message」「齋藤飛鳥 Message」「白石麻衣 Message」和 yodel App 中的訊息儲存到電腦。

  ![示範](https://user-images.githubusercontent.com/3148511/158026220-90735546-2401-40ca-a9e6-89d2176ad3b4.gif)
</div>

## 概述

安裝方式請參閱[安裝](#安裝)。

首先取得 `refresh_token`。取得方式請參閱[此處的日文說明](doc/how_to_get_refresh_token.md)。

接著執行以下命令，並將佔位符替換成對應 App 取得的 `refresh_token`：櫻坂46 Message、日向坂46 Message、乃木坂46 Message、齋藤飛鳥 Message、白石麻衣 Message 和 yodel。只需填寫已訂閱 App 的權杖。

這會儲存所有已訂閱成員的完整歷史訊息。

```fish
colmsg --s_refresh_token <s_refresh_token> --h_refresh_token <h_refresh_token> --n_refresh_token <n_refresh_token> --a_refresh_token <a_refresh_token> --m_refresh_token <m_refresh_token> --y_refresh_token <y_refresh_token>
```

在 Windows 上，請將 `colmsg` 替換為 `colmsg.exe`。

## 功能

* ✅ 不需要 root 裝置
* ✅ 支援 Android 和 iOS App
* ✅ 可在 Windows、macOS 和 Linux 上執行
* ✅ 支援多種訊息篩選與儲存方式
* ✅ 支援以下 App 版本：
  * 櫻坂46 Message：1.12.01.169
  * 日向坂46 Message：2.13.01.169
  * 乃木坂46 Message：1.8.01.169
  * 齋藤飛鳥 Message：1.1.01.169
  * 白石麻衣 Message：3.4.3.426
  * yodel：4.1.1.455

## 使用方式

概述介紹了基本用法。由於 `refresh_token` 屬於敏感資訊，不建議直接在終端機輸入。建議將它寫入設定檔，詳情請參閱[設定檔](#設定檔)。以下範例假設權杖已寫入設定檔。

可透過選項選擇要儲存的內容。

儲存指定成員的訊息：

```fish
colmsg -n 菅井友香 -n 佐々木久美
```

儲存指定團體的訊息：

```fish
colmsg -g sakurazaka
```

儲存指定類型的訊息：

```fish
colmsg -k picture -k video
```

儲存指定日期之後的訊息：

```fish
colmsg -F '2020/01/01 00:00:00'
```

選項可以組合使用。執行 `colmsg --help` 查看詳細說明。

## 詳細資訊

* 如果已經儲存過訊息，執行 `colmsg` 時會從最後一則已儲存訊息之後繼續取得。
* 訊息會依照以下目錄結構儲存：
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
* 檔名格式為 `<序號>_<類型>_<日期>_<發文者姓名>.<副檔名>`。序號和日期前綴維持不變，因此檔案仍依時間順序排列。無法確認發文者時使用 `unknown`。類型編號如下：
  * 0：文字
  * 1：圖片
  * 2：影片
  * 3：語音
  * 4：連結
* 執行 `colmsg --download-dir` 可查看目前平台的預設下載目錄。
* 已儲存的訊息不會被覆寫。

## 設定檔

可以在設定檔中預先設定預設選項。執行 `colmsg --config-dir` 查看預設目錄。也可以透過 `COLMSG_CONFIG_PATH` 指定設定檔路徑：

```fish
set -gx COLMSG_CONFIG_PATH /path/to/colmsg.conf
```

### 格式

設定檔是命令列參數的清單。執行 `colmsg --help` 可查看可用選項。以 `#` 開頭的行是註解。

範例：

```text
# 櫻坂46 refresh token
--s_refresh_token s_refresh_token

# 日向坂46 refresh token
--h_refresh_token h_refresh_token

# 乃木坂46 refresh token
--n_refresh_token n_refresh_token

# 齋藤飛鳥 refresh token
--a_refresh_token a_refresh_token

# 白石麻衣 refresh token
--m_refresh_token m_refresh_token

# yodel refresh token
--y_refresh_token y_refresh_token

# 僅儲存媒體檔案
-k picture -k video -k voice
```

## 安裝

### Windows

從[發行頁面](https://github.com/proshunsuke/colmsg/releases)下載 Windows 壓縮檔，並使用 [7-Zip](https://www.7-zip.org/) 等工具解壓縮。壓縮檔中包含 `colmsg.exe`，可在 PowerShell 或其他終端機執行。

### macOS

使用 Homebrew 安裝：

```fish
brew tap proshunsuke/colmsg
brew install colmsg
```

### Arch Linux

從 [AUR](https://aur.archlinux.org/packages/colmsg/) 安裝：

```fish
yay -S colmsg
```

### 二進位檔

其他架構的預先編譯程式可在[發行頁面](https://github.com/proshunsuke/colmsg/releases)下載。

## 開發

使用 `make test` 執行測試。若要測試所有 feature，請執行 `make test-all-features`。
使用 `make fmt` 自動格式化 Rust 程式碼，並使用 `make fmt-check` 檢查格式。

本機開發 API 時，可啟動 OpenAPI 模擬伺服器：

```fish
make server/kh
make server/n46
```

設定 `S_BASE_URL`、`H_BASE_URL` 和 `N_BASE_URL`，即可將請求轉送到模擬伺服器。例如：

```fish
env S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/ --help
```

## TODO

* [ ] 提供範例
* [ ] 平行儲存訊息
* [ ] 將 API 用戶端拆分為獨立 crate

## 授權條款

`colmsg` 依 MIT License 發行。詳情請參閱 [LICENSE](LICENSE.txt)。
