# Flap

[English](./README.md) | [日本語](./README.ja.md)

Flapは、Rustで実装されたミニマルかつ強力なスタックベースの難解プログラミング言語（Esolang）です。

## 特徴

- 最小限のコマンドセット
- スタックベースのアーキテクチャ
- 無限ループと条件分岐のサポート
- 整数およびASCII文字のサポート

## インストール

[Releases](https://github.com/niwatoriiiiiiiii/flap/releases) からお使いのプラットフォームに合わせた `flap_installer` をダウンロードして実行してください。

または、[Rust](https://www.rust-lang.org/) がインストールされている場合は以下でビルド・インストールできます：

```bash
git clone https://github.com/niwatoriiiiiiiii/flap.git
cd flap
cargo install --path .
```

## 使い方

Flapは便利なCLIコマンドを提供しています：

```bash
# 新しいプロジェクトを初期化 (src/main.flap と flap.toml を作成)
flap init

# カレントディレクトリの src/main.flap を実行
flap run

# 対話モード (REPL) を起動
flap repl

# flap を最新バージョンにアップデート
flap update

# 指定したファイルを実行
flap <file>
```

## 言語仕様

詳細な言語仕様は以下から確認できます：

- [日本語仕様書](./language_spec_jp.md)
- [英語仕様書](./language_spec_en.md)
