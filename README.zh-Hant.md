[日本語](README.md) | [English](README.en.md) | [简体中文](README.zh-Hans.md) | 繁體中文

<div align="center">
  <h1><strong>colmsg</strong></h1>
  <img src="https://user-images.githubusercontent.com/3148511/158018437-09822a33-8767-4e03-ba90-e0f69594c493.jpeg" width="32px" alt="櫻坂46 Message 標誌"><img src="https://user-images.githubusercontent.com/3148511/158018441-dd7cb9eb-bf31-4938-830d-1ef293a2afba.jpg" width="32px" alt="日向坂46 Message 標誌"><img src="https://user-images.githubusercontent.com/3148511/158018442-ae54e926-760d-4b47-b0a0-7255485e1f28.jpg" width="32px" alt="乃木坂46 Message 標誌">

  將「櫻坂46 Message」「日向坂46 Message」「乃木坂46 Message」「齋藤飛鳥 Message」「白石麻衣 Message」和 yodel App 中的訊息儲存到電腦。

  ![示範](doc/demo/colmsg.gif)
</div>

## 開始使用

請參照[安裝](#安裝)說明安裝 `colmsg`。
在 Windows 上，請將執行檔名稱替換為 `colmsg.exe`。

請事先在應用程式中完成登入帳號設定。  
請為每項服務登入一次。

```sh
colmsg login sakurazaka
```

可指定的服務名稱為 `sakurazaka`、`hinatazaka`、`nogizaka`、`asukasaito`、`maishiraishi` 和 `yodel`。
儲存所有已訂閱成員的完整歷史訊息。

```sh
colmsg
```

## 功能

* ✅ 不需要 root 裝置
* ✅ 整個流程可在 colmsg 中完成
* ✅ 可在 Windows、macOS 和 Linux 上執行
* ✅ 支援多種訊息篩選與儲存方式

## 訊息儲存選項

可透過選項選擇要儲存的內容。

儲存指定成員的訊息：

```sh
colmsg -n 菅井友香 -n 佐々木久美
```

儲存指定團體的訊息：

```sh
colmsg -g sakurazaka
```

儲存指定類型的訊息：

```sh
colmsg -k picture -k video
```

儲存指定日期之後的訊息：

```sh
colmsg -F '2020/01/01 00:00:00'
```

預設會在每個服務中並行儲存 4 位成員的訊息。可使用 `--jobs`（`-j`）變更並行數量。

```sh
colmsg --jobs 2
```

選項可以組合使用。執行 `colmsg --help` 查看詳細說明。

## 登入

登入時使用的瀏覽器會自動偵測。如需指定 Chrome、Brave、Edge 等以 Chromium 為基礎的瀏覽器，或自動偵測失敗，請透過 `--browser` 指定瀏覽器執行檔的路徑。

```sh
colmsg login sakurazaka --browser '/Applications/Brave Browser.app/Contents/MacOS/Brave Browser'
```

也可透過環境變數 `COLMSG_BROWSER` 指定，`--browser` 優先。

登入成功後，認證資訊儲存在設定目錄中的 `auth/<service>.json`。
可以查看已儲存的認證方式、帳號與有效期限。

```sh
colmsg auth status
```

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
* 檔名格式為 `<序號>_<類型>_<日期>_<發文者姓名>.<副檔名>`。
  * 序號表示訊息的時間順序。發文者姓名之前的部分維持原有順序，因此在檔案瀏覽器中依字典順序排序，即可依時間順序查看已儲存的訊息。
  * 無法確認發文者時使用 `unknown`。
  * 類型編號如下：
    * 0：文字
    * 1：圖片
    * 2：影片
    * 3：語音
    * 4：連結
* 執行 `colmsg --download-dir` 可查看依據設定檔和命令列選項決定的下載目錄。
* 已儲存的訊息不會被覆寫。

## 設定檔

可以在設定檔中預先設定預設選項。執行 `colmsg --config-path` 可查看目前使用的設定檔路徑。也可以透過 `COLMSG_CONFIG_PATH` 指定設定檔路徑：

```sh
export COLMSG_CONFIG_PATH="/path/to/colmsg.conf"
```

### 格式

設定檔是命令列參數的簡單清單。執行 `colmsg --help` 可查看可用選項與參數值。使用 `#` 可以加入註解。

設定檔範例：

```text
# 僅處理櫻坂的訊息
-g sakurazaka

# 僅儲存媒體檔案
-k picture -k video -k voice

# 將每項服務並行儲存的成員數設為 6
--jobs 6
```

## 傳統refresh_token認證（已棄用）

`refresh_token` 方式 **已棄用**。新使用者及移轉使用者請使用 `colmsg login <service>` 註冊 Cookie 認證。

僅在使用傳統方式時，請參考[舊取得步驟（日文）](doc/how_to_get_refresh_token.md)，將各項服務的權杖寫入設定檔。

傳統設定範例（已棄用，僅填寫所需服務）：

```text
# 設定 s_refresh_token
--s_refresh_token s_refresh_token

# 設定 h_refresh_token
--h_refresh_token h_refresh_token

# 設定 n_refresh_token
--n_refresh_token n_refresh_token

# 設定 a_refresh_token
--a_refresh_token a_refresh_token

# 設定 m_refresh_token
--m_refresh_token m_refresh_token

# 設定 y_refresh_token
--y_refresh_token y_refresh_token
```

## 安裝

### Windows

從[發行頁面](https://github.com/proshunsuke/colmsg/releases)下載包含 Windows 預先編譯執行檔的 ZIP 壓縮檔。
使用 [7-Zip](https://sevenzip.osdn.jp/) 等工具解壓縮。
解壓縮後可取得執行檔 `colmsg.exe`。
請在 [PowerShell](https://docs.microsoft.com/ja-jp/powershell/) 等終端機中執行。

### macOS

使用 Homebrew 安裝：

```sh
brew tap proshunsuke/colmsg
brew install colmsg
```

### Arch Linux

從 [AUR](https://aur.archlinux.org/packages/colmsg/) 安裝：

```sh
yay -S colmsg
```

### 二進位檔

其他架構的預先編譯程式可在[發行頁面](https://github.com/proshunsuke/colmsg/releases)下載。

## 開發

使用 `make test` 執行測試。若要測試所有 feature，請執行 `make test-all-features`。
使用 `make fmt` 自動格式化 Rust 程式碼，並使用 `make fmt-check` 檢查格式。

本機開發 API 時，可啟動 OpenAPI 模擬伺服器：

```sh
make server/kh
make server/n46
```

設定 `S_BASE_URL`、`H_BASE_URL` 和 `N_BASE_URL`，即可將請求轉送到模擬伺服器。例如：

```sh
S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/ --help
```

## 授權條款

`colmsg` 依 MIT License 發行。詳情請參閱 [LICENSE](LICENSE.txt)。
