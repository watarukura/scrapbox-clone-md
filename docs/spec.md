---

# spec.md

## 概要

本アプリは、ローカルの Markdown ファイルをベースとしたノート管理アプリケーションである。
Scrapbox のような知識ベース体験を目指すが、MVP では以下にスコープを限定する。

* ローカルフォルダをワークスペースとして扱う
* Markdown ファイルの読み込み・一覧表示
* WYSIWYG エディタによる編集
* Markdown として保存
* SQLite を用いた索引管理

※ WikiLink、backlink、2-hop link は本フェーズでは対象外

---

## 技術スタック

### フロントエンド

* React
* TypeScript
* Vite
* Tiptap（WYSIWYGエディタ）

### デスクトップ基盤

* Tauri

### バックエンド（ローカル）

* Rust
* SQLite（FTS5）

---

## アーキテクチャ

### 基本方針

* **Markdown ファイルを正本とする**
* SQLite は検索・一覧表示のための索引
* ファイル変更と DB を同期する

```
[Markdown Files] ←→ [Rust Service] ←→ [SQLite Index]
                               ↑
                           [React UI]
```

---

## ワークスペース仕様

### 定義

* ユーザーが選択したローカルフォルダ
* 配下の `.md` ファイルをすべて対象とする

### 制約

* 1 ワークスペース = 1 ルートフォルダ
* `.md` 以外は対象外
* 再帰的に走査する

---

## ディレクトリ構成

```
scrapbox-local-clone/
├─ src/
│  ├─ app/
│  │  ├─ routes/
│  │  │  ├─ Home.tsx
│  │  │  ├─ NoteEditor.tsx
│  │  │  └─ Settings.tsx
│  │  ├─ layout/
│  │  │  ├─ AppShell.tsx
│  │  │  ├─ Sidebar.tsx
│  │  │  └─ Header.tsx
│  │  └─ providers/
│  │     ├─ WorkspaceProvider.tsx
│  │     └─ NotesProvider.tsx
│  │
│  ├─ features/
│  │  ├─ workspace/
│  │  ├─ notes/
│  │  ├─ editor/
│  │  └─ search/
│  │
│  ├─ lib/
│  ├─ styles/
│  └─ main.tsx
│
├─ src-tauri/
│  ├─ src/
│  │  ├─ commands/
│  │  ├─ services/
│  │  ├─ repositories/
│  │  ├─ models/
│  │  ├─ db/
│  │  ├─ markdown/
│  │  └─ fs/
│  │
│  └─ tauri.conf.json
│
└─ docs/
   └─ spec.md
```

---

## データモデル

### 設計方針

* ファイルシステムが正本
* SQLite はインデックス用途
* UUID を主キーとして採用

---

### workspaces

```
CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_scanned_at TEXT
);
```

---

### notes

```
CREATE TABLE notes (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  title TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  absolute_path TEXT NOT NULL,
  file_name TEXT NOT NULL,
  extension TEXT NOT NULL DEFAULT 'md',
  frontmatter_json TEXT,
  body_markdown TEXT NOT NULL,
  body_plaintext TEXT,
  hash_sha256 TEXT,
  file_created_at TEXT,
  file_modified_at TEXT,
  indexed_at TEXT NOT NULL,
  is_deleted INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id),
  UNIQUE (workspace_id, relative_path)
);
```

---

### note_fts（全文検索）

```
CREATE VIRTUAL TABLE note_fts USING fts5(
  title,
  body_plaintext,
  content='notes',
  content_rowid='rowid'
);
```

---

### インデックス

```
CREATE INDEX idx_notes_workspace_id ON notes(workspace_id);
CREATE INDEX idx_notes_title ON notes(title);
CREATE INDEX idx_notes_relative_path ON notes(relative_path);
CREATE INDEX idx_notes_file_modified_at ON notes(file_modified_at);
CREATE INDEX idx_notes_is_deleted ON notes(is_deleted);
```

---

## ノート仕様

### ファイル形式

* 拡張子: `.md`
* エンコーディング: UTF-8

---

### タイトル決定ルール

優先順位:

1. frontmatter の `title`
2. 最初の H1 (`# タイトル`)
3. ファイル名（拡張子除く）

---

### frontmatter

```
---
title: Example
tags: [sample]
```

本文
```

* JSON として保存
* MVP では解析のみ（構造化しない）

---

## 処理フロー

### 初回スキャン

1. ワークスペースフォルダ選択
2. `.md` ファイルを再帰的に探索
3. 各ファイルについて:

* frontmatter 抽出
* 本文取得
* plaintext 生成
* hash 計算
4. `notes` テーブルへ upsert
5. FTS インデックス更新

---

### ノート読み込み

1. DB から note を取得
2. Markdown をエディタにロード

---

### 保存処理

1. エディタから Markdown を取得
2. ファイルへ書き込み（atomic write）
3. hash 再計算
4. DB 更新
5. FTS 更新

---

### ファイル変更検知

* ファイル監視（watch）
* 外部変更時:

* 再読み込み
* DB 更新

---

## Rust 側構成

### commands

* フロントからのエントリポイント

例:

* `open_workspace`
* `list_notes`
* `get_note`
* `save_note`
* `search_notes`

---

### services

* ビジネスロジック

* workspace_service

* note_service

* index_service

* file_watch_service

---

### repositories

* DB アクセス

---

### fs

* ファイル走査
* ファイル書き込み

---

### markdown

* frontmatter 解析
* Markdown 分離

---

## UI構成

### レイアウト

```
+-------------------+------------------------------+
| Sidebar           | Editor                       |
|-------------------|                              |
| Workspace         | Title                        |
| Search            | Toolbar                      |
| Note List         | Editor Area                  |
|                   |                              |
+-------------------+------------------------------+
```

---

### 機能

#### Sidebar

* ワークスペース表示
* 検索
* ノート一覧
* 新規ノート作成

#### Editor

* タイトル編集
* WYSIWYG 本文編集
* 保存状態表示

---

## 型定義（フロント）

```
export type Workspace = {
  id: string
  name: string
  rootPath: string
  createdAt: string
  updatedAt: string
  lastScannedAt?: string | null
```

export type Note = {
id: string
workspaceId: string
title: string
relativePath: string
absolutePath: string
fileName: string
extension: string
frontmatterJson?: string | null
bodyMarkdown: string
bodyPlaintext?: string | null
fileCreatedAt?: string | null
fileModifiedAt?: string | null
indexedAt: string
isDeleted: boolean
}
```

---

## 制約・前提

* Markdown の完全なラウンドトリップは保証しない
* サポートする記法は限定する（基本的な Markdown のみ）
* 外部エディタとの同時編集は考慮しない（MVP）

---

## 今後の拡張

### 次フェーズ

* WikiLink (`[[Page]]`)
* backlink
* 2-hop link
* aliases
* タグ管理

### 将来

* グラフビュー
* 同期（クラウド）
* プラグインシステム

---

## 設計上の重要ポイント

* Markdown を唯一の正本とする
* UUID による内部識別
* SQLite はキャッシュ兼インデックス
* 構造はシンプルに保つ（MVP重視）
