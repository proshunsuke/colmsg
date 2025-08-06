# refresh_tokenの取得方法

## はじめに

以下で説明する方法によってデータ損失等が発生する可能性があります。自己責任でお願いします。  
また、以下の説明に用いている画像は「欅坂46/日向坂46 メッセージ」アプリのものですが、「櫻坂46メッセージ」「日向坂46メッセージ」「乃木坂46メッセージ」「齋藤飛鳥メッセージ」「白石麻衣メッセージ」共に同様の方法でrefresh_tokenは取得可能なため、適宜それぞれのアプリ用に読み替えてください。

## 外部サービス連携

**必ず**外部サービス連携をしてください。外部サービス連携をしないとデータが失われる可能性があります。  
失われなかった場合でもデータ復旧のためにアプリ開発会社への問い合わせを行う必要があり、これには時間がかかる可能性があります。  
ちなみにおすすめはGoogleアカウントです。

<img src="https://user-images.githubusercontent.com/3148511/85218998-e0900a00-b3da-11ea-95a6-1bcf80453c3f.png" width="225" alt="setting.png"> → <img src="https://user-images.githubusercontent.com/3148511/85218999-e128a080-b3da-11ea-9841-4b2688057cdc.png" width="225" alt="sosical_service.png">

## アプリデータの削除

一度アプリのデータを削除する必要があります。  
過去のメッセージなどは外部サービス連携を行っていれば失われることは無いはずです。  

### iosアプリの場合

iosを使っている人の場合は一度アプリをアンインストールしてからインストールし直してください。  
インストール後、またアプリは起動しないでください。

### Androidアプリの場合

Androidアプリを使っている人の場合はデバイスのAndroidのバージョンによって異なります。

#### Android 6 以前の場合

Android 6 以前を使用している人の場合は [Androidアプリ共通のデータ削除](#Androidアプリ共通のデータ削除) へ進んでください。

#### Android 7 以降の場合

Android 7 以降を使用している人の場合は後述するmitmproxyが動作しないため、[Genymotion](https://www.genymotion.com/)などのエミュレータを使用し、Android 6 以前の環境を作ってください。

[こちらの記事](https://qiita.com/sou_lab/items/bb06bb653b291c90bf45)などを参考にしてGenymotionにGoogle Play ストアを入れてください。
その後「櫻坂46メッセージ」「日向坂46メッセージ」「乃木坂46メッセージ」「齋藤飛鳥メッセージ」「白石麻衣メッセージ」アプリをインストールしてください。

※ Google Play ストアからインストール出来ない場合はインターネット上からapkファイルを探して直接インストールしてください(APKPureなど)。

[Androidアプリ共通のデータ削除](#Androidアプリ共通のデータ削除) へ進んでください。

#### Androidアプリ共通のデータ削除

Androidアプリの場合はアンインストールすることなくデータ削除を行うことが出来ます。  
以下の手順に従ってデータ削除を行ってください。

<img src="https://user-images.githubusercontent.com/3148511/85218993-de2db000-b3da-11ea-9655-b0b4c56b766d.png" width="225"> → <img src="https://user-images.githubusercontent.com/3148511/85218994-df5edd00-b3da-11ea-8666-fc874ac41786.png" width="225"> → <img src="https://user-images.githubusercontent.com/3148511/85218996-dff77380-b3da-11ea-8478-03b3a0f46d62.png" width="225"> → <img src="https://user-images.githubusercontent.com/3148511/85218996-dff77380-b3da-11ea-8478-03b3a0f46d62.png" width="225"> → <img src="https://user-images.githubusercontent.com/3148511/85218997-dff77380-b3da-11ea-80be-0d5b72b0a366.png" width="225">

## refresh_tokenの取得

[mitmproxy](https://mitmproxy.org/)を使用します。  
mitmproxyの使用方法は検索すればたくさん出てきます。[こちら](https://vivibit.net/windows-mitmproxy-https)や[こちら](https://qiita.com/hkurokawa/items/9034274cc1b9e1405c68)の記事なども参考になるかも知れません。
Genymotionを使用している人の場合、プロキシのホスト名は `10.0.3.2` を指定すると上手くいきます([参考](https://qiita.com/hkusu/items/499575a566b20ce4d95b))。

次にmitmwebを起動し、アプリの通信内容を確認します。  
アプリを起動し、アカウントの引き継ぎを行ってください。  
Twitter連携を使用している場合など、アカウントの引き継ぎがうまくいかない場合はプロキシが問題である可能性があります。  
一度プロキシを無効にしてアカウントログイン画面に移動し、その後すぐにプロキシを有効にすることでうまくいく場合があります。  
Googleアカウントで引き継ぎを行っている場合はこの問題は起きないはずです。

<img src="https://user-images.githubusercontent.com/3148511/85219958-d1ad5580-b3e2-11ea-95f0-d448fd20150d.png" width="225"> → <img src="https://user-images.githubusercontent.com/3148511/85219960-d2de8280-b3e2-11ea-918e-d54a24018354.png" width="225"> → <img src="https://user-images.githubusercontent.com/3148511/85219961-d3771900-b3e2-11ea-809b-160ee1f757dd.png" width="225">

アカウントの引き継ぎを行っている最中にmitmproxyがアプリの通信内容をブラウザに表示させているはずです。  
ブラウザを確認し、 `https://api.kh.glastonr.net/v2/signin` へリクエストしている項目を探してください。  
その項目の `Response` を確認し、 `refresh_token` を取得してください。

<img src="https://user-images.githubusercontent.com/3148511/85220044-919aa280-b3e3-11ea-93fe-7b07a756057d.png">
