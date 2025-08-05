# CLAUDE.md

このファイルは、このリポジトリでコードを扱う際のClaude Code (claude.ai/code) への指針を提供します。

## プロジェクト概要

colmsgは日本のアイドルグループのメッセージアプリ（櫻坂46、日向坂46、乃木坂46、齋藤飛鳥）からメッセージをダウンロードするRust製CLIアプリケーションです。メッセージとメディアファイルを適切に整理してローカルに保存します。

## よく使う開発コマンド

### ビルド
```bash
# 標準的なRustビルド
cargo build
cargo build --release

# クロスプラットフォームリリースビルド
make release/x86_64-linux    # Linux x86_64
make release/x86_64-darwin   # macOS x86_64
make release/aarch64-darwin  # macOS ARM64
make release/x86_64-win      # Windows x86_64
```

### テスト
```bash
# テストの実行
cargo test

# モックサーバーでの実行（先にモックサーバーを起動）
S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/
```

### 開発環境
```bash
# モックAPIサーバーの起動
make server/kh     # 櫻坂/日向坂 API (ポート 8003)
make server/n46    # 乃木坂 API (ポート 8006)

# Swagger UIへのアクセス
make open/ui/kh    # http://localhost:8002 を開く
make open/ui/n46   # http://localhost:8005 を開く

# サーバーの停止
make stop/server/kh
make stop/server/n46
make down          # 全コンテナを停止
```

## アーキテクチャ

### 主要モジュール
- `src/bin/colmsg/`: CLIエントリーポイントとclapを使った引数解析
- `src/controller.rs`: メッセージダウンロードを統括するメインアプリケーションコントローラー
- `src/http/`: 各メッセージアプリAPI用のHTTPクライアント
  - `sakurazaka.rs`, `hinatazaka.rs`, `nogizaka.rs`, `asuka_saito.rs`
- `src/message/`: メッセージ処理と保存ロジック
  - 異なるメッセージタイプ（テキスト、画像、動画、音声）を処理
  - タイムスタンプベースのファイル名で保存

### API統合
- `/api/`内のOpenAPI仕様がメッセージアプリAPIを定義
- 認証ヘッダー付きのHTTPリクエストにreqwestを使用
- 環境変数経由で本番とモックAPIエンドポイントの両方をサポート

### 設定
- 設定ファイルにリフレッシュトークンとデフォルトオプションを保存
- 場所: `~/.config/colmsg/` (または`COLMSG_CONFIG_PATH`経由)
- メンバー/グループのフィルタリングとカスタム保存ディレクトリをサポート