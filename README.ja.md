# Flap

[English](./README.md) | [日本語](./README.ja.md)

Flapは、Rustで実装されたミニマルかつ強力なスタックベースの難解プログラミング言語（Esolang）です。

## 特徴

- 最小限のコマンドセット
- スタックベースのアーキテクチャ
- 無限ループと条件分岐のサポート
- 整数およびASCII文字のサポート

## インストール

[Rust](https://www.rust-lang.org/) がインストールされていることを確認し、以下を実行してください：

```bash
git clone https://github.com/niwatoriiiiiiiii/flap.git
cd flap
cargo build --release
```

## 使い方

`.flap` ファイルを実行するか、コードを文字列として直接渡します：

```bash
# ファイルを実行
cargo run -- examples/hello_flap.flap

# コードを直接実行
cargo run -- "10,20+p"
```

## 言語仕様

詳細な言語仕様は以下から確認できます：

- [日本語仕様書](./language_spec_jp.md)
- [英語仕様書](./language_spec_en.md)

## サンプルプログラム

[examples/](./examples) ディレクトリにいくつかのサンプルがあります：

- `hello_flap.flap`: 定番の "Hello, World!"
- `add.flap`: 足し算のデモ
- `parity.flap`: `if` コマンドを使った偶数・奇数判定
- `echo.flap`: 入力の表示
