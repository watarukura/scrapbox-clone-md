# scrapbox-clone-md

Markdownファイルをソースとするローカルファーストなノートアプリ。Scrapboxライクな体験をデスクトップで提供します。

## Requirements

- [mise](https://mise.jdx.dev/) (推奨) または以下を個別にインストール
  - Node.js (latest)
  - pnpm (latest)
  - Rust (latest)
- Tauri v2 の[システム依存ライブラリ](https://v2.tauri.app/start/prerequisites/)

## Usage

1. アプリを起動すると、ワークスペース（ディレクトリ）を選択するダイアログが表示されます
2. 選択したディレクトリ内の `.md` ファイルが一覧表示されます
3. ノートを選択して編集できます。変更は自動的にMarkdownファイルへ保存されます

## Development

### セットアップ

```bash
# mise を使う場合（Node.js, pnpm, Rust を自動インストール）
mise install

# 依存パッケージのインストール
pnpm install
```

### 開発サーバーの起動

```bash
pnpm tauri dev
```

### ビルド

```bash
pnpm tauri build
```

### テスト

#### Rust 単体テスト

```bash
cd src-tauri
cargo test
```

#### E2E テスト (Playwright)

```bash
# Playwright ブラウザのインストール（初回のみ）
pnpm exec playwright install

# E2E テストの実行
pnpm exec playwright test

# UI モードで実行（デバッグ時に便利）
pnpm exec playwright test --ui
```

### リント・フォーマット

```bash
# フロントエンド
pnpm exec tsc --noEmit        # 型チェック

# バックエンド
cd src-tauri
cargo fmt                      # フォーマット
cargo clippy                   # リント
```

### プロジェクト構成

```
src/             # フロントエンド (React)
src-tauri/       # バックエンド (Rust / Tauri)
  commands/      # Tauri コマンド（エントリポイント）
  services/      # ビジネスロジック
  repositories/  # DB アクセス
  fs/            # ファイルシステム操作
  markdown/      # Markdown パース
```
