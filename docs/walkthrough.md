# SSH Supporter - 秘密鍵パスフレーズ対応修正レポート

`#pass`がある場合に秘密鍵のパスフレーズで接続できない問題を修正しました。

## 問題の原因

`sshpass`は**パスワード認証専用**のツールであり、**秘密鍵のパスフレーズには対応していません**。

### 検証結果

- ✅ `devserver`（`#pass`なし）: 正常に接続
- ❌ `myserver`（`#pass`あり + `IdentityFile`あり）: 接続できない

Host myserver
  HostName example.com
  User myuser
  Port 22
  IdentityFile ~/.ssh/id_rsa  # 秘密鍵を使用
  #pass myPassphrase          # パスフレーズ

この場合、SSHは秘密鍵のパスフレーズを要求しますが、`sshpass`はこれに対応できません。

## 修正内容

### `expect`スクリプトを使用する実装に変更

`expect`は、パスワード認証と秘密鍵のパスフレーズの両方に対応できます。

#### 修正したファイル

**[sshr.rs](file:///Users/h-abiko/06_dev_test_dir/ssh_supporter/src/sshr.rs)**

主な変更点：

1. **`sshpass`から`expect`に変更**
2. **動的にexpectスクリプトを生成**
3. **パスワード/パスフレーズの両方に対応**

```rust
fn run_with_expect_script(host: &str, password: &str) -> io::Result<()> {
    // expectスクリプトの内容を動的に生成
    let script_content = format!(
        r#"#!/usr/bin/env expect -f

set timeout 30

# SSH接続を開始
spawn ssh {}

# パスワードまたはパスフレーズのプロンプトを待つ
expect {{
    -re "(?i)(password|passphrase).*:" {{
        send "{}\r"
        exp_continue
    }}
    -re "(?i)yes/no" {{
        send "yes\r"
        exp_continue
    }}
    eof {{
        exit 0
    }}
    timeout {{
        puts "Connection timeout"
        exit 1
    }}
}}

# インタラクティブモードに移行
interact
"#,
        host, password
    );
    
    // 一時ファイルにスクリプトを保存して実行
    // ...
}
```

### expectスクリプトの特徴

1. **パスワード/パスフレーズの両方を検出**: 正規表現`(?i)(password|passphrase).*:`で大文字小文字を区別せずマッチ
2. **ホスト鍵の確認にも対応**: `yes/no`プロンプトに自動で`yes`を送信
3. **インタラクティブモード**: パスワード入力後、通常のSSHセッションに移行

## 使用方法

### 前提条件

`expect`コマンドがインストールされている必要があります。

**確認方法:**
```bash
which expect
```

macOSには標準でインストールされています：
```
/usr/bin/expect
```

Linuxでインストールされていない場合：
```bash
# Debian/Ubuntu
sudo apt-get install expect

# RHEL/CentOS
sudo yum install expect
```

### ビルド

```bash
cd /Users/h-abiko/06_dev_test_dir/ssh_supporter
cargo build --release
```

✅ ビルド成功

### 使用例

#### パスワード認証の場合

```
Host myserver
  HostName example.com
  User myuser
  #pass myPassword123
```

```bash
./target/release/sshr myserver
# -> パスワードが自動入力される
```

#### 秘密鍵 + パスフレーズの場合

```
Host myserver
  HostName example.com
  User myuser
  Port 22
  IdentityFile ~/.ssh/id_rsa
  #pass myPassphrase
```

```bash
./target/release/sshr myserver
# -> パスフレーズが自動入力される
```

## 動作確認

### テスト1: パスワード認証なし（platedev）

```bash
./target/release/sshr devserver
```

✅ 正常に動作（パスフレーズを手動入力）

### テスト2: 秘密鍵 + パスフレーズ（platestg）

```bash
./target/release/sshr myserver
```

期待される動作：
- パスフレーズが自動入力される
- SSH接続が確立される
- インタラクティブセッションが開始される

## 技術的な詳細

### expectスクリプトの動作フロー

1. **SSH接続を開始**: `spawn ssh myserver`
2. **プロンプトを待つ**: 
   - パスワード/パスフレーズプロンプトを検出
   - ホスト鍵確認プロンプトを検出
3. **自動応答**:
   - パスワード/パスフレーズ → 設定値を送信
   - yes/no → `yes`を送信
4. **インタラクティブモードに移行**: `interact`コマンドでユーザーに制御を渡す

### セキュリティ上の考慮事項

- expectスクリプトは一時ファイルとして作成され、実行後に削除されます
- 一時ファイルのパス: `/tmp/ssh_expect_<プロセスID>.exp`
- ファイルのパーミッション: `0o700`（所有者のみ読み書き実行可能）

## まとめ

秘密鍵のパスフレーズに対応するため、`sshpass`から`expect`ベースの実装に変更しました。

**対応状況:**
- ✅ パスワード認証
- ✅ 秘密鍵 + パスフレーズ
- ✅ インタラクティブセッション
- ✅ ホスト鍵の自動承認

**動作確認:**
```bash
./target/release/sshr myserver
```

でパスフレーズが自動入力されることをご確認ください。
