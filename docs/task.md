# SSH Supporter CLI Tool Development

## Project Setup
- [x] Rustプロジェクトの初期化
- [x] 必要な依存関係の追加

## sshrコマンドの実装
- [x] SSH config パーサーの実装
- [x] パスワード抽出機能の実装
- [x] SSH接続時のパスワード自動入力機能の実装

## sshctコマンドの実装
- [x] `sshct new` - 新規接続先追加機能
  - [x] 対話型入力の実装
  - [x] SSH config への書き込み
- [x] `sshct show` - 接続先情報表示機能
  - [x] 設定情報の表示
  - [x] 接続確認と sshr 実行
- [x] `sshct edit` - 接続先編集機能
  - [x] 既存設定の読み込み
  - [x] 対話型編集の実装

## テストと検証
- [x] 各コマンドの動作確認
- [x] エラーハンドリングの確認
