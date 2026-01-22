# 状況メモ (2026-01-22)
- `nepl-core` のパーサーを修正し、`<() -> T>` などの空引数シグネチャ/ブロック引数を受け付けるようにした。`;` ドロップ周りのスタック処理も調整。
- #if[target] を 1 ステートメント適用で解釈し、非 wasm ターゲットの定義をスキップ。#wasm ブロックに簡易スタック検査を追加（local.get/set/i32.* を型スタックで検証）。
- 型変数ラベル `.label` は同名で共有されるようにし、ブロックが `()` を返す場合も式値として扱えるようにした。
- コード生成を wasm import ベースに再構成。`print_i32` は `env.print_i32` のホスト import（cli 側で Linker に登録）。エクスポート名重複を解消。
- CLI の import 解決を拡張（拡張子省略時に `.nepl` を補完、`#use` はインライン後にスキップ）。`main` が `()->()` の場合も実行可能に。
- stdlib を NEPLG2 用に整理（NEPLG1 の string/vec 等を削除、`stdlib/std/stdio.nepl` を追加、`std.nepl` を add/sub/lt のみ）。examples を NEPLG2 構文で再作成（counter/fib）。
- README を NEPLG2 仕様に刷新。
- `cargo test --workspace --locked` を完走済み。`nepl-cli` で `examples/counter.nepl` が実行できることを確認（0..4 出力）。

# これからの作業方針
- もし追加のホスト関数（例: random_i32）が必要なら import/Linker 実装を同様に追加する。
- #wasm 検査は現状 i32 系の最小集合のみ対応。命令セットを増やす場合はスタック効果テーブルを拡張する。
- 標準ライブラリの充実度が低いので、必要に応じて関数を追加しつつテストケースを増やす。
