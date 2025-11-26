# SSH Supporter CLI Tool 実装計画

SSH接続時のパスワード自動入力機能と、SSH設定管理機能を提供するRust製CLIツールを作成します。

## User Review Required

> [!IMPORTANT]
> このツールは `~/.ssh/config` ファイルを直接編集します。既存の設定が上書きされないよう、バックアップ機能は含まれていませんので、必要に応じて手動でバックアップを取ってください。

> [!WARNING]
> パスワードは `~/.ssh/config` 内にコメント形式 (`#pass password_here`) で平文保存されます。セキュリティリスクがあることをご理解ください。

## Proposed Changes

### プロジェクト構造

新規Rustプロジェクトとして以下の構造で作成します：

```
ssh_supporter/
├── Cargo.toml
└── src/
    ├── main.rs          # エントリーポイント
    ├── sshr.rs          # sshrコマンドの実装
    ├── sshct.rs         # sshctコマンドの実装
    ├── config.rs        # SSH config パーサー
    └── utils.rs         # 共通ユーティリティ
```

---

### 依存関係 (Cargo.toml)

#### [NEW] [Cargo.toml](file:///Users/h-abiko/06_dev_test_dir/ssh_supporter/Cargo.toml)

以下の依存関係を使用します：
- `clap`: コマンドライン引数パーサー
- `rpassword`: パスワード入力時の非表示化
- `regex`: SSH config のパースに使用
- `dialoguer`: 対話型入力UI
- `expectrl`: SSH接続時のパスワード自動入力（expect風の機能）

---

### Core Implementation

#### [NEW] [main.rs](file:///Users/h-abiko/06_dev_test_dir/ssh_supporter/src/main.rs)

メインエントリーポイント。`sshr` と `sshct` の2つのバイナリを生成するため、引数に応じて処理を分岐します。

#### [NEW] [config.rs](file:///Users/h-abiko/06_dev_test_dir/ssh_supporter/src/config.rs)

SSH config ファイル (`~/.ssh/config`) のパース・読み書き機能を実装：
- `SshConfig` 構造体: 1つのホスト設定を表現
- `parse_ssh_config()`: config ファイルをパース
- `write_ssh_config()`: config ファイルに書き込み
- `get_password()`: `#pass` コメントからパスワードを抽出

#### [NEW] [sshr.rs](file:///Users/h-abiko/06_dev_test_dir/ssh_supporter/src/sshr.rs)

`sshr` コマンドの実装：
- SSH config からパスワードを読み取り
- `expectrl` を使用してSSH接続を開始
- パスワードプロンプトを検出して自動入力

#### [NEW] [sshct.rs](file:///Users/h-abiko/06_dev_test_dir/ssh_supporter/src/sshct.rs)

`sshct` コマンドの実装：
- `sshct new`: 対話型で新規接続先を追加
- `sshct show`: 接続先情報を表示し、接続確認
- `sshct edit`: 既存接続先を対話型で編集

#### [NEW] [utils.rs](file:///Users/h-abiko/06_dev_test_dir/ssh_supporter/src/utils.rs)

共通ユーティリティ関数：
- SSH config ファイルパスの取得
- エラーハンドリング

---

### ビルド設定

2つのバイナリ (`sshr` と `sshct`) を生成するため、`Cargo.toml` に以下を設定：
```toml
[[bin]]
name = "sshr"
path = "src/main.rs"

[[bin]]
name = "sshct"
path = "src/main.rs"
```

実行時のバイナリ名で処理を分岐します。

## Verification Plan

### Automated Tests

```bash
# プロジェクトのビルド
cargo build --release

# 2つのバイナリが生成されることを確認
ls -la target/release/sshr
ls -la target/release/sshct
```

### Manual Verification

1. **sshct new のテスト**:
   ```bash
   ./target/release/sshct new test_server
   # 対話型入力で設定を追加
   # ~/.ssh/config に追加されたことを確認
   ```

2. **sshct show のテスト**:
   ```bash
   ./target/release/sshct show test_server
   # 設定情報が表示されることを確認
   ```

3. **sshct edit のテスト**:
   ```bash
   ./target/release/sshct edit test_server
   # 対話型で編集できることを確認
   ```

4. **sshr のテスト**:
   ```bash
   ./target/release/sshr test_server
   # パスワードが自動入力されてSSH接続されることを確認
   ```
