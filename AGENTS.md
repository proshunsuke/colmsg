## READMEと言語版

- `README.md` は日本語版、`README.en.md` は英語版、`README.zh-Hans.md` は簡体字中国語版、`README.zh-Hant.md` は繁体字中国語版。
- READMEの内容を変更するときは、原則として意味上の変更を4言語すべてに同時に反映する。特定の言語だけを変更する明示的な依頼がある場合はそれに従う。
- 各言語の表現は自然な翻訳にしつつ、機能、手順、警告、リンク先などの情報を一致させる。
- README冒頭の言語切り替えリンクも維持し、言語版の追加・削除時は全READMEでリンクを揃える。

## テストとコード品質

- テストの設計・追加・修正・レビューでは [テスト方針](.agents/skills/testing/SKILL.md) に従う。公開インターフェースから観測できる振る舞いを検証し、非公開関数を直接テストしない。
- 外部APIはテスト境界でモックし、通常のテストで実APIや実トークンに依存しない。
- テストは `make test`、全featureのテストは `make test-all-features`、カバレッジは `make coverage` で実行する。
- Rustコードを自動整形する場合は `make fmt`、整形確認には `make fmt-check` を使う。
- CIに含まれるチェックを追加・変更した場合は、対応するMakeターゲットを使ってローカルでも確認する。

## Rustツールチェーンの更新

- Rustを更新するときは、[Rust公式リリース](https://blog.rust-lang.org/releases/latest/)で最新stableのバージョンを確認する。
- 確認したバージョンを次の設定に同じ値で反映する。
  - `rust-toolchain.toml` の `channel`
  - `Cargo.toml` の `package.rust-version`
  - `.github/workflows/test.yml` と `.github/workflows/release-build.yml` の `dtolnay/rust-toolchain` 指定
- `rust-version` は最低対応バージョンを表すため、プロジェクトのサポート方針に従ってツールチェーンの固定値と揃える。
- 更新後は設定値に不一致がないことを確認し、`make fmt-check`、`make test`、`make test-all-features` を実行する。リリース用ビルドはGitHub Actionsのリリース検証ワークフローで確認する。
