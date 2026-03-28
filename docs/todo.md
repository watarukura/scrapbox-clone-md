# todo.md — 開発チェックリスト

## Phase 1: プロジェクト初期化

- [x] Tauri v2 + React + TypeScript + Vite プロジェクト作成

- [x] ビルド・起動確認

- [x] フロントエンド側ディレクトリ構成作成（`app/routes`, `app/layout`, `app/providers`, `features/`, `lib/`, `styles/`）

- [x] バックエンド側ディレクトリ構成作成（`commands/`, `services/`, `repositories/`, `models/`, `db/`, `markdown/`, `fs/`）

- [x] `src/lib/types.ts` に `Workspace` 型・`Note` 型を定義

## Phase 2: SQLite セットアップ

- [x] SQLite 接続・コネクション管理（`src-tauri/src/db/`）

- [x] `workspaces` テーブル作成マイグレーション

- [x] `notes` テーブル作成マイグレーション

- [x] `note_fts` 仮想テーブル作成マイグレーション

- [x] インデックス作成マイグレーション

- [x] 起動時自動マイグレーション実行

## Phase 3: ファイルシステム操作

- [x] `.md` ファイル再帰走査（`fs/scanner.rs`）

- [x] ファイル読み込み（`fs/reader.rs`）

- [x] アトミックファイル書き込み（`fs/writer.rs`）

## Phase 4: Markdown パーサー

- [x] YAML frontmatter 抽出・パース（`markdown/`）

- [x] frontmatter と本文の分離

- [x] Markdown → plaintext 変換（FTS 用）

- [x] タイトル決定ロジック（frontmatter.title → H1 → ファイル名）

## Phase 5: モデル・リポジトリ層

- [x] `Workspace` 構造体定義（`models/`）

- [x] `Note` 構造体定義（`models/`）

- [x] workspace リポジトリ — CRUD（`repositories/workspace_repository.rs`）

- [x] note リポジトリ — CRUD・upsert（`repositories/note_repository.rs`）

- [x] note リポジトリ — FTS インデックス更新

## Phase 6: サービス層

- [x] `workspace_service` — ワークスペース作成・取得

- [x] `workspace_service` — 初回スキャン統合（scan → parse → upsert）

- [x] `note_service` — ノート取得（DB）

- [x] `note_service` — ノート保存（ファイル → hash → DB → FTS）

- [x] `index_service` — 全ファイルスキャン → DB 同期

- [x] `index_service` — 差分検知（hash 比較）

- [x] `index_service` — 削除ファイルの `is_deleted` マーク

- [x] `search_service` — FTS5 全文検索

## Phase 7: Tauri コマンド

- [x] `open_workspace` コマンド

- [x] `list_notes` コマンド

- [x] `get_note` コマンド

- [x] `save_note` コマンド

- [x] `search_notes` コマンド

## Phase 8: フロントエンド — レイアウト

- [x] `AppShell.tsx` — 2 カラムレイアウト

- [x] `Sidebar.tsx` — ワークスペース名・検索・ノート一覧・新規作成ボタン

- [x] `Header.tsx` — ヘッダー

## Phase 9: フロントエンド — 状態管理

- [x] `WorkspaceProvider` — ワークスペース状態管理

- [x] `NotesProvider` — ノート一覧・選択状態管理

## Phase 10: フロントエンド — ページ

- [x] `Home.tsx` — フォルダ選択・ワークスペースを開く

- [x] `NoteEditor.tsx` — Tiptap エディタ統合

- [x] `NoteEditor.tsx` — Markdown ↔ エディタ状態変換

- [x] `NoteEditor.tsx` — 保存処理（Ctrl+S / 自動保存）

- [x] `NoteEditor.tsx` — 保存状態表示

- [x] 検索機能 — 検索入力 → 結果表示

## Phase 11: ファイル監視

- [x] `file_watch_service` — ワークスペースフォルダ変更監視

- [x] 変更検知時の再インデックス

- [x] Tauri イベントでフロントに通知

## Phase 12: 統合・仕上げ

- [ ] E2E 動作確認（ワークスペース選択 → スキャン → 一覧 → 編集 → 保存）

- [x] エラーハンドリング（ファイル I/O、DB、フロント表示）

- [x] スタイリング・レスポンシブ対応
