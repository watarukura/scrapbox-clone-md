---
name: tauri-dev
description: Tauri v2 アプリケーションのビルド・開発サーバー起動に関するスキル
---

# Tauri Dev

## Overview

本プロジェクトは Tauri v2 (Rust + React/TypeScript) デスクトップアプリケーションです。

## Commands

### 開発サーバー起動

```bash
pnpm tauri dev
```

### プロダクションビルド

```bash
pnpm tauri build
```

### フロントエンドのみ起動

```bash
pnpm dev
```

### フロントエンドビルド

```bash
pnpm build
```

## Project Structure

- `src/` - フロントエンド (React/TypeScript)
- `src-tauri/` - バックエンド (Rust/Tauri)
  - `src/commands/` - Tauri コマンド (エントリポイント)
  - `src/services/` - ビジネスロジック
  - `src/repositories/` - DB アクセス
  - `src/fs/` - ファイルシステム操作
  - `src/markdown/` - Markdown パース
  - `src/models/` - データモデル
  - `src/db/` - DB 初期化・マイグレーション

## Tool Versions

`mise.toml` で管理:
- Node.js: latest
- pnpm: latest
- Rust: latest
