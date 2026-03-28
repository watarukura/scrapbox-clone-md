# prompt_plan.md — 開発ステップ分解

## Phase 1: プロジェクト初期化

### Step 1-1: Tauri + React プロジェクトのスキャフォールド

- Tauri v2 + React + TypeScript + Vite でプロジェクトを作成

- ビルド・起動が通ることを確認

### Step 1-2: ディレクトリ構成の作成

- `src/` 配下: `app/routes`, `app/layout`, `app/providers`, `features/`, `lib/`, `styles/`

- `src-tauri/src/` 配下: `commands/`, `services/`, `repositories/`, `models/`, `db/`, `markdown/`, `fs/`

- 各ディレクトリに空の `mod.rs` または `index.ts` を配置

### Step 1-3: フロントエンド型定義

- `Workspace` 型と `Note` 型を spec.md に基づいて定義

- `src/lib/types.ts` に配置

---

## Phase 2: SQLite セットアップ

### Step 2-1: SQLite 接続の初期化

- Tauri の起動時に SQLite データベースを作成・接続する処理を実装

- `src-tauri/src/db/` にコネクション管理を配置

### Step 2-2: マイグレーション — テーブル作成

- `workspaces` テーブル作成

- `notes` テーブル作成

- `note_fts` 仮想テーブル作成

- インデックス作成

- 起動時に自動マイグレーションを実行

---

## Phase 3: ファイルシステム操作

### Step 3-1: ファイル走査（scanner）

- 指定フォルダ配下の `.md` ファイルを再帰的に列挙

- `src-tauri/src/fs/scanner.rs` に実装

### Step 3-2: ファイル読み書き（reader / writer）

- Markdown ファイルの読み込み

- Markdown ファイルのアトミック書き込み

- `src-tauri/src/fs/reader.rs`, `writer.rs` に実装

---

## Phase 4: Markdown パーサー

### Step 4-1: frontmatter 解析

- YAML frontmatter の抽出・パース

- frontmatter と本文の分離

- `src-tauri/src/markdown/` に実装

### Step 4-2: plaintext 生成

- Markdown から plaintext を生成（FTS 用）

- タイトル決定ロジック（frontmatter.title → H1 → ファイル名）

---

## Phase 5: モデル・リポジトリ層

### Step 5-1: Rust モデル定義

- `Workspace` 構造体

- `Note` 構造体

- `src-tauri/src/models/` に配置

### Step 5-2: workspace リポジトリ

- `workspaces` テーブルの CRUD

- `src-tauri/src/repositories/workspace_repository.rs`

### Step 5-3: note リポジトリ

- `notes` テーブルの CRUD（upsert 含む）

- FTS インデックスの更新

- `src-tauri/src/repositories/note_repository.rs`

---

## Phase 6: サービス層

### Step 6-1: workspace_service

- ワークスペースの作成・取得

- 初回スキャン処理の統合（scan → parse → upsert）

### Step 6-2: note_service

- ノート取得（DB から）

- ノート保存（ファイル書き込み → hash 再計算 → DB 更新 → FTS 更新）

### Step 6-3: index_service

- 全ファイルスキャン → DB 同期

- 差分検知（hash 比較）

- 削除ファイルの `is_deleted` マーク

### Step 6-4: search_service

- FTS5 を使った全文検索

---

## Phase 7: Tauri コマンド

### Step 7-1: ワークスペース系コマンド

- `open_workspace` — フォルダ選択 → ワークスペース作成 → 初回スキャン

- `src-tauri/src/commands/workspace_commands.rs`

### Step 7-2: ノート系コマンド

- `list_notes` — ノート一覧取得

- `get_note` — 単一ノート取得

- `save_note` — ノート保存

- `src-tauri/src/commands/note_commands.rs`

### Step 7-3: 検索コマンド

- `search_notes` — 全文検索

- `src-tauri/src/commands/search_commands.rs`

---

## Phase 8: フロントエンド — レイアウト

### Step 8-1: AppShell

- サイドバー + メインエリアの 2 カラムレイアウト

- `src/app/layout/AppShell.tsx`

### Step 8-2: Sidebar

- ワークスペース名表示

- 検索入力欄

- ノート一覧

- 新規ノート作成ボタン

- `src/app/layout/Sidebar.tsx`

### Step 8-3: Header

- 基本的なヘッダー

- `src/app/layout/Header.tsx`

---

## Phase 9: フロントエンド — 状態管理

### Step 9-1: WorkspaceProvider

- 現在のワークスペース状態を管理

- `open_workspace` コマンドの呼び出し

### Step 9-2: NotesProvider

- ノート一覧の状態管理

- 選択中ノートの管理

---

## Phase 10: フロントエンド — ページ

### Step 10-1: Home（ワークスペース選択）

- フォルダ選択ダイアログ

- ワークスペースを開く

### Step 10-2: NoteEditor

- Tiptap エディタの統合

- Markdown ↔ エディタ状態の変換

- 保存処理（Ctrl+S / 自動保存）

- 保存状態の表示

### Step 10-3: 検索機能

- 検索入力 → `search_notes` 呼び出し → 結果表示

---

## Phase 11: ファイル監視

### Step 11-1: file_watch_service

- ワークスペースフォルダの変更監視

- 変更検知時に該当ファイルを再インデックス

- Tauri イベントでフロントに通知

---

## Phase 12: 統合・仕上げ

### Step 12-1: エンドツーエンド動作確認

- ワークスペース選択 → スキャン → 一覧表示 → 編集 → 保存 の一連のフロー確認

### Step 12-2: エラーハンドリング

- ファイル読み書きエラー

- DB エラー

- フロントエンドでのエラー表示

### Step 12-3: スタイリング

- 基本的な CSS / スタイル調整

- レスポンシブ対応（最低限）
