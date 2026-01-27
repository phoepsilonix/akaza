# Claude Code Development Guidelines

このファイルには、Claude Code を使用してこのプロジェクトを開発する際のガイドラインを記載します。

## コミット前のチェックリスト

### 必須: コードフォーマット

**コミット前に必ず `cargo fmt` を実行してください。**

```bash
cargo fmt
```

これにより、Rust コードが統一されたスタイルでフォーマットされます。

### 推奨: テスト実行

変更後にテストを実行して、既存機能が壊れていないことを確認してください。

```bash
# 全体のテスト
cargo test

# 特定のパッケージのみ
cargo test --package libakaza
cargo test --package akaza-data
```

### 推奨: Clippy チェック

コードの品質を確認するため、clippy を実行してください。

```bash
cargo clippy --all-targets --all-features
```

## コミットメッセージ

- 日本語または英語で記述
- 変更内容を簡潔に説明
- 複数の変更がある場合は箇条書きで記載
- Co-Authored-By を含める（自動化されている場合）

## ブランチ戦略

- `main`: 安定版ブランチ
- 機能追加やバグ修正は feature ブランチから PR を作成
- ブランチ名は内容が分かるように命名（例: `add-core-tests`, `fix-rsmarisa-migration`）

## PR 作成時

- タイトルは変更内容を明確に
- 本文には以下を含める：
  - Summary: 変更の概要
  - 変更内容の詳細
  - テスト結果
  - 関連する Issue があれば記載

## 開発フロー

1. main ブランチから最新版を取得: `git checkout main && git pull`
2. 新しいブランチを作成: `git checkout -b feature-name`
3. 変更を実装
4. **`cargo fmt` を実行** ← 重要！
5. テストを実行: `cargo test`
6. 変更をコミット: `git add -A && git commit -m "..."`
7. プッシュ: `git push -u origin feature-name`
8. PR を作成: `gh pr create ...`

## プロジェクト固有の注意事項

### テストについて

- libakaza の変更時は必ずテストを追加または更新
- 新機能には対応するテストを追加
- エッジケースのテストも考慮

### 依存関係の更新

- renovate が自動的に依存関係を更新
- 重要な更新は手動で確認

### ドキュメント

- README.md は最新の状態に保つ
- コード内のコメントは日本語で記述可能
- 複雑なロジックには説明コメントを追加
