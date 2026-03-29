---
name: react-frontend
description: React フロントエンド開発のガイドライン。Tiptap エディタ、Tauri コマンド呼び出しルールを定義。
---

# React Frontend

## Overview

フロントエンドは React 19 + TypeScript + Vite で構築され、リッチテキストエディタに Tiptap を使用しています。

## Tech Stack

- React 19
- TypeScript 5.8
- Vite 7
- Tiptap 3 (エディタ)
- Tauri API v2 (バックエンド通信)

## Rules

- フロントエンドからファイルシステムに直接アクセスしない
- Markdown パースをフロントエンドで行わない
- ビジネスロジックを React コンポーネントに書かない
- バックエンドとの通信は Tauri コマンド経由のみ

## Commands

### TypeScript 型チェック

```bash
pnpm tsc --noEmit
```

### 開発サーバー

```bash
pnpm dev
```

### ビルド

```bash
pnpm build
```
