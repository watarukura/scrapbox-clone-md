---
name: e2e-testing
description: Playwright を使った E2E テストの実行・作成ガイドライン
---

# E2E Testing

## Overview

E2E テストは Playwright を使用し、`e2e/` ディレクトリに配置されています。

## Configuration

- テストディレクトリ: `./e2e`
- タイムアウト: 30秒
- ヘッドレスモード: 有効
- ベース URL: `http://localhost:1420`
- 開発サーバー: `pnpm dev` (ポート 1420)

## Commands

### E2E テスト実行

```bash
pnpm exec playwright test
```

### 特定テスト実行

```bash
pnpm exec playwright test e2e/<test-file>.spec.ts
```

### テストレポート表示

```bash
pnpm exec playwright show-report
```

### ブラウザインストール

```bash
pnpm exec playwright install
```

## Guidelines

- テストファイルは `e2e/` ディレクトリに `*.spec.ts` として配置する
- テストは開発サーバー (`pnpm dev`) が自動起動される
- CI 環境では `reuseExistingServer` が無効になる
- テスト結果は `test-results/` に出力される
