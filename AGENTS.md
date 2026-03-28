# AGENTS.md

## Principles

-   Markdown files are the source of truth.
-   SQLite is an index/cache layer.
-   The app must always reflect the filesystem state.
-   Never allow silent divergence between DB and filesystem.

## Architecture Boundaries

### Frontend (React)

-   UI rendering
-   Editor state
-   Calls Tauri commands

### Backend (Rust / Tauri)

-   File system access
-   Markdown parsing
-   Indexing (SQLite)
-   Search

### Rules

-   Frontend MUST NOT access filesystem directly
-   Backend MUST NOT contain UI logic

## Data Model Rules

-   `.md` files are canonical
-   `notes` table is derived
-   Always:
    1.  Write to filesystem
    2.  Then update DB
-   Never update DB without file write
-   Never trust DB over filesystem

## Save Flow

1.  Receive Markdown from editor
2.  Write file (atomic)
3.  Recompute hash
4.  Update notes table
5.  Update FTS index

## Sync Rules

-   On workspace open: full scan
-   On file change: re-index changed file only
-   Deleted files: mark is_deleted = 1

## Title Resolution Priority

1.  frontmatter.title
2.  first H1
3.  filename

## Directory Responsibilities

### src/

Frontend only

### src-tauri/commands

Tauri entrypoints

### src-tauri/services

Business logic

### src-tauri/repositories

DB access

### src-tauri/fs

Filesystem ops

### src-tauri/markdown

Parsing logic

## Anti-Patterns (DO NOT)

-   Business logic in React components
-   Direct DB writes bypassing services
-   File writes outside fs/writer
-   Markdown parsing in frontend
-   Duplicated indexing logic

## Schema Rules

-   Changes must be backward compatible
-   Prefer additive changes
-   Avoid column removal without migration

## Future Features

-   WikiLinks
-   Backlinks
-   Graph relationships

Rules: - Use UUID as primary key - Do not rely on title identity

## Coding Guidelines

-   Keep functions small
-   Prefer explicit logic
-   Avoid hidden side effects

## Change Checklist

-   File ↔ DB consistency preserved?
-   Any duplicated state introduced?
-   Correct layer placement?
-   Future WikiLink compatibility?

