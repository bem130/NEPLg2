# Zed Extension Plan

## 目的

- Zed で NEPLg2 の syntax highlight / diagnostics / hover / definition / inlay hints を提供する。
- 解析本体は `nepl-language` を共有利用し、`nepl-web` には依存しない。

## 構成

- `nepl-language`
  - compiler 実装を再利用した editor 向け解析 lib
- `tree-sitter-neplg2`
  - syntax highlight 用 grammar
- `nepl-lsp` または同等の Rust binary
  - `nepl-language` を使って Zed / VSCode 共通の editor 機能を提供する
- `editors/zed`
  - Zed 固有の薄い package / 起動設定

## 現在

- `nepl-language` は追加済み。
- まだ Zed package 本体、grammar、language server は未実装。

## 次にやること

1. tree-sitter grammar を追加する。
2. `nepl-language` を使う Language Server binary を追加する。
3. Zed extension package からその binary を起動する。
4. VSCode extension も同じ binary を使う。
