---
name: rust-backend
description: Rust バックエンド開発のガイドライン。レイヤー構成、データフロー、テスト方法を定義。
---

# Rust Backend

## Overview

Tauri バックエンドは Rust で実装され、ファイルシステムアクセス、Markdown パース、SQLite インデックス、検索機能を担当します。

## Architecture Layers

1. **commands/** - Tauri コマンドエントリポイント。フロントエンドから呼ばれる。
2. **services/** - ビジネスロジック。コマンドから呼ばれる。
3. **repositories/** - SQLite DB アクセス。サービスから呼ばれる。
4. **fs/** - ファイルシステム操作。サービスから呼ばれる。
5. **markdown/** - Markdown パースロジック。

## Data Flow Rules

- `.md` ファイルが正（canonical）。SQLite は派生データ。
- 保存フロー: ファイル書き込み → ハッシュ再計算 → notes テーブル更新 → FTS インデックス更新
- DB のみの更新は禁止。必ずファイル書き込みを先に行う。
- ファイルシステムの状態を常に信頼する。

## Title Resolution

優先順位:
1. frontmatter の `title`
2. 最初の H1
3. ファイル名

## Commands

### Rust ビルド

```bash
cd src-tauri && cargo build
```

### Rust テスト

```bash
cd src-tauri && cargo test
```

### Rust フォーマット

```bash
cd src-tauri && cargo fmt
```

### Rust リント

```bash
cd src-tauri && cargo clippy
```

## Anti-Patterns

- コマンド層にビジネスロジックを書かない
- `fs/` 以外でファイル書き込みしない
- サービスを経由せず直接 DB を更新しない
- UUID を主キーとして使用する（タイトルに依存しない）

## Schema Rules

- 後方互換性を維持する
- カラム追加を優先する
- マイグレーションなしのカラム削除は禁止
