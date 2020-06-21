# <img src="https://lh3.googleusercontent.com/NJqDujmTh1rwBtdnoOFci0bXNSNcm9pNjg93IfOkVGnjJC8hS5fzupnWFvOgqVqR7FY=s180" width="24px" alt="欅坂46/日向坂46 メッセージのロゴ"> colmsg

「欅坂46/日向坂46 メッセージ」アプリのメッセージをPCに保存します。

![sample.png](doc/sample.png)

## 概要

`colmsg` のインストール方法は[こちら](#インストール)を参照してください。

**まず初めに**「欅坂46/日向坂46 メッセージ」アプリのrefresh_tokenを取得してください。
取得方法は[こちら](doc/how_to_get_refresh_token.md)を参照してください。

取得出来たら以下を実行してください。
`<refresh_token>` に取得してきたrefresh_tokenを入れてください。  
購読しているメンバー全員の全期間のメッセージが保存されます。

```shell script
$ colmsg -t <refresh_token>
```

## 特徴

* [x] 端末のroot化の必要がありません
* [x] Android, iosアプリどちらにも対応しています
* [x] Windows, macos, Linuxで実行できます
* [x] 様々な保存方法が選べます

## 使い方

概要で基本的な使い方を説明しました。
しかし、refresh_tokenは機微情報なため、ターミナル上で直接入力するのはあまり良くないでしょう。  
そこで、configファイルにデフォルトのオプションを設定しておくことをおすすめします。
configファイルについては[こちら](#configファイル)を参照してください。
以降はconfigファイルでrefresh_tokenが設定されているものとします。

`colmsg` にはいくつかのオプションがあり、様々な保存方法を選べます。

特定のメンバーのメッセージを保存したい場合

```shell script
$ colmsg -n 菅井友香 -n 佐々木久美
```

特定のグループのメッセージを保存したい場合

```shell script
$ colmsg -g keyakizaka
```

特定の種類のメッセージを保存したい場合

```shell script
$ colmsg -k image -k video
```

特定の日時以降のメッセージを保存したい場合

```shell script
$ colmsg -F '2020/01/01 00:00:00'
```

オプションは組み合わせて使用することが出来ます。より詳細な説明は以下を実行して確認してください。

```shell script
$ colmsg --help
```

## 詳細な仕様

* 既にいくつかメッセージが保存されている場合にコマンドを実行すると、最後に保存したメッセージ以降のメッセージを取得して保存します
* 保存されるメッセージは次のディレクトリ構造で保存されます
  * ```shell script
        .
        ├── 日向坂46
        │   └── 佐々木久美
        │       ├── 1_0_20191231235959.txt
        │       └── 2_1_20200101000000.jpg
        └── 欅坂46
            └── 菅井友香
                ├── 3_2_20200101000001.mp4
                └── 4_3_20200101000002.mp4
        
    ```
* ファイル名の形式は `<シーケンス番号>_<種類>_<日付>.<拡張子>` となっています
  * シーケンス番号はメッセージの時系列を表す番号になっています。若い数字程昔のメッセージです。ファイルブラウザで辞書順に並べると保存したメッセージが時系列通りに並びます
  * 種類の数字は以下のように対応しています
    * 0: テキストメッセージ
    * 1: 画像
    * 2: 動画
    * 3: ボイス
* 各環境毎のデフォルトの保存先は以下を実行することで確認することが出来ます
  * ```shell script
    $ colmsg --download-dir
    ```
* 既に保存済のメッセージは上書き保存されません

## configファイル

`colmsg` は設定ファイルで予めオプションを指定することが出来ます。
デフォルトのパスは以下を実行することで確認することが出来ます。

```shell script
$ colmsg --config-dir
```
また、環境変数 `COLMSG_CONFIG_PATH` に設定ファイルの場所を明記することもできます。

```shell script
export COLMSG_CONFIG_PATH="/path/to/colmsg.conf"
```

### フォーマット

この設定ファイルはコマンドライン引数の単純なリストです。`colmsg --help` を利用すると、利用可能なオプションとその値を閲覧することが出来ます。さらに、`#` でコメント文を加えることができます。

設定ファイルの例:

```bash
# refresh_tokenを指定
-t refresh_token

# メディアファイルだけ保存するように設定
-k image -k video -k voice
```

## インストール

### macOS

Homebrewでインストールすることが出来ます。

```shell script
$ brew tap proshunsuke/colmsg
$ brew install colmsg
```

### Arch Linux

[AUR](https://aur.archlinux.org/packages/colmsg/)からインストールできます。

```bash
yay -S colmsg
```

### バイナリ

異なるアーキテクチャのためのビルド済実行ファイルを[リリースページ](https://github.com/proshunsuke/colmsg/releases)に配置しました。

## 開発

`colmsg` は外部APIを叩きます。開発時はAPIのモックサーバを建てると良いでしょう。そのためのSwagger Specを配置してあります。
TODO: Swagger Specを置く

モックサーバのbase urlを環境変数 `BASE_URL` で指定することが出来ます。以下のように指定することでモックサーバ宛てにAPIを叩きにいきます。

```shell script
$ BASE_URL=http://127.0.0.1:3100 cargo run -- --help
```

## TODO

* [ ] deleteの実装
* [ ] docker環境の用意
* [ ] CIによる自動テスト
* [ ] usernameとtokenの詳しい取得方法
* [ ] examplesの用意
* [ ] windowsでの動作確認
* [ ] メッセージ保存処理の並列化
* [ ] api clientのcrate化
* [ ] swagger specの配置

## ライセンス

`colmsg` は MIT License の条件の下で配布されています。

ライセンスの詳細については [LICENSE](LICENSE.txt) ファイルを参照して下さい。

## 注意事項

アプリの利用規約 第8条（禁止事項）に以下の項目があるため注意してください。

* (16) 当社が指定するアクセス方法以外の手段で本サービスにアクセスし、またはアクセスを試みる行為
* (17) 自動化された手段（クローラおよび類似の技術を含む）を用いて本サービスにアクセスし、またはアクセスを試みる行為 
