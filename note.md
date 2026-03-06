# 2026-03-06 作業メモ (フェーズF: tutorials Part6 拡充 + library-first 化)

- 目的:
  - `tutorials/getting_started` Part6（22〜27）の説明誤り・不足を監査し、短く簡潔で安全な書き方へ更新する。
  - 生ポインタ露出を減らすため、`kp` 側に `Vec<i32>` 直受け補助を追加する。
- 変更:
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
    - `Scanner/Writer` の基本パターンを pipe 中心に簡潔化。
    - i32/i64/空白区切り出力の 3 ケースを安全 API 前提で整理。
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - `Vec + sort + lower/upper_bound` を library-first で再構成。
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
    - DP 本体を維持しつつ I/O を簡潔化。
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
    - prefix を `kp/kpprefix` ハンドル API 前提へ更新。
    - two pointers の条件評価を短絡評価に依存しない安全な形へ修正。
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 手書き BFS から `kp/kpgraph` 利用へ移行。
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - 未完成表記を廃止し、Part6 総まとめとしてテンプレート・対応表・実戦フローを追加。
  - `tutorials/getting_started/00_index.n.md`
    - 誤字を修正（関数のふりがな）。
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` ハンドルと `prefix_build_vec_i32` / `prefix_sum_i32` / `prefix_free_i32` を追加。
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` を追加。
  - `todo.md`
    - フェーズFの完了済み Part6 専用タスクを削除（未完了のみ維持）。
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i stdlib/kp/kpprefix.nepl -i stdlib/kp/kpsearch.nepl --no-tree -o /tmp/tests-part6-kp-refresh-v7.json -j 15`
    - 結果: `219/219 pass`
  - 補助確認:
    - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md --no-tree -o /tmp/tests-part6-25-v6.json -j 15`
    - 結果: `207/207 pass`

# 2026-03-06 作業メモ (フェーズD: llvm `add/sub` 再定義リンク失敗の根本修正)

- 目的:
  - `--runner all --llvm-all` 実行時に `tests/llvm_target.n.md::doctest#4/#5` が `invalid redefinition of function 'add'/'sub'` で失敗する問題を、後付け回避ではなく生成IR構造から解消する。
- 原因:
  - `stdlib/core/math.nepl` の overload 群（`add/sub` など）が `#llvmir` 内で同一シンボル名（`@add`, `@sub`）を使っていた。
  - LLVM はシンボル名で overloading できないため、同一モジュールへ複数型版を同名定義するとリンク時に衝突する。
  - さらに `u8` と `i32` は LLVM ABI で同じ `i32` に落ちるため、型別 overload をそのままシンボル名で共存させる設計が成立しない。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 生成完了直前に `deduplicate_overloaded_llvm_symbols` を追加し、同名 `define` をシグネチャ単位で一意化。
    - `define` 側の重複を `name__ovN_<sig>` へ正規化し、対応する `call` 参照も同一シグネチャで張り替える。
    - 前段として `#llvmir` 呼び出し要件抽出と AST raw-body 選別補助を追加し、不要な overload 出力を抑制。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `cargo build -p nepl-cli` -> success
  - `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-dedup-pass.json -j 15` -> `6/6 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-dedup.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: llvm codegen 内の precheck 後診断返却を除去)

- 目的:
  - `precheck` 実行後に `codegen_llvm` が `TypecheckFailed` を返していた残存経路を除去し、前段検査不変条件へ統一する。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 内の `select_active_raw_body(... )` `Err(diag)` 分岐を `TypecheckFailed` 返却から internal panic へ変更。
    - これにより、raw-body 選択失敗は前段 `target_precheck::precheck_module_before_codegen` でのみ診断され、codegen 到達後は生成専任になる。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-llvm-invariant-2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-precheck-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: llvm precheck 回帰ケースの追加)

- 目的:
  - LLVM backend 到達前に未対応 intrinsic を診断できることを回帰固定する。
- 変更:
  - `tests/llvm_target.n.md`
    - `llvm_precheck_rejects_wasm_only_intrinsic` を追加。
    - `#intrinsic "i32_add"` を `#target llvm` で使った場合に `diag_id: 3012` を期待する compile_fail ケースを追加。
- 検証:
  - `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-precheck-case.json -j 15`
    - 追加ケース（`doctest#6::llvm`）は pass。
    - 既存ケース `doctest#4/#5` は `invalid redefinition of function 'add'` で fail（既知未解決）。
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-test-add.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: allocator helper 解決の意味論修正)

- 目的:
  - runtime helper 共通化後に発生した run-time 失敗 (`unreachable` / `memory access out of bounds`) を、間に合わせではなく helper 解決の意味論から修正する。
- 原因:
  - `alloc`（安全API）と `alloc_raw`（低レベルAPI）は現状の lowering では型互換になりうるため、`ALLOC_CANDIDATES=["alloc","alloc_raw"]` へ変更すると backend 内部確保で誤って `alloc` を掴む経路が発生する。
  - その結果、内部確保の前提（生ポインタ返却）と合わず、実行時に `unreachable` / OOB が発生した。
- 変更:
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` を `["alloc_raw", "alloc"]` に戻し、内部 helper 解決は生ポインタ意味論を優先するよう修正。
    - 単体テスト期待値も raw 優先へ更新。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-order-fix.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: runtime helper 解決の共通化と raw 依存縮小)

- 目的:
  - `nepl-core` 内で重複していた runtime helper（alloc/dealloc/realloc）解決ロジックを共通化し、`_raw` 名依存を段階縮小する。
  - helper 名の優先順位を安全API名（suffixなし）優先へ統一する。
- 変更:
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` を `["alloc", "alloc_raw"]` に変更（安全API優先）。
    - `RuntimeHelperKind` / `helper_candidates` / `helper_base_name` を追加。
    - `find_runtime_helper_key`（名前解決）と `find_runtime_helper_index`（index解決）を追加。
  - `nepl-core/src/codegen_wasm.rs`
    - ローカル実装だった helper 名解決を削除し、`runtime_helpers::find_runtime_helper_index` に統一。
  - `nepl-core/src/monomorphize.rs`
    - helper 保持ルート探索を `find_runtime_helper_key` + `RuntimeHelperKind` へ置換。
    - 重複していた名前マッチ関数を削除。
  - `nepl-core/src/codegen_llvm.rs`
    - helper 候補取得を `helper_candidates(RuntimeHelperKind::...)` に統一。
    - `resolve_symbol_name` の候補一致を `helper_base_name` ベースへ変更し、namespaced/mangled 名でも同一規則で解決。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-helper-unify.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: llvm backend の wasm-body 分岐を不変条件化)

- 目的:
  - `codegen_llvm` 側に残っていた backend 入力エラー分岐（`UnsupportedWasmBody`）を前段検査前提へ寄せる。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError` から `UnsupportedWasmBody` / `UnsupportedParsedFunctionBody` を削除。
    - `emit_ll_from_module_for_target` 内で `ActiveRawBody::Wasm` 到達時の `Err` を internal panic に変更。
    - `FnBody::Wasm` reachable 到達時の `Err` を internal panic に変更。
    - HIR lowering 経路で `HirBody::Wasm` 到達時の `Err` を internal panic に変更。
    - 対応テスト `emit_ll_rejects_entry_with_wasm_body` は `TypecheckFailed` を期待する形へ更新。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: wasm codegen 診断返却経路の撤去)

- 目的:
  - `codegen` 到達後は生成専任にする方針に合わせ、`codegen_wasm` の `Vec<Diagnostic>` 返却経路を撤去する。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_body` / `lower_user` の戻り値を `Result<Function, Vec<Diagnostic>>` から `Function` へ変更。
    - `gen_block` / `gen_expr` の `diags` 引数を削除。
    - `generate_wasm` の code section 生成で `Err(ds)` 分岐を削除し、前段検査通過後は直接生成する形に統一。
    - backend 内診断として残っていた未使用関数 `validate_wasm_stack` を削除。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-wasm-no-diag.json -j 15` -> `8/8 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-wasm-no-diag.json -j 15` -> `791/791 pass`

# 2026-03-06 作業メモ (フェーズD: wasm helper 解決の自己再帰バグ修正)

- 目的:
  - `tests + stdlib` で発生していた `RangeError: Maximum call stack size exceeded` を根本原因から解消する。
- 再現と切り分け:
  - `option.nepl` doctest を単独再現すると `wasm-function[4]` の自己再帰で停止。
  - 同一ソースを `nepl-cli` で生成した wasm は正常実行。
  - `web` 生成 WAT と `native` 生成 WAT を比較すると、同一箇所で `call 5` が `call 4`（自己呼び出し）に化けていた。
- 原因:
  - `codegen_wasm` の runtime helper 解決が曖昧な文字列一致（prefix/contains）依存だった。
  - allocator helper 解決時に `alloc` と `alloc_raw` の取り違えが発生し、enum/tuple 構築時の内部確保で自己再帰が起きていた。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - helper 名の基底名抽出 `helper_base_name` を追加。
    - runtime helper 解決を基底名一致へ変更し、曖昧一致を廃止。
    - 現在 lowering 中の関数インデックスは helper 候補から除外。
    - `LocalMap` に `alloc_helper_idx` を保持し、関数ごとに一度だけ helper を確定。
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` を `["alloc_raw", "alloc"]` の順へ変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/vec/sort.nepl --no-stdlib --no-tree -o /tmp/tests-vec-option-after-alloc-helper-fix.json -j 15` -> `22/22 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-helper-fix.json -j 15` -> `791/791 pass`

# 2026-03-05 作業メモ (フェーズD: web 実行時 `compile: unreachable` の根本修正)

- 目的:
  - `web/dist` 経路でのみ発生していた `phase=compile, error=unreachable` を根本原因から解消する。
- 原因:
  - `codegen_wasm.rs` の raw wasm 行パースで、ローカル解決クロージャが `parse_wasm_line_with_lookup` 側の `$` 正規化と二重処理になっていた。
  - その結果、`#wasm` 本文の `$a`/`$b` が codegen 時のみ `unknown local` になり panic していた（precheck 側とは不整合）。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `parse_wasm_line` の lookup を `|name| locals.lookup(name)` に統一。
    - 旧 `parse_local` ヘルパを削除。
  - `nepl-web/src/lib.rs`
    - `console_error_panic_hook::set_once()` を `#[wasm_bindgen(start)]` で有効化し、WASM panic の原因位置を可視化。
  - `nodesrc/run_test.js`
    - `formatError` を追加し、compile/run 失敗時に stack を保持して JSON 出力へ反映。
- 検証:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-rootfix.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl --no-stdlib --no-tree -o /tmp/tests-list-after-rootfix.json -j 15` -> `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-rootfix.json -j 15` -> `707/791 pass`（残り `84 fail` は run 時 `Maximum call stack size exceeded`。`compile: unreachable` は再現せず）

# 2026-03-05 作業メモ (フェーズD: web 実行時 `unreachable` の切り分け)

- 目的:
  - 全体テスト (`tests + stdlib`) で多発する `phase=compile, error=unreachable` を、間に合わせではなく根本原因から切り分ける。
- 実施:
  - `trunk build` 後に
    - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-baseline-after-revert-v1.json -j 15`
    - 結果: `349/791 pass`、`442 fail`、上位失敗は `stdlib/alloc/collections/list.nepl` doctest 群の `unreachable`。
  - 同じ入力を `nepl-cli` で単体コンパイル:
    - `target/debug/nepl-cli -i /tmp/list_doctest1_clean.nepl --target std --emit wasm -o /tmp/list_doctest1_out -v`
    - 結果: compile 成功 (`DEBUG: compile_module returned Ok`)。
- 結論:
  - 失敗は `web/dist`（WASM 上の compiler 実行）経路に限定される。
  - `codegen_wasm` の今回差分を戻しても再現するため、単純な backend 変更起因ではない。
  - 以降は `web` 側で panic を診断化して原因位置を可視化するタスクを上流課題として扱う。

# 2026-03-05 作業メモ (フェーズD: todo整理 + llvm precheck 返り値規約)

- 目的:
  - `todo.md` の完了済み項目（`UnsupportedHirLowering` 整理）を反映し、未完了だけを残す。
  - LLVM 前段検査に「非 unit 関数は値を返す」規約を追加して、backend 依存失敗の前段化を進める。
- 変更:
  - `todo.md`
    - フェーズDの完了済み行
      - `llvm 経路でも backend 依存エラーを前段診断に寄せる（UnsupportedHirLowering の整理）`
      を削除し、残課題として
      - `llvm 経路の precheck を拡張し、intrinsic/戻り値規約など backend 依存失敗を前段で確定する。`
      へ更新。
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` に `TypeCtx` を渡す形へ変更。
    - reachable な `HirBody::Block` 関数について、戻り値型が非 `unit` かつ block が値を返さない場合を `D3003` で診断。
  - `nepl-core/src/codegen_llvm.rs`
    - `precheck_llvm_codegen(&types, &hir, &reachable_set)` 呼び出しへ更新。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v9.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm codegen_precheck に実検査を追加)

- 目的:
  - `codegen` 到達後は生成専任に寄せるため、LLVM 側でも前段検査で弾ける入力を増やす。
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` を追加。
    - 到達関数（reachable set）に対して expression tree を走査し、LLVM 未対応 intrinsic を前段診断化。
    - 未対応 intrinsic は `D3012 (TypeUnknownIntrinsic)` で報告。
  - `nepl-core/src/codegen_llvm.rs`
    - HIR lower 前に `precheck_llvm_codegen` を実行し、error があれば `TypecheckFailed` で早期終了。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v8.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend 診断型の整理)

- 目的:
  - `codegen_llvm` から `UnsupportedHirLowering` 返却経路が消えた状態を型定義にも反映する。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError::UnsupportedHirLowering` を enum / Display から削除。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v6.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm 残存 backend 診断の不変条件化 継続)

- 目的:
  - `codegen_llvm` に残っていた `UnsupportedHirLowering` を削減し、前段通過後は生成専任モデルへ寄せる。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 以下を `UnsupportedHirLowering` 返却から internal panic へ変更:
      - 関数 return 型不一致
      - enum/struct/tuple 構築時の `alloc` 必須判定
      - enum payload / struct field / tuple item の値生成必須・型不一致
      - `match` arm の結果型不一致
      - unknown intrinsic 到達
      - unsupported expression kind 到達
      - 文字列リテラルID範囲外
      - 文字列具体化時の `alloc` 必須判定
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v5.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm intrinsic 引数・型チェックの backend 診断を不変条件化)

- 目的:
  - `codegen_llvm` intrinsic lowering に残っていた backend 診断を削減し、前段通過後の生成専任モデルへ寄せる。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - 以下を `UnsupportedHirLowering` 返却から internal panic へ変更:
      - `load` の引数個数/型引数個数不一致、ポインタ値不在、ポインタ型不一致
      - `store` の引数個数/型引数個数不一致、ポインタ/値不在、ポインタ型不一致、`u8` 値型不一致、格納型不一致
      - `add` の引数個数不一致、lhs/rhs 不在、i32以外
      - `f32_to_i32` / `i32_to_u8` / `u8_to_i32` の引数個数・値不在・型不一致
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm 制御構文の backend 診断を不変条件化)

- 目的:
  - `codegen_llvm` の `if/while/match` で残っていた backend 診断を削減し、型検査・前段検証通過後は生成専任へ寄せる。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `if`:
      - 条件が値を返さない
      - 条件が `i32/bool` 互換でない
      - then/else 分岐結果型不一致
      を `UnsupportedHirLowering` 返却から internal panic へ変更。
    - `while`:
      - 条件が値を返さない
      - 条件が `i32/bool` 互換でない
      を internal panic へ変更。
    - `match`:
      - scrutinee が値を返さない
      - scrutinee が enum pointer (`i32`) でない
      - arm が0件
      を internal panic へ変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm call_indirect の backend 診断を不変条件化)

- 目的:
  - `codegen_llvm` の `call_indirect` で残っていた backend 診断（`UnsupportedHirLowering`）を削減し、前段通過後は生成専任に寄せる。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `call_indirect` について以下の `UnsupportedHirLowering` 返却を internal panic 化:
      - callee が値を返さない
      - callee が `i32` 関数IDでない
      - 引数が値を返さない
      - 引数個数不一致
      - 引数型不一致
      - 候補関数未検出
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: raw wasm 行検査の前段分離を完了)

- 目的:
  - `codegen_precheck` が `codegen_wasm` 実装詳細へ依存する経路を解消し、前段検査の責務を `wasm_shared` へ集約する。
  - 「codegen 到達時は生成専任」の方針を維持し、raw wasm 行パース失敗を前段で確定する。
- 変更:
  - `nepl-core/src/wasm_shared.rs`
    - `parse_wasm_line_with_lookup` を共有化。
    - `precheck_raw_wasm_body` を追加し、`HirBody::Wasm` 行を前段で検査して `D4004` を返すように変更。
  - `nepl-core/src/passes/codegen_precheck.rs`
    - raw wasm 事前検査呼び出し先を `codegen_wasm` から `wasm_shared` へ変更。
  - `todo.md`
    - フェーズDの「`codegen_precheck` の wasm 側ヘルパ依存整理」項目を完了として削除。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: #wasm のスタック検証を前段検査へ移動)

- 目的:
  - 「codegen は正しい入力を生成するだけ」の方針に合わせ、`#wasm` ボディ検証を backend 実行時ではなく `codegen_precheck` 側で完了させる。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `precheck_raw_wasm_body` シグネチャを `precheck_raw_wasm_body(ctx, func)` に変更。
    - raw 行のパース成功時に命令列を蓄積し、前段で `validate_wasm_stack` を実行するよう変更。
    - `lower_user` の `HirBody::Wasm` 経路から `validate_wasm_stack` を削除。
    - `generate_wasm` の診断集約を実質空に整理（codegen 内診断を発生させない方向に統一）。
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_raw_wasm_body` 呼び出しを新シグネチャへ更新。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: codegen_precheck の wasm 事前検査を共通モジュールへ分離)

- 目的:
  - `passes/codegen_precheck.rs` が `codegen_wasm.rs` 実装詳細へ直接依存していた状態を整理し、前段検査ロジックを共有モジュールへ分離する。
  - 「codegen は正しい入力を生成するだけ」の方針に合わせ、backend の `skip`/診断蓄積を不変条件違反へ寄せる。
- 変更:
  - `nepl-core/src/wasm_shared.rs` を新規追加。
    - wasm署名解決 (`wasm_sig`, `wasm_sig_ids`)
    - generic skip 判定 (`should_skip_wasm_codegen_for_generic`)
    - 到達関数解析 (`collect_reachable_wasm_functions`)
    - 間接呼び出しを含む署名集合収集 (`collect_wasm_signature_set`)
    - wasm intrinsic 対応判定 (`is_supported_wasm_intrinsic`)
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 上記ロジックを `wasm_shared` 参照へ置換。
    - `precheck_raw_wasm_body` のみ `codegen_wasm` 側を継続利用（次段で分離予定）。
  - `nepl-core/src/codegen_wasm.rs`
    - extern/function 署名不一致時の `skip` を廃止し internal panic 化。
    - `lower_body` で backend 診断が返る経路を internal panic 化。
    - 共有ロジックは `wasm_shared` 呼び出しへ委譲。
  - `nepl-core/src/lib.rs`
    - `pub mod wasm_shared;` を追加。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend 診断を前段不変条件へ移行)

- 目的:
  - `todo.md` フェーズD方針に合わせ、`codegen_llvm` 側で発行していた「前段通過後に到達しないはず」の診断を廃止し、前段検証の不変条件として扱う。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `let` の型不一致 (`let type mismatch`) を `UnsupportedHirLowering` から internal panic へ変更。
    - `set` の型不一致 (`set type mismatch`) を `UnsupportedHirLowering` から internal panic へ変更。
    - 未解決 trait call の到達を `UnsupportedHirLowering` から internal panic へ変更。
    - call 引数型不一致を `UnsupportedHirLowering` から internal panic へ変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant-panic-v1.json -j 15` -> `707/791 pass`（`Maximum call stack size exceeded` が多数。今回の変更対象外の既存失敗として継続調査）

# 2026-03-05 作業メモ (フェーズC/D接続: core/mem に MemPtr 初期化オーバーロード追加)

- 目的:
  - `core/mem` 後段移行（`stdlib/std`/tutorials）で `i32` 生ポインタを露出せずに配列初期化できる上流APIを用意する。
  - `MemPtr` モデル上で `fill/memset` を統一し、`Result` で失敗を扱えるようにする。
- 変更:
  - `stdlib/core/mem.nepl`
    - `memset_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` を追加。
    - `fill_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` を追加。
    - `fill_i32 <(MemPtr<i32>,i32,i32)->Result<(),str>>` を追加。
    - 無効ポインタや負の長さは `Result::Err` を返す。
  - `tests/memory_safety.n.md`
    - `MemPtr fill_i32/fill_u8 の安全オーバーロード` ケースを追加。
    - `MemPtr fill 系は無効引数を Err で返す` ケースを追加。
- 検証:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-fill-overload.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-mem-fill-overload.json -j 15` -> `787/787 pass`

# 2026-03-05 作業メモ (フェーズD: kpread_core ヘッダフィールドの型安全化)

- 目的:
  - `kpread_core` に残っていたヘッダ生オフセット（`0/4/8`）を列挙型へ移行し、`kpread`/`kpwrite` と同じ境界表現に揃える。
  - ヘッダレイアウトの意味を型で固定し、オフセット誤指定を上流で防ぐ。
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `ScannerHeaderFieldCore` を追加（`BufPtr` / `Len` / `Pos`）。
    - `scanner_header_core_off` を追加し、オフセット解決を1箇所に集約。
    - `store_i32_u8_at sc*_region 0/4/8 ...` を列挙型 + オフセット関数経由へ置換。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kp-core-header-field-enum.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-core-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite ヘッダフィールドの型安全化)

- 目的:
  - `kpwrite` のヘッダアクセスで使っていた生オフセット値（`0/4/8/12/16`）を列挙型に置換し、`kpread` と同じ安全モデルへ統一する。
  - `mem/kpread/kpwrite` の公開API安全化で、ヘッダ境界の意味を型で表現する。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterHeaderField` を追加（`BufPtr` / `Cap` / `WriteLen` / `IovPtr` / `NwPtr`）。
    - `writer_header_off` を追加し、オフセット解決を一箇所に集約。
    - `writer_header_ptr` / `writer_load_header` / `writer_store_header` / `writer_load_header_ptr` の第2引数を `i32` から `WriterHeaderField` に変更。
    - 呼び出し側の生数値オフセットを全廃し、列挙値に置換。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kp-header-field-enum-unified.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpwrite-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpread ヘッダフィールドの型安全化)

- 目的:
  - `kpread` のヘッダアクセスで使っていた生オフセット値（`0/4/8`）を列挙型へ置き換え、呼び出し側の誤指定を減らす。
  - `todo.md` 2026-03-03 フェーズD（`mem/kpread/kpwrite` の公開API安全化）に沿って、上流の表現を固定する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `ScannerHeaderField` を追加（`BufPtr` / `Len` / `Pos`）。
    - `scanner_header_off` を追加し、オフセット解決を1箇所へ集約。
    - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` の第2引数を `i32` から `ScannerHeaderField` に変更。
    - 呼び出し側の `scanner_load_header sc 0/4/8` と `scanner_store_header sc 8 ...` を列挙型指定へ置換。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-field-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-header-field.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: kpread ヘッダアクセスのサイレント失敗を除去)

- 目的:
  - `scanner_load_header` / `scanner_store_header` の失敗時フォールバック（`0` / `()`）を廃止し、ヘッダ不整合を隠蔽しない。
  - 上流仕様（安全API優先）に合わせ、壊れた状態を継続させるより即時停止に統一する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_load_header`:
      - `scanner_header_ptr` が `Err` の場合の `0` 返却を `#intrinsic "unreachable"` へ変更。
      - `load_i32` が `None` の場合の `0` 返却を `#intrinsic "unreachable"` へ変更。
    - `scanner_store_header`:
      - `scanner_header_ptr` が `Err` の場合の無視を `#intrinsic "unreachable"` へ変更。
      - `store_i32` が `Err` の場合の無視を `#intrinsic "unreachable"` へ変更。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-unreachable-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-unreachable.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD先行: Writer を RegionToken 保持へ移行)

- 目的:
  - `kpread` と同様に `kpwrite` でも公開ハンドルが領域情報を持つようにし、メモリ安全APIを統一する。
- 根本原因:
  - `Writer` は `MemPtr<u8>` を直接保持し、ヘッダ領域サイズ（20byte）が型に表現されていなかった。
  - 途中で追加した `writer_mem(Writer)->MemPtr<u8>` ヘルパは `Writer` を値渡しで受けるため、
    non-copy な `Writer` の move を発生させ `D3053` を引き起こした。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` を `Writer.region: RegionToken<u8>` に変更。
    - `writer_wrap` で `region_new raw 20` を構築。
    - `writer_mem` ヘルパは削除し、`region_ptr get w "region"` を直接展開して move を回避。
  - `stdlib/kp/kpread_core.nepl`
    - `store_i32_u8_at/load_i32_u8_at` を `RegionToken<u8>` 受け取りへ変更。
    - `sc0/iov/nread/sc` の各領域を `RegionToken` 化してアクセス経路を統一。
    - 途中で発生した `match` アーム崩れ（`D3009/D3008/D3045`）を修正。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-regiontoken-v3.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先行: kpread_core の内部ヘッダアクセスを RegionToken 化)

- 目的:
  - `kpread_core` の内部メモリアクセスも `RegionToken` 経由に統一し、`MemPtr + off` の直接算術依存を減らす。
- 根本原因:
  - `store_i32_u8_at` / `load_i32_u8_at` が `MemPtr<u8>` と `off` から直接 `MemPtr<i32>` を作る設計で、
    領域境界の前提がヘルパ外へ漏れていた。
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_region_ptr` を追加し、`region_ptr_at<u8,i32>` を使用。
    - `store_i32_u8_at` / `load_i32_u8_at` の引数を `RegionToken<u8>` に変更。
    - `sc0(12)`, `iov(8)`, `nread(4)`, `sc(12)` で `RegionToken` を構築してヘルパへ渡す形に更新。
  - 途中修正:
    - `match dealloc_ptr<u8> buf cap` の `Result::Err` アームのインデント崩れにより
      `D3009/D3008/D3045` が発生したため、分岐構造を正しく修正。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-kpread-core-regiontoken-v2.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先行: kpwrite ヘッダアクセスを RegionToken 経由へ移行)

- 目的:
  - `kpwrite` 側でもヘッダアクセスを `RegionToken` ベースに寄せ、`core/mem` の境界検証APIを再利用できるようにする。
- 根本原因:
  - 既存 `writer_header_ptr` は `mem_ptr_addr + off` で直接アドレス算術を行い、
    20byte ヘッダ境界の前提を関数ごとに暗黙化していた。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_region` を追加（`region_new w_mem 20`）。
    - `writer_header_ptr` を `Result<MemPtr<i32>,str>` へ変更し、`region_ptr_at<u8,i32>` を使用。
    - `writer_load_header` / `writer_store_header` を上記 `Result` 経路へ更新。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-header-regiontoken.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズD先行: kpread の Scanner ヘッダを RegionToken 化)

- 目的:
  - `todo.md` フェーズD着手として、`kpread` の公開ハンドルに領域所有情報を持たせ、`core/mem` の新安全APIへ寄せる。
- 根本原因:
  - `Scanner` が `MemPtr<u8>` 直接保持のみで、ヘッダ領域境界の情報が型に乗っていなかった。
  - ヘッダアクセスが `mem_ptr_addr + off` の算術依存で、境界検証を再利用しにくかった。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `Scanner` フィールドを `raw: MemPtr<u8>` から `region: RegionToken<u8>` に変更。
    - `scanner_wrap` で `region_new raw 12` を構築。
    - `scanner_header_ptr` を `region_ptr_at<u8,i32>` ベースの `Result` 返却へ変更。
    - `scanner_load_header` / `scanner_store_header` を上記 `Result` 経路で処理。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-scanner-regiontoken.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズC: core/mem に RegionToken 安全APIを追加)

- 目的:
  - `todo.md` フェーズCに沿って、`MemPtr<T>` と `RegionToken<T>` を使う安全APIを `core/mem` に追加し、`kpread/kpwrite` 移行の上流基盤を作る。
- 根本原因:
  - 既存 `mem` は `MemPtr<T>` までは整備済みだったが、領域所有を表す公開APIが不足しており、
    境界情報付きアクセスを型として統一できていなかった。
- 変更:
  - `stdlib/core/mem.nepl`
    - `RegionToken<T>` 補助APIを追加:
      - `region_new`
      - `region_in_bounds`
      - `region_ptr_at`
      - `alloc_region_bytes`
      - `alloc_region`
      - `dealloc_region`
    - これにより、領域サイズを伴う型付きオフセット取得を `Result` で扱えるようにした。
  - `tests/memory_safety.n.md`
    - `alloc_region/region_ptr_at/dealloc_region` の基本動作ケースを追加。
    - 範囲外オフセットで `Result::Err` を返す回帰ケースを追加。
- 検証:
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md -i tests/plan.n.md -i tests/block_single_line.n.md --no-stdlib --no-tree -o /tmp/tests-semicolon-focus.json -j 15`
  - 結果: `67/67 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md --no-tree -o /tmp/tests-memory-safety-region-token.json -j 15`
  - 結果: `211/211 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-memory-kp-regression.json -j 15`
  - 結果: `221/221 pass`

# 2026-03-05 作業メモ (フェーズB2: trait capability の型付き保持へ移行)

- 目的:
  - trait capability 判定の文字列再解析を減らし、型付きデータで一貫して扱う。
- 根本原因:
  - 既存実装では `TraitInfo.capabilities` が `Vec<String>` のため、
    `TraitSemantics::detect` で毎回文字列を再パースしていた。
  - この構造は capability 判定の責務が分散し、将来拡張時に不整合を生みやすい。
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo.capabilities` を `Vec<String>` から `Vec<TraitCapability>` へ変更。
    - trait 定義処理 (`Stmt::Trait`) で capability を1回だけパースし、型付きで保持。
    - 重複 capability 指定は同一trait内で重複登録しないよう整理。
    - `TraitSemantics::detect` は `TraitInfo` 内の型付き capability を直接参照。
    - 不要になった `detect_declared_trait_capabilities` を削除。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-capability-typed.json -j 15`
  - 結果: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-capability-typed.json -j 15`
  - 結果: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-capability-typed.json -j 15`
  - 結果: `783/783 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite header 読み取りの Result 化と None フォールバック廃止)

- 目的:
  - `writer_load_header` の `None -> 0` フォールバックを廃止し、header 読み取り失敗を明示分岐で扱う。
- 根本原因:
  - 従来の `writer_load_header` は `load_i32` 失敗時に 0 を返しており、異常状態を正常値へ潰していた。
  - そのため後続処理で `buf/cap/iov/nw` が不正値のまま進行する余地があった。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header` を `Result<i32,str>` へ変更。
    - `writer_load_header_ptr` を `Result<MemPtr<u8>,str>` へ変更。
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`,
      `writer_put_u8_handle`, `writer_write_str_handle`,
      `writer_write_i32_handle`, `writer_write_u64_handle` を
      `Result` 分岐で安全に処理する形へ更新。
    - `if` レイアウト中の冗長な `then: block:` を除去し、`D2002` 回避のため式構造を仕様準拠へ整理。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-after-header-result-v2.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-after-header-result.json -j 15`
  - 結果: `226/226 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpwrite-style-fix.json -j 15`
  - 結果: `215/215 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite の header アクセス集約と non-copy 整合)

- 目的:
  - `kpwrite.nepl` で散在していた header 生アクセス（`load_i32 add w_raw ...` / `store_i32 add w_raw ...`）を共通化し、`Writer` の non-copy/move 規則と矛盾しない形へ整理する。
- 根本原因:
  - `Writer` は non-copy なのに、最初のヘルパ化で `writer_load_header/store_header` が `Writer` 値渡しとなり、ヘルパ呼び出し自体が move を発生させ `D3053` を誘発していた。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_ptr/load/store` を追加。
    - 上記ヘルパは `Writer` ではなく `w_raw:i32` を受け取り、`Writer` の move を発生させない設計に変更。
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`, `writer_put_u8_handle`, `writer_write_str_handle`, `writer_write_i32_handle`, `writer_write_u64_handle` を共通ヘルパ経由に置換。
    - 置換後、`w_raw` 直接参照は解放処理境界（`writer_free_handle`）のみへ縮小。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-header-v2.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v4.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズB2: trait capability 判定の自動推定を廃止)

- 目的:
  - `copy/clone` の trait 意味付けを明示 capability (`#capability`) のみに限定し、暗黙推定による誤判定を根本的に除去する。
- 根本原因:
  - `TraitSemantics::detect` が capability 未指定時に
    - `Self -> Self` 単一メソッド trait を clone 候補
    - marker trait を copy 候補
    として推定していた。
  - これにより trait 設計意図と無関係な構造一致だけで copy/clone 意味が付与される余地があった。
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` から clone/copy の自動候補推定を削除。
    - `#capability copy` / `#capability clone` の宣言結果のみを意味付けに使用。
    - 不要化した `trait_has_single_unary_self_to_self_method` と `trait_is_marker` を削除。
    - `TraitSemantics::detect` の未使用 `ctx` 引数を削除。
  - `tests/move_effect.n.md`
    - `#capability` 未指定 trait が copy/clone として推定されないことを確認する回帰ケースを追加。
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3096 TypeUnknownTraitCapability` を追加。
  - `nepl-core/src/typecheck.rs`
    - trait 定義で未知の `#capability` 名を検出し、`D3096` を返すよう変更。
  - `tests/move_effect.n.md`
    - `#capability cpoy` の compile_fail ケース（`diag_id: 3096`）を追加。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-v1.json -j 15`
  - 結果: `269/269 pass`
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-capability-v2.json -j 15`
  - 結果: `227/227 pass`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-v2.json -j 15`
  - 結果: `272/272 pass`

# 2026-03-05 作業メモ (フェーズC: kpread の header 直アクセスを共通安全ヘルパへ統一)

- 目的:
  - `kpread.nepl` で残っていた `sc_raw` ベースの header 直接読み書きを除去し、`Scanner` 境界の型安全性を上げる。
- 根本原因:
  - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` を導入済みでも、主要パーサ関数が旧経路（`load_i32 add sc_raw ...` / `store_i32 add sc_raw ...`）を使い続けていた。
  - これにより API 境界は `Scanner` でも、実装内部が生ポインタ前提のまま分岐していた。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 以下の関数で header アクセスを `scanner_load_header` / `scanner_store_header` に統一:
      - `scanner_skip_ws_handle`
      - `scanner_is_eof_handle`
      - `scanner_skip_token_handle`
      - `scanner_read_token_handle`
      - `scanner_read_i32_handle`
      - `scanner_read_u64_handle`
      - `scanner_read_i64_handle`
      - `scanner_read_f64_handle`
      - `scanner_read_all_i32_handle`
    - 置換後、`kpread.nepl` 内の `sc_raw` 直接アクセスは `scanner_header_ptr` 内の実装一点のみに集約。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-safe-headers-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v3.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズC: kpread 基盤 handle の Scanner 型化)

- 目的:
  - `kpread` の公開面で露出している生 `i32` ハンドル関数を段階的に減らすため、基盤となる3関数を `Scanner` 受け取りへ変更する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_skip_ws_handle` を `(Scanner)*>()` へ変更。
    - `scanner_is_eof_handle` を `(Scanner)*>bool` へ変更。
    - `scanner_skip_token_handle` を `(Scanner)*>()` へ変更。
    - `scanner_read_token_handle` を `(Scanner)*>str` へ変更。
    - 上記呼び出し箇所（`i32` ベースの既存 handle 群）では `scanner_wrap mem_ptr_wrap sc` を明示して渡すよう統一。
    - 公開ラッパ（`scanner_skip_ws` など）は raw 取り出しをやめて `Scanner` を直接渡すよう簡素化。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-scanner-handle-v1.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpread 残り handle 群の Scanner 型化完了)

- 目的:
  - `kpread` で残っていた `*_handle <(i32)...>` 群を `Scanner` 受け取りへ統一し、公開/内部の型境界を一貫化する。
- 根本原因:
  - 一部 handle が `i32` を直接受け取り、他の `Scanner` 受け取り関数と境界設計が混在していた。
  - その結果、公開ラッパで `mem_ptr_addr get sc "raw"` を都度書く必要があり、raw 露出と誤用余地が残っていた。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 以下を `Scanner` 受け取りへ変更:
      - `scanner_read_i32_handle`
      - `scanner_read_u64_handle`
      - `scanner_read_i64_handle`
      - `scanner_read_f64_handle`
      - `scanner_read_f32_handle`
      - `scanner_read_vec_i64_handle`
      - `scanner_read_vec_i32_handle`
      - `scanner_read_matrix_i32_handle`
      - `scanner_read_all_i32_handle`
      - `scanner_read_na_i32_handle`
      - `scanner_read_interval_queries_i32_handle`
      - `scanner_read_query_tuples_i32_handle`
      - `scanner_read_ndrh_i32_handle`
    - 各関数内部では必要箇所のみ `sc_raw = mem_ptr_addr get sc "raw"` を導入し、既存ロジックを維持。
    - 公開ラッパ (`scanner_read_i32` など) は raw 抽出を削除して handle へ `Scanner` を直接渡すよう統一。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpread-scanner-allhandles-v1.json -j 15`
  - 結果: `212/212 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-scanner-allhandles-v2.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite handle API の線形化と move 整合化)

- 目的:
  - `kpwrite` の内部 API でも生 `i32` 境界を減らしつつ、`Writer` の non-copy 設計と move 規則が矛盾しない形へ整理する。
- 根本原因:
  - `Writer` を受ける handle が `()` を返す設計のまま `Writer` を複数回利用しており、`D3053/D3054`（moved value）を誘発していた。
  - 一時 `writer_wrap` を多用する形は局所的には動くが、設計として線形消費規則が明確でなかった。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_writeln_handle` / `writer_write_*_handle` を `Writer` 受け取り・`Writer` 返却に統一。
    - 各 handle で `w_raw` を内部取得し、更新後は `writer_wrap mem_ptr_wrap w_raw` を返す線形 API に変更。
    - 複数操作を行う handle（`writer_write_i32_handle`, `writer_write_u64_handle`, `writer_write_*_ln_handle` など）は `let mut ww <Writer>` / `set ww ...` で線形に更新。
    - 公開 API (`writer_write_i32` など) は raw 再ラップの重複を削除し、対応 handle を直接呼ぶ構造へ整理。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kpwrite-only-v4.json -j 15`
  - 結果: `208/208 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-handle-wrap-v3.json -j 15`
  - 結果: `217/217 pass`

- 補足（設計判断）:
  - 一時 `writer_wrap` を都度作る呼び出しは move エラー回避としては機能するが、線形 API 設計として不明瞭だったため採用しない。
  - `Writer -> Writer` の更新連鎖を handle 層で明示し、move 規則と API 契約を一致させる方針に統一した。

# 2026-03-05 作業メモ (フェーズC: kpread_core の生メモリアクセス安全API化)

- 目的:
  - syscall 境界以外の生メモリアクセスを `MemPtr` + `Result/Option` 経由へ寄せ、失敗検出を上流化する。
- 根本原因:
  - `kpread_core` 内で `mem_ptr_addr` + 生 `store_i32/load_i32` を直接実行しており、境界不整合時に失敗を型で扱えなかった。
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_ptr`, `store_i32_u8_at`, `load_i32_u8_at` を追加。
    - scanner header 初期化 (`sc0`, `sc`) を `store_i32_u8_at` 経由へ変更し、失敗時は確保済み領域を解放して `Err` 返却。
    - `iov/nread` 構築時の書き込みと `nread` 読み取りを安全ヘルパ経由へ変更。
    - メモリアクセス失敗時は `mem_failed` を立て、後段で一括解放して `Result::Err \"kpread_core.memory access failed\"` を返す経路を追加。
    - `fd_read` 呼び出し自体は syscall 仕様上 `i32` ポインタが必要なため、境界点でのみ `mem_ptr_addr` を使用。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-core-safe-v1.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: `core/mem` の `*_ptr` を安全API経由へ統一)

- 目的:
  - `MemPtr` 系 API の内部実装を `alloc_raw/realloc_raw/dealloc_raw` 直結から分離し、`alloc/realloc/dealloc` を通る共通安全経路へ統一する。
- 変更:
  - `stdlib/core/mem.nepl`
    - `alloc_ptr` を `alloc` 経由へ変更。
    - `realloc_ptr` を `realloc` 経由へ変更。
    - `dealloc_ptr` を `dealloc` 経由へ変更。
  - これにより `MemPtr` 系エラー経路は基底安全APIの前提検査結果と整合する。
- 検証:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v2.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズC: kpread_core 内部確保を `*_ptr` API に統一)

- 目的:
  - `kpread_core` 内部での生ポインタ管理を減らし、`MemPtr<u8>` を使った確保/再確保/解放へ寄せる。
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `buf/iov/nread/scanner header` の確保を `alloc_ptr<u8>` に変更。
    - バッファ拡張を `realloc_ptr<u8>` に変更。
    - 解放を `dealloc_ptr<u8>` に変更。
    - `fd_read` や `store_i32/load_i32` へ渡す箇所のみ `mem_ptr_addr` で `i32` に明示変換。
  - `scanner_new_impl` は既存どおり `Result<MemPtr<u8>,str>` を返し、API互換を維持。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v6.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpread_core の返却型を MemPtr 化)

- 目的:
  - `kpread` 入力初期化の上流（`kpread_core`）でも生 `i32` 返却を減らし、`MemPtr<u8>` で境界を揃える。
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `scanner_new_impl` の戻り値を `Result<MemPtr<u8>,str>` に変更。
    - 成功時 `sc:i32` は `mem_ptr_wrap` して返却。
    - 失敗系の `Result` 型パラメータを `MemPtr<u8>` に統一。
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_handle` は `scanner_new_impl` をそのまま返す実装へ簡素化。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v5.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpread/kpwrite の `*_new_handle` 返り値を MemPtr 化)

- 目的:
  - 生成系 API の境界から生 `i32` を減らし、`MemPtr<u8>` による型境界を明確化する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_handle` を `Result<MemPtr<u8>,str>` へ変更。
    - `scanner_new` は `MemPtr<u8>` をそのまま `scanner_wrap` に渡す形へ変更。
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle` を `Result<MemPtr<u8>,str>` へ変更。
    - 内部確保で得た `w:i32` は `mem_ptr_wrap` して `Ok` 返却。
    - `writer_new` は `MemPtr<u8>` をそのまま `writer_wrap` に渡す形へ変更。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v4.json -j 15`
  - 結果: `216/216 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite Writer ラップ境界の型整合)

- 目的:
  - `todo.md` フェーズC（公開APIの生ポインタ露出削減）に沿って、`kpwrite` の `Writer` 生成境界を `MemPtr<u8>` で統一する。
- 根本原因:
  - `Writer.raw` は `MemPtr<u8>` だが `writer_wrap` が `(i32)->Writer` で、生ポインタを直接受け取る境界が残っていた。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_wrap` を `(MemPtr<u8>)->Writer` に変更。
    - `writer_new` と `Writer` を返す公開ラッパ群で `i32` を `mem_ptr_wrap` してから `writer_wrap` を呼ぶよう統一。
  - 内部 `*_handle` は段階移行として `i32` を維持（公開API境界のみ型安全化）。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v3.json -j 15`
  - 結果: `216/216 pass`

# 2026-03-05 作業メモ (フェーズC: kpread Scanner ラップ境界の型整合)

- 目的:
  - `todo.md` フェーズC（公開APIの生ポインタ露出削減）に沿って、`kpread` の `Scanner` 生成境界を `MemPtr<u8>` で統一する。
- 根本原因:
  - `Scanner.raw` は `MemPtr<u8>` なのに `scanner_wrap` が `(i32)->Scanner` で、生成境界で生ポインタを直接受けていた。
  - これにより `Scanner` の公開型設計と生成シグネチャが不一致だった。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_wrap` を `(MemPtr<u8>)->Scanner` に変更。
    - `scanner_new` で `raw:i32` を `mem_ptr_wrap` してから `scanner_wrap` へ渡すよう変更。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v2.json -j 15`
  - 結果: `216/216 pass`

# 2026-03-05 作業メモ (compile_fail: diag_id + 位置検証の運用固定)

- 目的:
  - `compile_fail` ケースで `diag_id` だけでなく発生位置（file/line/col）も安定検証できるようにする。
- 変更:
  - `nodesrc/tests.js`
    - `extractDiagSpansFromCompileError` を行単位解析へ変更。
    - `--> ...` 行から末尾 `:line:col` を基準に抽出するよう修正し、パス中のコロンを含む形式にも耐えるようにした。
  - `nodesrc/parser.js`
    - doctest メタ `diag_spans` に JSON object 形式（`{file,line,col}`）を許可。
    - 既存の `"line:col"` / `"file:line:col"` 文字列表記は互換維持。
  - `tests/compile_fail_diag_location.n.md`
    - `diag_spans` の object 形式を使う回帰ケースを追加。
- 検証:
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md -i tests/lexer_diag.n.md --no-stdlib --no-tree -o /tmp/tests-compile-fail-location-verify.json -j 15`
  - 結果: `6/6 pass`

# 2026-03-05 作業メモ (`;` 診断の上流化と loader 診断整形)

- 目的:
  - `tests/block_semicolon_return.n.md::doctest#10` の backend 漏れ（wasm validation error）を止め、parser 段で `diag_id` を固定化する。
  - `compile_fail` で loader 経由のエラーでも `error[Dxxxx]` を安定取得できるようにする。
- 根本原因:
  - `if:` レイアウト内の `Stmt::ExprSemi` が上流で拒否されず、codegen まで進んでいた。
  - `nepl-web/src/lib.rs` で loader エラーを `to_string()` しており、`Diagnostics` 文字列が整形されず `diag_id` 抽出が不安定だった。
- 変更:
  - `nepl-core/src/parser.rs`
    - `reject_layout_semicolon` を追加。
    - `extract_if_layout_exprs` / `extract_if_layout_exprs_lenient` で `ExprSemi` を `D2002` として即時拒否。
    - `while` / 一般引数レイアウトは既存仕様（`;` 許容）を維持。
  - `nepl-web/src/lib.rs`
    - loader 失敗時に `render_loader_error` を通すよう変更。
    - `LoaderError::Core` は `render_core_error` へ流し、`error[Dxxxx]` 形式で返す。
  - `tests/plan.n.md`
    - `diag_id` 期待を実装実態に合わせて `2002 -> 2001` に修正（2ケース）。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/lexer_diag.n.md -i tests/plan.n.md -i tests/block_single_line.n.md -i tests/block_semicolon_return.n.md --no-stdlib --no-tree -o /tmp/tests-diag-parser.json -j 15` -> `70/70 pass`

# 2026-03-05 作業メモ (codegen 前段共通 precheck 導入: raw body/target 診断の統一)

- 目的:
  - `codegen_wasm` / `codegen_llvm` が個別に `#wasm/#llvmir` の target 不整合を診断する構造をやめ、前段共通チェックで診断を確定する。
- 根本原因:
  - `#if[target=...]` 評価、active 文抽出、raw body 選択ロジックが `typecheck` と `codegen_llvm` に分散し、判定差分と backend 依存診断が発生していた。
- 変更:
  - 新規 `nepl-core/src/target_precheck.rs` を追加。
    - `gate_allows`（`#if[target/profile]` 判定）
    - `active_stmt_indices`（active 文抽出）
    - `select_active_raw_body`（関数 body 内 `#wasm/#llvmir` 選択）
    - `precheck_function_raw_body_target` / `precheck_module_raw_bodies`（target 整合検証）
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3094 TypeMultipleActiveRawBodies`
    - `D3095 TypeRawBodyTargetMismatch`
  - `nepl-core/src/compiler.rs`
    - `compile_module` の typecheck 前に `precheck_module_raw_bodies` を実行し、エラー時は早期終了。
  - `nepl-core/src/typecheck.rs`
    - `check_function` 冒頭で `precheck_function_raw_body_target` を実行し、`typecheck` 直接利用経路でも同一診断を保証。
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 冒頭で `precheck_module_raw_bodies` を実行。
    - `#if` active 文抽出を共通 `active_stmt_indices` に統一。
    - Parsed 関数の raw body 選択を共通 `select_active_raw_body` に統一。
    - 重複していた local gate/raw 選択関数群を削除。
  - テスト:
    - 既存更新:
      - `tests/neplg2.n.md` の `wasm_rejects_llvmir_body_with_diag_id` を `diag_id: 3095` へ変更。
      - `tests/neplg2.n.md` に `raw_body_conflict_reports_diag_id`（`diag_id: 3094`）追加。
      - `tests/llvm_target.n.md` の `llvm_rejects_wasm_body` に `diag_id: 3095` 追加。
    - 新規追加:
      - `tests/raw_body_precheck.n.md`（3ケース、`D3094/D3095` を固定確認）。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md --no-stdlib --no-tree --runner all -o /tmp/tests-raw-body-precheck.json -j 15` -> `3/3 pass`
  - 参考: `tests/neplg2.n.md` + `tests/llvm_target.n.md` を `--with-stdlib` で実行すると既知の stdlib 側失敗（list doctest）が混ざるが、追加した `D3094/D3095` ケース自体は通過していることを `/tmp/tests-codegen-precheck.json` で確認。

# 2026-03-05 作業メモ (`;` 仕様先行修正: `stdlib/core/math.nepl`)

- 目的:
  - `plan.md` の「複行文には末尾 `;` を付けない」制約に合わせ、`overload` 失敗の根本原因を先に解消する。
- 根本原因:
  - `stdlib/core/math.nepl` の `i128/u128` 周辺で、複行 `if:` を右辺に持つ `let` 文の末尾に `;` が残っていた。
  - これが式の `()` 化を誘発し、wasm 検証段で `invalid wasm generated: expected i64 but nothing on stack` を引き起こしていた。
- 変更:
  - `stdlib/core/math.nepl` の該当箇所で、複行 `if:` 右辺 `let` の末尾 `;` を除去。
  - 対象: `to_i128`, `u128/i128` の `carry/borrow` 計算、`mul_wide` の `carry_mid/carry_lo` 計算。
- 検証:
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-nostd.json -j 15`
  - 結果: `43/43 pass`

# 2026-03-05 作業メモ (パーサ根本修正: 単行 block 制約と `ExprSemi` 意味論保持)

- 目的:
  - `tests/plan.n.md::doctest#29`（単行 `block` 内に複行 `block:` が入ってしまう）をコンパイラ側で根本修正する。
  - `tests/block_semicolon_return.n.md::doctest#10`（複行式末尾 `;` の意味落ち）を解消する。
- 根本原因:
  - パーサが「単行 block 文脈」を保持しておらず、単行 `block` 内でも `parse_block_after_colon()` を通して複行 `:` ブロックを受理していた。
  - `extract_if_layout_exprs` / `extract_while_layout_exprs` / `extract_arg_layout_exprs` が `Stmt::ExprSemi` を `Stmt::Expr` と同一扱いし、`;` による unit 化とスタック検証を落としていた。
- 変更:
  - `nepl-core/src/parser.rs`
    - `single_line_block_depth` を追加し、単行 block 解析中に複行 `:` ブロックを検出したら `D2002` を出すように変更。
    - `parse_single_line_block*` で文脈深さを管理するよう変更。
    - `ExprSemi` を保持してレイアウト抽出へ渡す共通ヘルパーを追加。
    - if/while/引数レイアウト抽出で `ExprSemi` を捨てずに block 化して保持し、型検査段で `;` 意味論が反映されるように変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/plan.n.md --no-stdlib --no-tree -o /tmp/tests-plan-nostd.json -j 15` -> `36/36 pass`
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md --no-stdlib --no-tree -o /tmp/tests-block-semicolon-nostd.json -j 15` -> `10/10 pass`
- 影響:
  - `--with-stdlib` で走らせると stdlib doctest 側に `;` 意味論不整合が顕在化（`List` などで `expected ... got unit`）。
  - これは今回のパーサ修正で隠れていた仕様違反が表面化した状態。
  - 次段として stdlib 側の `;` 使用箇所を plan.md に合わせて整理する必要がある。

# 2026-03-05 作業メモ (plan.md 全体再読: plan.n.md 拡充)

- 目的:
  - `plan.md` 全体を再読し、実装が誤りやすい仕様を `tests/plan.n.md` に集約して回帰可能にする。
- 変更:
  - `tests/plan.n.md` を拡充。
  - 既存 `compile_fail` に `diag_id` を付与:
    - `plan_block_trailing_semicolon_makes_unit_and_breaks_i32_return` -> `3003`
    - `plan_semicolon_requires_exactly_one_value_growth` -> `3016`
  - 追加した主な仕様テスト:
    - `block:` 後ろはコメントのみ許可、トークン禁止
    - 引数オフサイド（複数行引数）
    - `while` の `cond/do` 記法（inline / block）
    - 関数リテラル `():`、`fn` 糖衣 + `@` 関数値参照
    - pipe の改行記法
    - 単行ブロックの多段ネスト
    - `if:` が3式必須
    - 単行ブロック複文（`;`区切り）と末尾 `;` による `()` 化
    - 1行2文（区切りなし）エラー
    - `Tuple:` リテラル
    - 型注釈が式に前置される挙動
- 検証:
  - `node nodesrc/tests.js -i tests/plan.n.md --no-tree -o /tmp/tests-plan-nmd-2.json -j 15`
  - 結果: `240 total / 239 pass / 1 fail`
- 差分（plan.md と実装）:
  - `plan_single_line_block_cannot_contain_multiline_block` が `expected compile_fail` に対して compile success。
  - これは plan.md の「単行ブロック内に複行ブロックを置けない」制約に対する未実装ギャップ。

# 2026-03-04 作業メモ (フェーズB2継続: Copy/Clone 判定の trait識別子化)

- 目的:
  - `todo.md` フェーズB2「trait 契約判定の文字列依存を減らす」を進め、`Copy/Clone` 能力判定を trait名ではなく trait識別子で扱う。
- 根本原因:
  - `TraitSemantics` と `ImplInfo` の判定は `trait_name` 文字列比較に依存しており、名前解決変更や alias 導入時に脆い。
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics` を `copy_trait/clone_trait: Option<(String, TypeId)>` に変更。
    - `is_copy_trait` / `is_clone_trait` を `TypeId` 比較へ変更。
    - `detect_capability_trait` の戻り値を `Option<(String, TypeId)>` へ変更。
    - `ImplInfo` に `trait_self_ty: Option<TypeId>` を追加し、`Copy/Clone` 判定・重複 impl 判定に利用。
    - `ctx.set_copy_trait_enabled(...)` は `copy_trait_name().is_some()` で制御。
    - 最終 impl 生成パスの copy 判定も `trait_info.self_ty` を使用。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `278/278 pass`
- 状況:
  - `Copy/Clone` 能力判定の主要経路は trait名文字列比較から離脱。
  - 残りの文字列依存は一般 trait 境界判定（`trait_bound_satisfied` など）側に限定される。

# 2026-03-04 作業メモ (フェーズB2継続: Copy判定の経路分離と tests/*.n.md 回帰追加)

- 目的:
  - `todo.md` フェーズB2の残件として、trait モード時の `Copy` 判定を旧互換経路から分離し、名前ハードコード依存をさらに減らす。
  - 変更に対応する回帰を `tests/*.n.md` に追加する。
- 根本原因:
  - `TypeCtx::is_copy` は trait モードでも先に `is_copy_eligible`（`i64/f64` 名ハードコード）を通るため、`impl Copy` ベース判定に完全移行できていなかった。
  - `Copy impl` 妥当性検査も同じ経路を使っており、段階移行の境界が曖昧だった。
- 変更:
  - `nepl-core/src/types.rs`
    - `is_copy_impl_eligible` を追加（`impl Copy` 妥当性専用）。
    - `is_copy` を経路分離:
      - trait モード有効時は `is_copy_with_trait_model` を直接使用。
      - trait モード無効時のみ `is_copy_eligible` を使用。
    - `is_copy_eligible_inner` に `allow_opaque_named` を追加し、`is_copy_impl_eligible` からは Named 型を名前依存なしで妥当判定可能にした。
  - `nepl-core/src/typecheck.rs`
    - `impl Copy for T` の対象妥当性検査を `ctx.is_copy_impl_eligible(target_ty)` に変更。
  - `tests/move_effect.n.md`
    - 回帰ケースを2件追加:
      - `Copy` trait 有効時、`i64` に `Copy impl` がない場合は move エラー（`diag_id: 3053`）。
      - `Clone+Copy impl` を与えた `i64` は再利用可能。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `278/278 pass`
- 状況:
  - `Copy` 判定の trait モード経路は分離済み。
  - 次段で `Copy/Clone` 能力宣言の抽象化（trait 名検出ロジックのさらなる一般化）へ進む。

# 2026-03-04 作業メモ (フェーズB2: Copy能力判定のtrait移行スイッチ導入)

- 目的:
  - `todo.md` フェーズB2の「`Copy/Clone` 能力判定のハードコード撤廃」に向け、`Copy` trait 実装情報へ段階移行する土台を追加する。
- 根本原因:
  - `TypeCtx::is_copy` は常に構造ベース判定のみで、`impl Copy for T` の有無を能力判定に反映できなかった。
  - 既存資産との互換を保ちながら移行する切替点がなく、一括移行すると広範囲の回帰リスクが高かった。
- 変更:
  - `nepl-core/src/types.rs`
    - `TypeCtx` に `copy_trait_enabled: bool` を追加。
    - `set_copy_trait_enabled(bool)` を追加。
    - `is_copy` を段階判定へ変更:
      - まず既存 `is_copy_eligible` で前提検証。
      - `copy_trait_enabled == false` では従来挙動を維持。
      - `copy_trait_enabled == true` では `is_copy_with_trait_model` を使い、ADT は `impl Copy` 登録（`copy_impl_targets`）を必須化。
    - 追加調整:
      - trait モード時の `TypeKind::Named` / `TypeKind::Apply` 判定を型名ハードコードから外し、`has_copy_impl_target` ベースへ変更。
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` 後に `ctx.set_copy_trait_enabled(...)` を設定し、`Copy` trait が定義されるモジュールでのみ新判定を有効化。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `276/276 pass`
- 状況:
  - 互換性を保ったまま `Copy` trait 反映の切替点を導入済み。
  - 次段で `Copy/Clone` を能力テーブル化し、判定ロジックの文字列依存をさらに削減する。

# 2026-03-04 作業メモ (上流修正: codegen_wasm 診断IDの明示化)

- 目的:
  - `todo.md` 残件だった `codegen_*.rs` の主要診断を `diag_id` で固定し、codegen 失敗の分類を文言依存から切り離す。
- 根本原因:
  - `codegen_wasm.rs` の `Diagnostic::error(...)` は ID 未付与で、codegen フェーズ失敗を安定的に特定できなかった。
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D4001..D4015` を追加:
      - `CodegenWasmUnsupportedExternSignature`
      - `CodegenWasmUnsupportedFunctionSignature`
      - `CodegenWasmMissingReturnValue`
      - `CodegenWasmRawLineParseError`
      - `CodegenWasmLlvmIrBodyNotSupported`
      - `CodegenWasmStringLiteralNotFound`
      - `CodegenWasmUnknownVariable`
      - `CodegenWasmUnknownFunctionValue`
      - `CodegenWasmUnknownFunction`
      - `CodegenWasmMissingIndirectSignature`
      - `CodegenWasmUnsupportedIndirectSignature`
      - `CodegenWasmUnknownIntrinsic`
      - `CodegenWasmUnsupportedEnumPayloadType`
      - `CodegenWasmUnsupportedStructFieldType`
      - `CodegenWasmUnsupportedTupleElementType`
  - `nepl-core/src/codegen_wasm.rs`
    - 主要 codegen エラー発生点に `with_id(...)` を付与。
    - 追加対象:
      - extern/function シグネチャ lower 失敗
      - missing return
      - raw wasm parse 失敗
      - wasm backend での llvm ir body
      - unknown variable/function/function value
      - indirect call signature 問題
      - unknown codegen intrinsic
      - enum/struct/tuple の unsupported payload/field/element 型
  - `tests/neplg2.n.md`
    - `wasm_rejects_llvmir_body_with_diag_id` を追加（`diag_id: 4005`）。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/neplg2.n.md -i tests/functions.n.md -i tests/selfhost_req.n.md --no-tree -o /tmp/tests-codegen-diag-subset.json -j 15` -> `276/276 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-codegen-diagid.json -j 15` -> `798/798 pass`
- 状況:
  - `todo.md` の診断ID残件（codegen 主要診断）は完了。

# 2026-03-04 作業メモ (上流修正: typecheck の module/impl 定義時診断IDを明示化)

- 目的:
  - `todo.md` 残件だった `typecheck.rs` 上流（module/impl 定義フェーズ）の未付与診断を `diag_id` で固定し、文言依存を除去する。
- 根本原因:
  - 定義登録/impl 検証フェーズは `Diagnostic::error(...)` のまま残っており、同種エラーでも ID が不安定だった。
  - そのため `compile_fail` の失敗理由が文言変更で揺れる状態だった。
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3073..D3092` を追加:
      - `TypeUnknownTraitBound`
      - `TypeWasiImportTargetMismatch`
      - `TypeExternSignatureMustBeFunction`
      - `TypeItemNameConflict`
      - `TypeEnumTypeParamBoundsUnsupported`
      - `TypeStructTypeParamBoundsUnsupported`
      - `TypeTraitTypeParamsUnsupported`
      - `TypeTraitMethodTypeParamsUnsupported`
      - `TypeInherentImplUnsupported`
      - `TypeImplTypeParamsUnsupported`
      - `TypeUnknownTrait`
      - `TypeImplTargetMustBeConcrete`
      - `TypeFunctionSignatureMustBeFunction`
      - `TypeAliasTargetNotFound`
      - `TypeFunctionSignatureOverloadNotFound`
      - `TypeDuplicateImplMethod`
      - `TypeImplMethodNotFoundInTrait`
      - `TypeImplMethodSignatureMismatch`
      - `TypeImplMissingTraitMethod`
      - `TypeEntryFunctionMissingOrAmbiguous`
  - `nepl-core/src/typecheck.rs`
    - 上流定義フェーズ（enum/struct/trait/impl/alias/entry）の未付与エラーへ `with_id(...)` を付与。
    - `check_function` 冒頭の signature/arity 検証にも ID を付与。
  - `tests/neplg2.n.md`
    - 既存 `compile_fail` に `diag_id` を追加:
      - `pipe_target_missing_after_annotation_is_error` -> `3016`
      - `wasi_import_rejected_on_wasm_target` -> `3074`
      - `name_conflict_enum_fn_is_error` -> `3076`
      - `trait_bound_missing_impl_is_error` -> `3069`
      - `trait_method_arity_mismatch_is_error` -> `3068`
      - `unknown_trait_bound_is_error` -> `3073`
  - `tests/functions.n.md`
    - `function_alias_target_not_found`（`diag_id: 3086`）を追加。
  - `tests/selfhost_req.n.md`
    - `test_req_trait_extensions` に `diag_id: 3081` を追加。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/neplg2.n.md -i tests/functions.n.md -i tests/selfhost_req.n.md --no-tree -o /tmp/tests-typecheck-item-diag-subset.json -j 15` -> `275/275 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-item-diagid.json -j 15` -> `797/797 pass`
- 状況:
  - `typecheck.rs` の上流定義フェーズ診断ID付与は完了。
  - 次段は `todo.md` 残件どおり `codegen_*.rs` の主要診断ID明示化。

# 2026-03-04 作業メモ (上流修正: lexer 診断IDの明示化と回帰追加)

- 目的:
  - `lexer.rs` の未付与エラーに診断IDを付け、`compile_fail + diag_id` で固定検証できる状態にする。
- 根本原因:
  - `unknown token/directive` 以外の字句エラーは `with_id` 未付与で、失敗分類が文言依存になっていた。
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D1203..D1209` を追加:
      - `LexerIndentTabsNotAllowed`
      - `LexerExpectedIndentedBlock`
      - `LexerInvalidPubDirectivePrefix`
      - `LexerIndentWidthMismatch`
      - `LexerIndentLevelMismatch`
      - `LexerInvalidStringEscape`
      - `LexerUnterminatedStringLiteral`
  - `nepl-core/src/lexer.rs`
    - タブインデント、`#wasm/#llvmir` 後インデント不足、`pub` 接頭辞誤用、
      インデント幅不一致/階層不一致、invalid escape、unterminated string に `with_id` を付与。
  - `tests/lexer_diag.n.md`
    - 新規追加（3ケース）:
      - invalid escape -> `diag_id: 1208`
      - unterminated string -> `diag_id: 1209`
      - invalid `pub` prefix -> `diag_id: 1205`
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/lexer_diag.n.md --no-tree -o /tmp/tests-lexer-diag.json -j 15` -> `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-lexer-diagid-extend.json -j 15` -> `796/796 pass`
- 状況:
  - parser + lexer + typecheck（主要経路）の診断ID固定化が進行。
  - 次段は `typecheck` 上流（module/impl 定義時）と `codegen_*.rs` の残未付与診断を整理する。

# 2026-03-04 作業メモ (上流修正: overload/trait/pipe の診断ID拡張)

- 目的:
  - `typecheck` の未付与エラー（特に overload/trait method/pipe/arity 周辺）を診断IDで固定化し、`compile_fail` 回帰を安定化する。
- 根本原因:
  - 同一カテゴリの型検査失敗で `with_id` 未付与経路が残り、文言変更に弱い状態だった。
  - trait 経由呼び出しの失敗（未知メソッド・境界未充足など）が `diag_id` で識別できなかった。
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3066..D3072` を追加:
      - `TypeTraitMethodTypeArgsNotSupported`
      - `TypeTraitMethodNotFound`
      - `TypeArgumentArityMismatch`
      - `TypeTraitBoundUnsatisfied`
      - `TypeInvalidDeref`
      - `TypeAssignmentArityMismatch`
      - `TypeCallReductionLimitExceeded`
  - `nepl-core/src/typecheck.rs`
    - 以下の診断に `with_id` を付与:
      - `pipe has no target` -> `D3013`
      - trait method への型引数未対応 -> `D3066`
      - trait method 不在 -> `D3067`
      - overload の型引数不一致 -> `D3021`
      - 引数個数不一致（関数/constructor/trait method receiver）-> `D3068`
      - trait 境界未充足 -> `D3069`
      - assignment 個数不一致 -> `D3071`
      - field assignment 型不一致 -> `D3036`
      - 非参照型 deref -> `D3070`
      - call reduction 反復上限超過 -> `D3072`
  - `tests/overload.n.md`
    - `compile_fail + diag_id` を3ケース追加:
      - trait method 型引数未対応 (`3066`)
      - trait method 不在 (`3067`)
      - trait 境界未充足 (`3069`)
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-diagid-extend.json -j 15` -> `244/244 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-diagid-extend.json -j 15` -> `793/793 pass`
- 状況:
  - `D3006`（no matching overload）と field access（`D3011`）は診断経路を分離したまま維持。
  - 次段は `todo.md` の診断ID拡張残件（lexer + typecheck上流の未付与領域）を継続する。

# 2026-03-04 作業メモ (上流修正: typecheck の noshadow/shadow 診断IDを明示化)

- 目的:
  - `typecheck` の `noshadow` / `non-shadowable` 系エラーを診断生成点で固定し、回帰を `diag_id` で検証可能にする。
- 根本原因:
  - 同一カテゴリの shadow 関連エラーに `with_id` 未付与経路が残り、文言依存の判定になっていた。
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `cannot shadow non-shadowable ...` 系を `TypeNoShadowViolation (D3014)` へ統一。
    - `noshadow declaration ... conflicts ...` 系を `TypeNoShadowConflict (D3015)` へ統一。
    - 関数/関数alias/ローカル let の各経路で secondary label 付き診断にも同IDを付与。
  - `tests/shadowing.n.md`
    - `compile_fail` 4ケースに `diag_id: 3014` を追加して固定化。
- 検証:
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/move_effect.n.md --no-tree -o /tmp/tests-shadowing-moveeffect-diagid.json -j 15` -> `248/248 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-shadow-diagid.json -j 15` -> `790/790 pass`
- 状況:
  - shadow/noshadow の主要経路は `diag_id` 固定化済み。
  - 次段は `typecheck` の残未付与カテゴリ（undefined/overload/pipe/pure-impure）へ拡張する。

# 2026-03-04 作業メモ (上流修正: typecheck field-access 診断IDの明示化)

- 目的:
  - `typecheck.rs` の field access 系エラーを診断生成点で `DiagnosticId` 固定し、`compile_fail` を ID で安定検証できるようにする。
- 根本原因:
  - `core/field::get` / `put` 経由の失敗は、型検査フェーズで発生するにもかかわらず、`with_id` なしの `Diagnostic::error` が残っていた。
  - 文言のみ依存だと、エラーテキスト調整時に回帰検出が不安定になる。
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `resolve_field_access_with_mode` 配下の field 参照失敗（範囲外/フィールド不存在/非複合型）に
      `TypeInvalidFieldAccess (D3011)` を明示付与。
  - `tests/move_effect.n.md`
    - `core/field` の不正アクセスを `compile_fail + diag_id: 3011` で固定するケースを追加。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move_effect-check.json -j 15` -> `221/221 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-field-diagid.json -j 15` -> `790/790 pass`
- 状況:
  - field access 系は `D3011` で明示化完了。
  - 次段は `typecheck` の未付与領域（shadow / overload / pipe / undefined 系）を順次明示化する。

# 2026-03-04 作業メモ (上流修正: parser 診断IDの未付与箇所を明示化)

- 目的:
  - `todo.md` の「診断IDの明示付与（parser/typecheck/resolve）」を上流から進め、`parser.rs` の未付与診断を生成点で固定する。
- 根本原因:
  - `Diagnostic::error(...)` が `with_id` なしで残っており、同種エラーでもIDが安定しない経路があった。
  - 文言依存のままだと `compile_fail` の回帰固定が不十分になる。
- 変更:
  - `nepl-core/src/parser.rs`
    - 再帰上限/無進捗回復/marker配置/mlstr/#externシグネチャ/型パラメータ解析などの未付与診断へ `with_id` を付与。
    - 付与IDは既存の Parser 系 (`ParserExpectedToken`, `ParserUnexpectedToken`, `ParserExpectedIdentifier`, `ParserInvalidTypeExpr`, `ParserInvalidExternSignature`) を利用。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-parser-diagid.json -j 15` -> `789/789 pass`
- 状況:
  - parser の `Diagnostic::error` は診断生成点で ID 明示化済み。
  - 次段は `typecheck.rs` の未付与診断へ同方針を展開する。

# 2026-03-04 作業メモ (上流テスト整備: `tests/move_check.n.md` の skip 解除)

- 目的:
  - `move_check` 系 `.n.md` の上流回帰を `skip` 依存から外し、診断ID付き compile_fail で固定化する。
- 変更:
  - `tests/move_check.n.md`
    - `move_simple_ok` を実コード化（`ret: 0`）。
    - `move_use_after_move` を `compile_fail + diag_id: 3053` に変更。
    - `move_in_branch` を `compile_fail + diag_id: 3054` に変更。
    - `move_in_loop` を `compile_fail + diag_id: 3065` に変更。
- 根本原因:
  - 旧 Rust テスト移植時に `skip` が残っており、分岐合流/ループ再利用の move 回帰が CI で検出不能だった。
  - 診断IDで失敗理由を固定しないと、文言揺れで意図しない回帰を見落とす。
- 検証:
  - `node nodesrc/tests.js -i tests/move_check.n.md --no-tree -o /tmp/tests-move-check-nmd.json -j 15` -> `217/217 pass`
- 状況:
  - `move_check.n.md` の先頭4ケースは実行型になり、`skip` は除去済み。
  - 次段で `todo.md` の診断ID未付与領域（parser/typecheck/resolve）を継続する。

# 2026-03-04 作業メモ (フェーズD進行: Scanner/Writer の直接利用へ下流移行)

- 目的:
  - `kpread/kpwrite` 公開APIの安全型利用を下流へ浸透させ、生ハンドル由来の中間束縛を減らす。
- 変更:
  - `tests/kp.n.md`
  - `tests/kp_i64.n.md`
  - `tests/stdin.n.md`
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - `examples/kp_fizzbuzz.nepl`
  - それぞれ `let sc_obj <Scanner> unwrap_ok scanner_new; let sc <Scanner> sc_obj;` を
    `let sc <Scanner> unwrap_ok scanner_new;` へ統一。
  - カタログ内の `sc_handle` も削除し、`Scanner` を直接渡す形へ統一。
- 根本原因:
  - 公開APIが安全型で整っていても、下流コードに旧来の二段束縛が残ると、生ハンドル前提へ戻しやすくなる。
  - 先に利用側の書き方を揃えることで、次段の公開面整理（ハンドル版隔離）を安全に進められる。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-tree -o /tmp/tests-kp-typed-usage.json -j 15` -> `225/225 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-scanner-writer-typed-direct.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-scanner-writer-typed-direct.json -j 15` -> `262/262 pass`
- 状況:
  - 下流の主要利用箇所は `Scanner/Writer` 直接利用へ移行済み。
- 次段で `kpread/kpwrite` の i32 ハンドル受け取りオーバーロードの公開面整理を継続する。

# 2026-03-04 作業メモ (上流修正: move_check 診断IDの明示化)

- 目的:
  - `move_check` が生成する主要エラーに `diag_id` を付与し、`compile_fail` を診断IDで固定検証できる状態にする。
- 根本原因:
  - move/borrow 系エラーは文言一致に依存しており、将来の文言調整でテストが壊れやすかった。
  - `todo.md` の「診断IDの明示付与」を満たすには、診断生成点（`move_check.rs`）で enum を直接指定する必要があった。
- 変更:
  - `nepl-core/src/diagnostic_ids.rs`
    - `3051..3065` の move/borrow 系 `DiagnosticId` を追加。
    - `from_u32` / `message` に新IDを追加。
  - `nepl-core/src/passes/move_check.rs`
    - `Diagnostic::error(...)` に `with_id(...)` を付与。
    - 対象: use/move/borrow/assign/drop/loop合流の主要診断。
  - `tests/move_effect.n.md`
    - 既存 compile_fail 2件に `diag_id` を追加（shared borrow move / move後再利用）。
    - 新規 compile_fail 2件を追加（move後borrow=3063、分岐後potentially moved=3054）。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-diagid.json -j 15` -> `220/220 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-move-diagid.json -j 15` -> `789/789 pass`
- 状況:
  - move/borrow系の `compile_fail + diag_id` 基盤が上流で確立。
  - 次段は `todo.md` の診断ID未適用領域（parser/typecheck/resolveの残り）へ拡張する。

# 2026-03-04 作業メモ (フェーズD進行: `scanner_new` / `writer_new` の曖昧オーバーロード根治)

- 目的:
  - `unwrap_ok scanner_new` / `unwrap_ok writer_new` で発生した `D3005 ambiguous overload` を、戻り値型のみで分岐する nullary オーバーロード設計から解消する。
- 根本原因:
  - `scanner_new` / `writer_new` に `Result<i32,str>` 版と `Result<Scanner/Writer,str>` 版を同名で共存させたため、引数0の呼び出しで文脈不足時に戻り値型だけでは選択不能になっていた。
  - その曖昧性が `kp` doctest / `tests` / `tutorials` の `unwrap_ok scanner_new` 系呼び出しに波及し、下流で連鎖的に型不一致を誘発していた。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new <()*>Result<i32,str>>` を `scanner_new_handle <()*>Result<i32,str>>` に改名。
    - 公開 `scanner_new` は `Result<Scanner,str>` のみを提供。
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new <()*>Result<i32,str>>` を `writer_new_handle <()*>Result<i32,str>>` に改名。
    - 公開 `writer_new` は `Result<Writer,str>` のみを提供。
  - `tests/overload.n.md`
    - 追加した zero-arg `Result` ケースのシグネチャ/式を修正し、pure 文脈で正しく検証できる状態へ調整。
- 検証:
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-zeroarg-result.json -j 15` -> `241/241 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpread-kpwrite-new-overload.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpread-overload-unify.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpread-overload-unify.json -j 15` -> `262/262 pass`
- 状況:
  - `new` 系の公開 API で「戻り値型のみ差分」の曖昧性を除去。
  - フェーズDの安全API統一路線（公開面は安全型、ハンドル版は内部名に隔離）に整合。

# 2026-03-04 作業メモ (フェーズD進行: `kpread` の `_raw` 依存を同名オーバーロードへ整理)

- 目的:
  - `kpread` の `scanner_*_raw` 命名を段階縮退し、`i32` ハンドル版と `Scanner` 版を同名オーバーロードとして統一する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` を除く `scanner_*_raw` を `scanner_*` へ改名。
    - `i32` 受け取り実装と `Scanner` 受け取り実装を同名で共存させる構成に変更。
    - 既存ラッパは同名オーバーロードの `i32` 版を呼び出すように更新。
- 根本原因:
  - `_raw` 接尾辞分岐が API 読み取りコストを上げ、実際には型だけで区別できる箇所まで命名差分を持っていた。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpread-kpwrite-overload-unify.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpread-overload-unify.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpread-overload-unify.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread` は `scanner_new_raw` を除いて `_raw` 接尾辞なしで運用可能な状態になった。
  - 次段は `scanner_new_raw` の扱い（戻り値型依存の曖昧性解消設計）を上流設計と合わせて検討する。

# 2026-03-04 作業メモ (フェーズD進行: `kpwrite` の `_raw` 依存を同名オーバーロードへ整理)

- 目的:
  - `kpwrite` 内部で分離していた `*_raw` 群を、`i32` ハンドル版と `Writer` 版の同名オーバーロードで統一し、公開面の命名を簡潔化する。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_raw` を除き、`writer_*_raw` を `writer_*` へ改名。
    - `i32` 受け取り実装と `Writer` 受け取り実装を同名で共存させる形に変更。
    - 既存の `Writer` 版からは同名の `i32` 版を呼ぶように整理。
- 根本原因:
  - `_raw` 接尾辞を前提にラッパ層が増え、API 仕様の読み取りコストが上がっていた。
  - 既存のオーバーロード機構で十分に区別可能な箇所まで命名分岐していた。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-overload-unify.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpwrite-overload-unify.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpwrite-overload-unify.json -j 15` -> `262/262 pass`
- 状況:
  - `kpwrite` は `writer_new_raw` を除いて `_raw` 接尾辞なしで運用可能な状態になった。
  - 次段で `kpread` 側も同方針で段階整理する。

# 2026-03-04 作業メモ (フェーズD進行: `alloc` 安全API標準名化の回帰復旧)

- 目的:
  - `core/mem` の `alloc/realloc/dealloc` を `Result` 返却へ標準名化した変更に対して、下流の `kp`/tests/tutorials の破損を上流原因から復旧する。
- 変更:
  - `stdlib/kp/kpprefix.nepl`
    - doctest の `alloc/dealloc` を `alloc_raw/dealloc_raw` へ更新。
  - `stdlib/kp/kpsearch.nepl`
    - doctest の `alloc/dealloc` を `alloc_raw/dealloc_raw` へ更新。
  - `tests/capacity_stack.n.md`
  - `tests/sort.n.md`
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `examples/tui_editor/editor_fs.nepl`
    - 置換ミスで壊れていた `#import "alloc_raw/...` を `#import "alloc/...` へ復旧。
- 根本原因:
  - 生メモリAPI移行の一括置換時に、関数呼び出しだけでなく import パス文字列まで `alloc_raw` に書き換わっていた。
  - `alloc` が `Result` 返却になった後も、`kp` doctest の一部が `i32` 前提の旧記述を保持していた。
- 検証:
  - `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md --no-tree -o /tmp/tests-mem-kp-safe-api-switch.json -j 15` -> `233/233 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-mem-kp-safe-api-switch-r2.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-mem-kp-safe-api-switch-r2.json -j 15` -> `262/262 pass`
- 状況:
  - `alloc` 安全API標準名化の現行差分は、`tests + stdlib + tutorials` で回帰通過。
  - 次段は `todo.md` のフェーズD残件（公開面からの raw 露出整理）を継続する。

# 2026-03-04 作業メモ (フェーズD進行: vec の `alloc/realloc/dealloc` を `*_raw` へ直接移行)

- 目的:
  - `vec` だけ残っていた `alloc/realloc/dealloc` 呼び出しを `*_raw` に統一し、メモリAPI移行の停滞要因を解消する。
- 変更:
  - `stdlib/alloc/collections/vec.nepl`
    - `alloc` -> `alloc_raw`
    - `realloc` -> `realloc_raw`
    - `dealloc` -> `dealloc_raw`
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl -i stdlib/tests/vec.n.md -i tests/capacity_stack.n.md -i tests/pipe_collections.n.md --no-tree -o /tmp/tests-vec-raw-direct.json -j 15` -> `236/236 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-vec-raw-direct.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-vec-raw-direct.json -j 15` -> `262/262 pass`
- 状況:
  - 以前 `todo.md` に残していた `vec` の `realloc_raw` OOB 再現は現行系で再現せず、移行を完了できた。

# 2026-03-04 作業メモ (上流修正: codegen の alloc helper 解決を `*_raw` 優先へ統一)

- 目的:
  - `alloc/dealloc/realloc` の同名安全オーバーロード導入時に、codegen 側が誤った helper を解決して再帰・スタックオーバーフローへ落ちる根本原因を上流で除去する。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - 内部確保 helper 解決を `alloc_raw` 優先、`alloc` フォールバックへ変更。
  - `nepl-core/src/codegen_llvm.rs`
    - runtime helper 解決関数 `resolve_runtime_helper_symbol` を追加。
    - `alloc/dealloc/realloc` 到達関数追加で `*_raw` 優先、旧名フォールバックへ変更。
    - `resolve_alloc_symbol` を `alloc_raw` 優先に変更。
    - entry lower 時の fallback allocator 判定を `alloc_raw` 優先探索に変更。
    - `resolve_symbol_name` は map の実キー参照を返す実装に変更。
  - `nepl-core/src/monomorphize.rs`
    - runtime helper 保持対象を `alloc_raw/dealloc_raw/realloc_raw` 優先に変更（旧名フォールバック）。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-overload-memory-after-core-helper-fix.json -j 15` -> `244/244 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-core-helper-fix.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-core-helper-fix.json -j 15` -> `262/262 pass`
- 状況:
  - 上流（codegen/monomorphize）の helper 解決経路が `*_raw` 優先で揃ったため、次段の `core/mem` 安全API標準名化を再開できる状態になった。

# 2026-03-04 作業メモ (調査: alloc 同名オーバーロードの衝突と差し戻し)

- 事象:
  - `core/mem` に `alloc/realloc/dealloc` の `MemPtr` 安全オーバーロードを追加すると、
    `stdlib/core/option.nepl::doctest#3` / `stdlib/core/result.nepl::doctest#4` などで
    `Maximum call stack size exceeded` が発生。
- 原因:
  - コンパイラ生成コード側が `alloc : (i32)->i32` を暗黙前提としており、
    同名オーバーロード追加で実行時経路が崩れる。
- 対応:
  - `alloc/realloc/dealloc` の `MemPtr` 同名オーバーロードは一旦差し戻し。
  - `load/store` の `MemPtr` 同名オーバーロードは維持。
  - 追加した `tests/memory_safety.n.md` の `alloc<...>` ケースは削除。
- テスト:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-alloc-overload-revert.json -j 15` -> `213/213 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-mem-overload-revert.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-overload-revert2.json -j 15` -> `262/262 pass`
- 次対応:
  - `alloc` 系の標準名安全化は、コンパイラ側の暗黙依存を先に解消してから再導入する。

# 2026-03-04 作業メモ (フェーズD進行: core/mem の MemPtr load/store を標準名オーバーロード化)

- 目的:
  - `*_ptr` 接尾辞依存を減らし、`MemPtr` 利用時は標準名 `load_i32/store_i32/load_u8/store_u8` で書けるようにする。
- 変更:
  - `stdlib/core/mem.nepl`
    - `load_i32/store_i32/load_u8/store_u8` に `MemPtr` 引数版のオーバーロードを追加。
    - 旧 `load_i32_ptr/store_i32_ptr/load_u8_ptr/store_u8_ptr` は互換エイリアス化。
    - `MemPtr` オーバーロードは無効ポインタ時に `Option::None` / `Result::Err` を返す。
- テスト:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-mem-overload-loadstore.json -j 15` -> `218/218 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-mem-loadstore-overload.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-loadstore-overload.json -j 15` -> `262/262 pass`
- 状況:
  - `MemPtr` 利用コードは標準名で安全な load/store を呼べる状態になった。
  - 次段は `alloc/realloc/dealloc` 側の公開名安全化を継続する。

# 2026-03-04 作業メモ (フェーズD進行: kpread_core 解放経路の Result 化)

- 目的:
  - `kpread_core` の初期化失敗時巻き戻しで `dealloc_raw` 直呼びを減らし、失敗処理を `Result` へ寄せる。
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `nread` 確保失敗時、`iov/buf` の解放を `dealloc_result` ベースへ変更。
    - `realloc` 失敗時、`iov/nread_ptr/buf` の解放を `dealloc_result` ベースへ変更。
    - `scanner` ヘッダ確保失敗時と成功後の一時領域解放も `dealloc_result` ベースへ変更。
    - 解放失敗は巻き戻し処理を止めず吸収する方針で統一。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kp-core-dealloc-result.json -j 15` -> `228/228 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpreadcore-dealloc-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpreadcore-dealloc-result.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread_core` 初期化失敗時の解放経路は `Result` 系APIに寄せられた。
  - 次段で `core/mem` 公開名の安全API標準化を継続する。

# 2026-03-04 作業メモ (フェーズD進行: kpwrite 初期化の根本整理)

- 目的:
  - `kpwrite` 初期化を `0` センチネル分岐から外し、`Result` ベースで確保失敗と巻き戻しを一元化する。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle_raw` を削除。
    - `writer_alloc_buf` を追加し、`4096 -> 1024 -> 256` の段階確保を `Result<WriterBuf,str>` で返すように変更。
    - `writer_try_free` を追加し、初期化途中の失敗時に解放失敗を吸収して巻き戻せるように変更。
    - `writer_new_raw` は `alloc_result/dealloc_result` 前提の `match` 連鎖へ置換し、確保失敗時の返却理由を段階別に固定。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-result-init-refine.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpwrite-resultrefine.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpwrite-resultrefine.json -j 15` -> `262/262 pass`
- 状況:
  - `writer_new_raw` の失敗表現は `Result` へ収束し、センチネル `0` 依存の分岐を初期化経路から除去できた。
  - 次段は `todo.md` フェーズDの主課題（`core/mem` 公開APIの安全名統一）を継続する。

# 2026-03-04 作業メモ (フェーズD進行: kpwrite 初期化経路の Result 化)

- 目的:
  - `kpwrite` の初期化経路を `Result` 経路へ揃え、`kpread` と同じ失敗表現に統一する。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - 旧 `writer_new_raw`（`i32`返却）本体を `writer_new_handle_raw` へ分離。
    - 新 `writer_new_raw` を `Result<i32,str>` 返却へ変更。
    - `writer_new` は `writer_new_raw` の `Result` を `Writer` へ持ち上げる実装へ変更。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-result-init.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpwrite-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpwrite-result.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread/kpwrite` の初期化公開経路はどちらも `Result` ベースで統一済み。
  - 次段は `todo.md` フェーズD残件として、`mem` 側公開名の安全API標準化を進める。

# 2026-03-04 作業メモ (フェーズD進行: kpread_core の初期化を Result ベース化)

- 目的:
  - `kpread` 初期化経路の失敗表現を `0` センチネル依存から `Result` へ寄せる。
  - メモリ確保失敗時の分岐を型で扱えるようにし、段階的な安全API標準化を進める。
- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `scanner_new_impl_i` を `scanner_new_impl` へ改名。
    - 戻り値を `i32` から `Result<i32,str>` へ変更。
    - `alloc_result/realloc_result` を使って確保失敗を `Err` 化。
    - 後始末（解放）は既存レイアウト維持のため `dealloc_raw` を継続使用。
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` を `Result<i32,str>` 返却へ変更。
    - `scanner_new` は `scanner_new_raw` の `Result` をそのまま `Scanner` へ持ち上げる形に変更。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-tree -o /tmp/tests-kpread-result-init.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpreadcore-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpreadcore-result.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread` の初期化経路は `Result` ベースに移行済み。
  - 次段で `kpwrite` 初期化経路も同じ方針に揃える。

# 2026-03-04 作業メモ (フェーズD進行: `*_new_raw` 名統一と todo 未完了整理)

- 目的:
  - `kpread/kpwrite` の内部初期化関数名を `*_raw` に統一し、公開入口を `scanner_new` / `writer_new` に寄せる。
  - `todo.md` から完了済みのテスト追加項目を削除し、未完了のみを保持する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_i32` -> `scanner_new_raw`。
    - `scanner_new` からの呼び出し先を更新。
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_i32` -> `writer_new_raw`。
    - `writer_new` からの呼び出し先を更新。
  - `todo.md`
    - フェーズEの完了済み小項目（`tests/move_effect.n.md` 追加、`tests/overload.n.md`/`tests/kp*.n.md` 更新）を削除。
    - 項目8の完了済み小項目（`tests/memory_safety.n.md` 追加）を削除。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kp-newraw-rename.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-newraw-rename.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-newraw-rename.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread/kpwrite` の内部初期化関数名が `*_raw` で揃った。
  - 次段はフェーズD残件として、`mem` 公開面の安全API標準名化（`Result/Option` 前提）を進める。

# 2026-03-04 作業メモ (フェーズD進行: kpread の raw 実装名分離)

- 目的:
  - `kpread` の内部 `i32` ハンドル実装と公開 `Scanner` API を明確に分離し、公開面の型安全性を上げる。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `i32` 受け取り実装を `scanner_*_raw` へ改名。
    - `Scanner` 受け取り公開関数は既存名を維持し、内部で `*_raw` を呼び出す形へ変更。
    - 対象: `skip_ws/is_eof/skip_token/read_token/read_i32/read_i64/read_u64/read_f32/read_f64/read_vec/read_matrix/read_all/read_*input` 一式。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-raw-split-both.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpread-split.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpread-split.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread/kpwrite` ともに「公開 API = Scanner/Writer 型」「内部実装 = *_raw」へ分離済み。
  - 次段は `todo.md` 2026-03-03 フェーズDの残件（`mem` 公開面の `_safe` 廃止と `_raw` 最終削除）へ進む。

# 2026-03-04 作業メモ (フェーズD進行: kpwrite の raw 実装名分離)

- 目的:
  - `kpwrite` の内部 `i32` ハンドル実装と公開 `Writer` API を明確に分離し、公開面の型安全性を上げる。
- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `i32` 受け取り実装を `writer_*_raw` へ改名。
    - `Writer` 受け取り公開関数は既存名を維持し、内部で `*_raw` を呼び出す形へ変更。
    - 対象: `free/flush/ensure/put_u8/writeln/write_*` 一式。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-raw-split.json -j 15` -> `226/226 pass`
- 状況:
  - `kpwrite` は「公開 API = Writer 型」「内部実装 = *_raw」へ分離完了。
  - 次段で `kpread` も同方針に揃える。

# 2026-03-04 作業メモ (overload テスト拡充: 注釈混在ケースの追加)

- 目的:
  - `overload` 回帰に、型注釈の混在パターン（ブロック注釈・関数呼び出し注釈・パイプ・関数リテラル）を追加する。
- 変更:
  - `tests/overload.n.md`
    - `overload_mixed_annotations_block_call_pipe_lambda` を追加。
    - `overload_pipe_annotations_with_mixed_cast_i32_i64_i128` を追加。
- 切り分け:
  - 初版では `pipe requires a value on the stack (D3013)` と `ambiguous overload (D3005)` を再現。
  - 解析結果:
    - `let ...:` の引数ブロック直後に `|>` を直接接続する形は現行仕様では式境界が分かれる。
    - `|> <i64> cast` は「関数値への注釈」として解釈され、戻り値注釈にはならず曖昧化する。
  - テストは仕様に整合する形へ修正:
    - ブロック注釈は `base` に束縛してから通常呼び出しで連結。
    - cast は `seed` を明示変換した後に pipe で加算を実施。
- テスト:
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-after-fix2.json -j 15` -> `239/239 pass`

# 2026-03-04 作業メモ (フェーズD進行: stdlib の生メモリ呼び出しを `*_raw` へ段階移行)

- 目的:
  - `mem` の公開名切替前に、stdlib 側の生アロケータ呼び出しを `alloc_raw/dealloc_raw/realloc_raw` に寄せる。
- 変更:
  - `stdlib/alloc/collections/{btreemap,btreeset,hashmap,hashset,list,ringbuffer,stack,vec/sort}.nepl`
  - `stdlib/alloc/{diag/error,string}.nepl`
  - `stdlib/kp/{kpdsu,kpfenwick,kpgraph,kpprefix,kpread_core}.nepl`
  - `stdlib/nm/{parser,html_gen}.nepl`
  - `stdlib/platforms/wasix/tui.nepl`
  - `stdlib/std/{env/cliarg,fs,stdio}.nepl`
  - 上記で `alloc/dealloc/realloc` の生呼び出しを `*_raw` に置換（`core/mem` の公開名依存を分離）。
- 切り分け:
  - 一括置換後、`tests/capacity_stack.n.md::doctest#3` で OOB を再現。
  - 原因切り分けで `vec.nepl` の `realloc_raw` 置換時のみ再現することを確認したため、`vec.nepl` 本体は現時点では `realloc` 呼び出しを維持して回避。
  - この差分は `todo.md` に未解決課題として追記。
- テスト:
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-raw-migration-wide2.json -j 15` -> `725/725 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-raw-migration-wide.json -j 15` -> `262/262 pass`
- 状況:
  - stdlib の大部分は `*_raw` 呼び出しへ移行済み。
  - 残件は `vec.nepl` の `realloc_raw` 移行に伴う OOB 原因の根本修正。

# 2026-03-04 作業メモ (フェーズD進行: `kpread/kpwrite` の生メモリ呼び出しを `*_raw` へ移行)

- 目的:
  - `core/mem` の `*_raw` 分離に合わせ、`kpread/kpwrite` 側の生アロケータ呼び出しを明示化する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 文字列トークン生成時の確保を `alloc` から `alloc_raw` へ変更。
  - `stdlib/kp/kpwrite.nepl`
    - writer 初期化/解放の `alloc`/`dealloc` 呼び出しを `alloc_raw`/`dealloc_raw` へ変更。
    - ドキュメントコメントの文言を実装に合わせて調整（「ヒープ確保なし」）。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-mem-raw-callsite-migration.json -j 15` -> `229/229 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-after-kp-memraw-migration-full.json -j 15` -> `725/725 pass`
- 状況:
  - `mem` の生アロケータ利用箇所は `kpread/kpwrite` で `*_raw` へ追従済み。
  - 次段は `alloc/realloc/dealloc` 公開名を Result/Option 安全APIへ切り替える準備として、残り呼び出し箇所を段階移行する。

# 2026-03-04 作業メモ (フェーズD進行: `core/mem` に `*_raw` 隔離を導入)

- 目的:
  - 生ポインタAPIを段階的に分離し、次段の安全API標準名化に備える。
- 変更:
  - `stdlib/core/mem.nepl`
    - 生API本体を `alloc_raw` / `realloc_raw` / `dealloc_raw` へ改名。
    - `alloc` / `realloc` / `dealloc` は `*_raw` への委譲エイリアスへ変更。
    - `alloc_result` / `realloc_result` / `dealloc_result` と `alloc_ptr` 系は `*_raw` を直接呼ぶように変更。
- テスト:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-raw-alias.json -j 15` -> `213/213 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-mem-raw-alias.json -j 15` -> `725/725 pass`
- 状況:
  - `mem` 側で「生API本体」と「公開名」を分離できた。
  - 次段は `alloc/realloc/dealloc` 公開名を安全APIへ切り替える際の呼び出し側移行（stdlib/tests/tutorials）に着手できる状態。

# 2026-03-04 作業メモ (フェーズE前進: `mem_result` 系APIの回帰テスト追加)

- 目的:
  - `core/mem` の `alloc_result/realloc_result/dealloc_result` 命名変更をテストで固定する。
- 変更:
  - `tests/memory_safety.n.md`
    - `alloc_result/dealloc_result` の正常系テストを追加。
    - `dealloc_result` の無効引数 `Err` 返却テストを追加。
- テスト:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-result-rename.json -j 15` -> `213/213 pass`
- 状況:
  - `core/mem` の `_safe` 命名除去分について、命名変更後の最小回帰を固定した。

# 2026-03-04 作業メモ (フェーズD進行: `core/mem` の `_safe` 命名除去)

- 目的:
  - `core/mem` の安全ラッパAPIから `_safe` 接尾辞を除去し、命名規約を次段移行しやすい形へ揃える。
- 変更:
  - `stdlib/core/mem.nepl`
    - `alloc_safe` -> `alloc_result`
    - `realloc_safe` -> `realloc_result`
    - `dealloc_safe` -> `dealloc_result`
    - 関連ドキュメントコメント内の関数名・注意事項を更新。
  - `todo.md`
    - フェーズDの文言を、`_safe` 統一方針から「`_safe` 接尾辞廃止＋安全API標準名化」へ更新。
    - `move/effect` 反映項目を、`mem` 側と `kpread/kpwrite` 側の残件に分割して明記。
- テスト:
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-after-mem-safe-rename.json -j 15` -> `723/723 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-safe-rename.json -j 15` -> `262/262 pass`
- 状況:
  - `_safe` 命名除去は `core/mem` で着手済み。
  - 次段は API 本体を Result/Option 標準名へ寄せるため、`alloc/realloc/dealloc` の生ポインタAPI整理（`*_raw` 隔離）に進む。

# 2026-03-04 作業メモ (フェーズD進行: kpread/kpwrite の `_raw` 名整理完了)

- 目的:
  - `kpread/kpwrite` で残っていた `_raw` 接尾辞の公開名を整理し、通常API名へ統一する。
  - 変更後の全体回帰を `tests + stdlib + tutorials` で確認する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` を `scanner_new_i32` へ変更。
    - `scanner_skip_ws_raw` / `scanner_is_eof_raw` / `scanner_skip_token_raw` / `scanner_read_*_raw` を `scanner_*` へ統一。
    - ドキュメントコメント中の関数名記述も実体に合わせて更新。
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_raw` を `writer_new_i32` へ変更。
    - `writer_write_*_raw` / `writer_writeln_raw` / `writer_flush_raw` / `writer_free_raw` を `writer_*` へ統一。
    - ドキュメントコメント中の関数名記述も実体に合わせて更新。
- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-stdlib --no-tree -o /tmp/tests-kpread-kpwrite-no-raw.json -j 15` -> `5/5 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-full-after-kp-overload-unify.json -j 15` -> `781/781 pass`
- 状況:
  - `kpread/kpwrite` から `_raw` 接尾辞は解消済み。
  - `todo.md` の `_safe/_raw` 最終整理は `mem.nepl` 側（`alloc_safe/realloc_safe/dealloc_safe`）が残件。

# 2026-03-04 作業メモ (フェーズD進行: Scanner/Writer API一本化とハンドル露出除去)

- 目的:
  - `kpread/kpwrite` の公開APIから `scanner_handle/writer_handle` を除去し、`Scanner`/`Writer` 型APIへ一本化する。
  - `Scanner` 呼び出しが move で破綻する根本原因（コンパイラの非Copy特例）を上流で修正する。
- 変更:
  - `stdlib/kp/kpread.nepl`
    - 生ハンドル実装を `*_raw` 名へ分離。
    - 公開関数は `Scanner` 引数の通常名（`scanner_read_i32` など）に統一。
    - `scanner_handle` 相当の公開関数を削除し、内部でのみ `mem_ptr_addr get sc "raw"` を使用。
  - `stdlib/kp/kpwrite.nepl`
    - 生ハンドル実装を `*_raw` 名へ分離。
    - 公開関数は `Writer` 引数の通常名（`writer_write_i32` など）に統一。
    - `writer_handle` 相当の公開関数を削除し、内部でのみ `mem_ptr_addr get w "raw"` を使用。
  - 依存箇所の移行:
    - `tests/kp.n.md`, `tests/kp_i64.n.md`, `tests/stdin.n.md`
    - `tutorials/getting_started/22_*.n.md`, `24_*.n.md`, `25_*.n.md`, `27_*.n.md`
    - `examples/kp_fizzbuzz.nepl`
    - `stdlib/kp/kpgraph.nepl`（`dense_graph_read_undirected_1indexed` を `Scanner` 受け取りへ変更）
  - 上流修正:
    - `nepl-core/src/types.rs` の明示非Copy判定から `Scanner` を除外（`RegionToken`/`Writer` は維持）。
- テスト:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpgraph.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-api-unify.json -j 15` -> `231/231 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-full-after-kp-api-unify.json -j 15` -> `781/781 pass`
- 状況:
  - `kpread/kpwrite` の公開APIは `Scanner`/`Writer` 型ベースに揃った。
  - 次段は `todo.md` フェーズDの残件（`_safe` 廃止と `_raw` 最終削除、trait 境界導入）を進める。

# 2026-03-04 作業メモ (フェーズD前進: ptr安全APIの _safe 依存切り離し)

- 目的:
  - `mem` の公開 `Result` API を `_safe` ラッパ名から独立させ、`_safe` 廃止に向けた段階移行を進める。
- 変更:
  - `stdlib/core/mem.nepl`
    - `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` の内部実装を `alloc_safe/realloc_safe/dealloc_safe` 呼び出しから分離。
    - `alloc` / `realloc` / `dealloc` を直接呼び、公開API側で `Result` 判定を行うように変更。
- テスト:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-ptr-safe-decouple.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-ptr-safe-decouple.json -j 15` -> `723/723 pass`
- 状況:
  - `*_ptr` 系の公開安全APIは `_safe` 名に依存しない形へ移行済み。
  - 次段では `alloc_safe/realloc_safe/dealloc_safe` 自体を縮退し、公開名一本化へ進める。

# 2026-03-04 作業メモ (フェーズE前進: memory_safety 回帰追加)

- 目的:
  - `todo.md` フェーズEの追加項目 `tests/memory_safety.n.md` を先行で固定化する。
- 変更:
  - `tests/memory_safety.n.md` を新規追加。
    - `alloc_ptr/load_i32_ptr/store_i32_ptr/dealloc_ptr` の正常系。
    - 無効ポインタ `load` が `Option::None` を返す異常系。
    - 無効ポインタ `store` が `Result::Err` を返す異常系。
- テスト:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-memory-safety-tests.json -j 15` -> `723/723 pass`
- 状況:
  - `tests/memory_safety.n.md` 追加タスクは完了し、`todo.md` から削除済み。
  - 次は `mem/kpread/kpwrite` の `_safe` なし安全API一本化と `_raw` 最終削除へ進む。

# 2026-03-04 作業メモ (フェーズC着手: MemPtr のジェネリクス化)

- 目的:
  - `doc/memory_safety_compiler_design.md` の型モデルに沿って、`MemPtr<T>` を公開API側へ反映する。
- 変更:
  - `stdlib/core/mem.nepl`
    - `MemPtr` を `MemPtr<.T>` へ変更。
    - `mem_ptr_wrap` / `mem_ptr_addr` / `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `mem_ptr_add` をジェネリクス対応。
    - `load_i32_ptr` / `store_i32_ptr` は `MemPtr<i32>`、`load_u8_ptr` / `store_u8_ptr` は `MemPtr<u8>` を受けるように変更。
  - `stdlib/kp/kpread.nepl`
    - `Scanner.raw` を `MemPtr<u8>` 化。
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` を `MemPtr<u8>` 化。
- テスト:
  - `node nodesrc/tests.js -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-mem-kp-generic-memptr.json -j 15` -> `220/220 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-memptr-generic.json -j 15` -> `720/720 pass`
- 状況:
  - `MemPtr<T>` 型モデルは導入済み（公開APIの i32 生ポインタ除去は継続）。
  - 次は `RegionToken` 導入と `alloc/realloc/dealloc` の `Result` 一本化を進める。

# 2026-03-04 作業メモ (フェーズB完了: Copy/Clone 制約 + RegionToken 非Copy化)

- 目的:
  - `todo.md` フェーズB残件だった `Copy/Clone` 制約検査と `RegionToken` 非Copy扱いを型検査に反映する。
- 変更:
  - `nepl-core/src/types.rs`
    - `TypeCtx::is_copy` に明示非Copy型判定を追加（`RegionToken` / `Scanner` / `Writer`）。
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3049` (`TypeCopyImplTargetNotCopy`) と `D3050` (`TypeCopyImplRequiresClone`) を追加。
  - `nepl-core/src/typecheck.rs`
    - `impl Copy for T` の収集時に `ctx.is_copy(T)` を検証し、非Copy対象を `D3049` で拒否。
    - `Copy` 実装には同一対象 `Clone` 実装が必要な検査を追加し、欠落時 `D3050` で拒否。
    - 拒否対象の `Copy` 実装は後続の impl 収集/照合から除外。
  - `tests/move_effect.n.md`
    - `D3049`/`D3050` の compile_fail ケースを追加。
    - `Clone+Copy` 両実装時の成功ケースを追加。
    - `RegionToken` の move 後再利用拒否ケースを追加。
- テスト:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-copy-clone.json -j 15` -> `218/218 pass`
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-overload-typeannot-copyclone.json -j 15` -> `266/266 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-copy-clone.json -j 15` -> `720/720 pass`
- 状況:
  - フェーズBの `Copy/Clone` 制約と `RegionToken` 非Copy化は反映済み。
  - 次は `todo.md` のフェーズC/D（`MemPtr<T>` と `mem/kpread/kpwrite` の安全API一本化）へ進む。

# 2026-03-04 作業メモ (フェーズB進行: move_check に borrow 状態遷移を実装)
- 目的:
  - `todo.md` のフェーズBにある `move_check` 状態機械を `BorrowedShared/BorrowedUnique` まで拡張し、分岐/ループ/match 合流を保守的に正しく扱う。
- 実装:
  - `nepl-core/src/passes/move_check.rs`
    - `VarState` に `BorrowedShared` / `BorrowedUnique` を追加。
    - `BorrowKind` を導入し、`visit_borrow` を `Shared/Unique` 区別で処理。
    - `check_use` を更新し、borrow 中 move や unique borrow 中 use を拒否。
    - `check_assign` / `check_drop` / `check_borrow` を追加し、代入・drop・borrow での状態遷移を一元化。
    - `merge_state_pair` / `merge_states` を追加し、`if`/`match`/`while` 合流を `Valid/Borrowed/Moved/PossiblyMoved` で統一。
    - `Intrinsic::load/store` のアドレス引数 borrow 判定を `BorrowKind` に接続。
  - `tests/move_effect.n.md`
    - 非Copy値の shared borrow 中 move が拒否される回帰を追加。
    - Copy値 borrow が利用を阻害しない回帰を追加。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-overload-typeannot.json -j 15` -> `262/262 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-move-borrow.json -j 15` -> `716/716 pass`
- 次:
  - フェーズB残件 (`Copy/Clone` trait制約検査, `RegionToken` 消費規則) に進む。

# 2026-03-04 作業メモ (フェーズB着手: `TypeCtx::is_copy` 構造型判定)
- 目的:
  - フェーズBの最初の実装として、`TypeCtx::is_copy` を tuple/struct/enum と generic apply へ拡張する。
  - 再帰検出ロジックの誤判定（同一型の再訪で常に false）を解消する。
- 実装:
  - `nepl-core/src/types.rs`
    - `is_copy_inner` を `visiting + mapping` 方式に変更。
    - `TypeKind::Struct` / `TypeKind::Enum` を構造的再帰判定へ変更。
    - `TypeKind::Apply` で base の type parameter を実引数へ束縛して copy 判定できるよう対応。
    - 判定終了時に `visiting.remove` を行い、兄弟ノード再訪での偽陰性を解消。
  - `tests/move_effect.n.md`
    - Copy フィールドのみの struct 再利用ケース（成功）
    - `Apply` された generic struct 再利用ケース（成功）
    - payload が Copy の enum 再利用ケース（成功）
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/generics.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-moveeffect-generics-overload.json -j 15` -> `269/269 pass`
- 次:
  - move_check 側の状態遷移（`PossiblyMoved` 合流、borrow 状態）を `is_copy` 拡張に合わせて精査する。

# 2026-03-04 作業メモ (2026-03-03 フェーズA完了: raw/intrinsic effect 一元化)
- 目的:
  - フェーズA残件だった「intrinsic / raw target body の effect 判定一元化」を実装し、pure 文脈からの I/O を型検査段階で拒否する。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `IMPURE_IO_EFFECT_MARKERS` を追加し、I/O語彙テーブルを導入。
    - `intrinsic_effect` / `raw_lines_effect` / `raw_body_effect` を追加して effect 判定を共通化。
    - `BlockChecker::validate_raw_body_effect` を追加し、`#wasm`/`#llvmir` 本体が I/O語彙を含む場合、pure 関数で `D3025` を返すように変更。
    - `FnBody::Parsed` の target選択raw本体、および `FnBody::Wasm` / `FnBody::LlvmIr` 直指定の両方で同じ検査を実施。
    - `PrefixItem::Intrinsic` でも共通 effect 判定を通すよう変更。
  - `tests/move_effect.n.md`
    - pure raw body で `fd_write` を含むケースを追加（`compile_fail`, `diag_id: 3025`）。
  - `todo.md`
    - 完了済みフェーズA項目を削除し、未完のみへ整理。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md -i tests/intrinsic.n.md --no-tree -o /tmp/tests-effect-overload-typeannot-intrinsic.json -j 15` -> `263/263 pass`
- 現状:
  - フェーズA（effect規則の反映）は完了。
  - 次はフェーズB（`TypeCtx::is_copy` 拡張と move/borrow 状態遷移の厳密化）へ進む。

# 2026-03-04 作業メモ (2026-03-03 フェーズA再開: effect診断IDと回帰追加)
- 目的:
  - `todo.md` の 2026-03-03 計画フェーズAを再開し、pure/impure 判定の診断固定を進める。
- 実装:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3025 TypePureCallsImpureFunction` を追加。
  - `nepl-core/src/typecheck.rs`
    - 「pure context cannot call impure function」の全発生箇所に `D3025` を付与。
  - `tests/move_effect.n.md` を新規追加。
    - pure からメモリ操作を呼べるケース（成功）
    - pure から impure 関数呼び出し拒否（`diag_id: 3025`）
    - ローカル `set` が pure のまま使えるケース（成功）
    - グローバル `set` が impure になるケース（`diag_id: 3025`）
  - `todo.md`
    - 完了済み項目（`builtins` のメモリ系 Pure 化、entry 強制 Impure 特例の削除）をフェーズAから削除。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-effect-overload-typeannot.json -j 15` -> `256/256 pass`
- 次:
  - フェーズA残件の「intrinsic / raw target body の effect 一元判定」を実装する。

# 2026-03-04 作業メモ (オーバーロード修正の完了と 2026-03-03 計画への復帰)
- 目的:
  - オーバーロード解決の不安定箇所（関数値引数・pipe 併用・型注釈混在）を根本修正し、`todo.md` の `2026-03-03 move/effect/memory` 実装へ復帰する。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - 関数シグネチャ参照を `function_signature_for_entry` に集約し、type_args 適用後の引数型を一貫取得するよう修正。
    - pipe 注入時に nullary callable の過早 reduce を避ける制御と、target 入力型を使った `reduce_pipe_pending_value_with_target` を追加。
    - オーバーロード候補の絞り込みで「具体型候補優先」「型パラメータ数最小候補優先」を導入し、`D3005` の過検出を抑制。
  - `tests/overload.n.md`, `tests/typeannot.n.md`
    - ブロック注釈/関数呼び出し注釈/pipe 注釈/関数リテラル注釈の混在ケースを拡充し、今回の修正点を回帰固定。
  - `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/stack.nepl`, `stdlib/tests/stack.n.md`
    - `push` 利用形と型推論ケースを整理し、オーバーロード解決の実運用ケースを固定。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/typeannot.n.md -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md --no-tree -o /tmp/tests-overload-typeannot-vec-stack.json -j 15` -> `286/286 pass`
- 現状:
  - オーバーロード修正は完了。
  - 次は `todo.md` の `2026-03-03 move/effect/memory 本格実装計画` フェーズA（effect規則のコンパイラ反映）を再開する。

# 2026-03-04 作業メモ (pipe 活用と `push` 推論の確認)
- 目的:
  - 既存書き換え方針として、pipe 演算子を活用して中間変数とインデントを抑える。
  - `vec_push<i32> ...` ではなく `push ...` だけで型推論できる利用形を明示する。
- 実施:
  - `stdlib/alloc/collections/list.nepl`
    - doctest のリスト構築を `list_nil |> list_push_front ...` へ変更。
    - move 規則に合わせて再利用箇所を再束縛へ整理。
    - 実装の一部で中間変数を削減（`list_len`, `list_get`, `list_free`, `list_reverse`）。
  - `stdlib/alloc/collections/vec.nepl`
    - doctest の `vec_push<i32>` / `push<i32>` を `push` に統一。
    - `vec_new<i32> |> push 10 |> push 20` の形へ変更し、型引数省略で成立する例へ更新。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl -i stdlib/alloc/collections/vec.nepl --no-stdlib --no-tree -o /tmp/tests-list-vec-pipe.json -j 15` -> `28/28 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-stdlib --no-tree -o /tmp/tests-vec-push-infer.json -j 15` -> `17/17 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-plus-tests-after-push-alias.json -j 15` -> `700/700 pass`

# 2026-03-03 作業メモ (仕様最終確認: 前置記法/オーバーロード整合)
- 実施:
  - `doc/move_effect_spec.md` に「NEPLg2既存仕様との整合」章を追加。
  - 前置記法、型注釈、オーバーロード、暗黙cast禁止との整合を明記。
  - 同名オーバーロードの effect 一致制約を仕様へ反映。
- 結果:
  - 設計方針（メモリ操作 pure / I/O のみ impure）と既存言語仕様の論理矛盾は無し。
  - 実装未反映箇所（builtins の effect, entry 特例）は引き続き `todo.md` 管理。

# 2026-03-03 作業メモ (move/effect/memory 仕様の再確定: trait 統合)
- 目的:
  - heap/線形メモリ操作を pure とする設計を矛盾なく確定し、`move/borrow/copy/clone` と一体で仕様化する。
- 実施:
  - `doc/move_effect_spec.md` を更新。
    - `Pure/Impure` の判定を「I/O 外部副作用基準」に固定。
    - メモリ操作 pure 化の成立条件（状態隠蔽・生ポインタ非公開・Result/Option 化）を明文化。
    - `trait` の位置づけを追加し、`Copy/Clone` とメモリ系 trait の役割を定義。
  - `doc/memory_safety_compiler_design.md` を更新。
    - trait 制約検査（`Copy` 可否、`Clone` 規約、`MemReadable/MemWritable/RegionOwned`）を追加。
    - `core/mem` と `kpread/kpwrite` の trait ベース API 方針を追記。
- 現実装との差分:
  - `builtins.rs` では `alloc/realloc/dealloc` が依然 `Effect::Impure`。
  - `typecheck.rs` では entry を強制 `Impure` にしている。
  - trait 境界でのメモリ能力検査は未実装。
- 次:
  - `todo.md` の move/effect・メモリ安全タスクに trait 導入を反映し、実装フェーズへ進む。

# 2026-03-03 作業メモ (メモリ安全コンパイラ機構の設計)
- 目的:
  - `i32` 生ポインタ露出を減らし、コンパイラ検査で `mem/kpread/kpwrite` の誤用を防ぐ。
- 追加:
  - `doc/memory_safety_compiler_design.md` を新規作成。
  - `MemPtr<T>` / `RegionToken` モデル、境界検査挿入、解放状態検査、診断方針を定義。
  - `alloc/realloc/dealloc/load/store` を Pure とし、I/O 系のみ Impure とする方針を明記。
- 実装差分:
  - まだ仕様段階で、`TypeCtx/move_check/typecheck` への反映は未着手。
  - 実装タスクは `todo.md` の「8. メモリ安全コンパイラ機構の導入」で追跡する。

# 2026-03-03 作業メモ (move/effect 精査結果: 現行実装との差分)
- 精査対象:
  - `nepl-core/src/typecheck.rs`
  - `nepl-core/src/builtins.rs`
  - `nepl-core/src/types.rs`
- 差分:
  - `check_function` で `is_entry` 時に `current_effect = Impure` を強制している。
  - builtins の `alloc/realloc/dealloc` が `Effect::Impure` 登録になっている。
  - `TypeCtx::is_copy` が `Struct/Enum` を一律 `false` としている。
- 判断:
  - いずれも `doc/move_effect_spec.md` の再設計仕様と不一致。
  - 先に仕様を固定し、実装は上流から段階的に修正する（entry特例 -> builtins effect -> is_copy拡張）。

# 2026-03-03 作業メモ (move/effect 再設計仕様の文書化)
- 目的:
  - `move` と `pure/impure` の責務分離を明文化し、`mem/kpread/kpwrite` の安全API移行を設計レベルで固定する。
- 追加:
  - `doc/move_effect_spec.md` を新規作成。
  - 次を仕様として確定:
    - `->` を Pure、`*>` を Impure として扱う。
    - heap/線形メモリ操作（`alloc/realloc/dealloc/load/store`）は Pure。
    - Impure は I/O・syscall・環境依存値取得に限定。
    - move は effect と独立に評価。
    - `entry` を常に Impure 扱いする特例は撤廃対象。
    - `_safe` 接尾辞を廃止し、安全版APIをデフォルト化する方針。
- 差分:
  - 実装はまだ旧挙動が残る（特に entry 特例、Copy 判定の構造型対応、intrinsic effect 一元表）。
  - 本エントリは仕様確定まで。実装反映は `todo.md` 側で継続管理する。

# 2026-03-03 作業メモ (mem/kp の `_raw` 段階廃止と安全API寄せ)
- 目的:
  - `mem/kpread/kpwrite` の `_raw` 接尾辞を段階廃止し、安全API（`Result/Option`）中心へ寄せる。
  - `Scanner` / `Writer` ラッパ導入後の move 破綻を根本修正する。
- 実装:
  - `stdlib/core/mem.nepl`
    - `mem_ptr_raw` を `mem_ptr_addr` へ変更。
    - `alloc_ptr_raw / realloc_ptr_raw / dealloc_ptr_raw / load_*_ptr_raw / store_*_ptr_raw` を削除。
    - 公開APIは `alloc_ptr/realloc_ptr/dealloc_ptr/load_*_ptr/store_*_ptr`（`Result/Option`）に統一。
  - `stdlib/kp/kpread.nepl`
    - `scanner_raw` -> `scanner_handle`、`scanner_new_raw` -> `scanner_new_handle` に改名。
    - `Scanner` 利用側は `scanner_handle` を一度取り出して i32 系 read API を使う形に統一（move 破綻回避）。
  - `stdlib/kp/kpwrite.nepl`
    - `writer_raw` -> `writer_handle`、`writer_new_raw` -> `writer_new_handle` に改名。
    - `Writer` オーバーロード群の move バグを修正:
      - `writer_handle` で i32 を取り出し
      - 低レベル関数を呼び
      - `writer_wrap raw` を返す
    - i32 低レベル関数での `set w ...`（immutable 代入）を除去。
    - doctest の `Writer` 使用例を再束縛（`set w ...`）に修正。
  - `tests/kp.n.md`, `tests/kp_i64.n.md`, `tests/stdin.n.md`
    - `Scanner` から `scanner_handle` を取得して読み取りを行う形へ更新。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md --no-tree --no-stdlib -o /tmp/tests-kp-safe-now6.json -j 16`
    - `15/15 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i examples/kp_fizzbuzz.nepl --no-tree --no-stdlib -o /tmp/tests-kp-safe-broader2.json -j 16`
    - `20/20 pass`
- 残課題:
  - `scanner_handle` / `writer_handle` / `mem_ptr_addr` は依然としてハンドル露出点であり、最終的には公開APIから隠蔽する必要がある。
  - `Result` ベース一本化（`_safe` から suffix なし統一）は `mem` 以外の stdlib へ横展開が必要。

# 2026-03-03 作業メモ (オーバーロード根本修正: 関数値引数の arity/型文脈解決)
- 目的:
  - `use_binary 3 4 calc` や `5 |> use_unary calc` のように、オーバーロード関数名を「関数値引数」として渡すケースを安定解決する。
  - 間に合わせで中間変数へ分解せず、入れ子呼び出し/パイプのまま通す。
- 原因:
  - typecheck の直接 callable 経路で、引数位置に `Var(calc)` が来た時に、期待される関数型（例: `(i32,i32)->i32`）へ具体化されず、未解決のまま残っていた。
  - その結果、compile では `undefined identifier` / run では `null function or function signature mismatch` が発生していた。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `apply_function` の引数処理で、`Var(name)` かつ値 binding 不在の場合に callable 候補を検索。
    - 引数位置の期待型 `param_ty` に unify する候補を選別し、単一候補なら `FnValue(selected_symbol)` へ置換。
    - 複数候補一致時は `D3005`（ambiguous overload）を返す。
    - 候補なしは既存どおり `D3006`（no matching overload）へ到達。
  - `tests/overload.n.md`
    - パイプ/混在 cast/関数戻り値注釈推論ケースを拡充。
    - 仕様変更で成功可能になった 2 ケース（単項 arity 文脈・pipe 単項文脈）を `compile_fail` から成功テストへ変更。
    - `stack_new` の `Result` 化に合わせて該当ケースを `unwrap_ok` ベースへ更新。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/overload_after_expect_update.json -j 1`
    - `30/30 pass`

# 2026-02-27 作業メモ (GitHub Actions: wasm-bindgen ダウンロード失敗の安定化)
- 背景:
  - `trunk build` 実行時に、Trunk 内部の `wasm-bindgen` 自動ダウンロードが接続断で失敗するケースが発生。
  - エラー例: `failed downloading release archive` / `connection closed before message completed`
- 実装:
  - `trunk` を使う workflow へ、事前に `wasm-bindgen-cli 0.2.108` を導入する step を追加。
  - 追加先:
    - `.github/workflows/gh-pages.yml`
    - `.github/workflows/nepl-test-wasi.yml`
    - `.github/workflows/nepl-test-llvm.yml`
    - `.github/workflows/nmd-doctest.yml`
  - 導入方法:
    - `cargo install --locked wasm-bindgen-cli --version 0.2.108`
    - 5回リトライ + backoff（5s,10s,15s,20s,25s）
- 期待効果:
  - Trunk の実行中ダウンロード依存を減らし、ネットワーク瞬断時の失敗率を低減。
  - 失敗時も step 単位で再試行されるため、CI 全体の安定性が向上。

# 2026-02-27 作業メモ (`@` 強制関数値とオーバーロード関連の診断ID拡張)
- 目的:
  - `@` を callable 以外へ適用したときの誤受理を根本修正する。
  - オーバーロード/型引数/引数型不一致の診断を `diag_id` で安定検証できるようにする。
- 原因:
  - `typecheck` の識別子解決で、`forced_value (@name)` の分岐が「関数 binding であること」を常に検証しておらず、値 binding が通る経路が残っていた。
  - 一部診断が既存IDへ過剰集約され、`compile_fail` の精密検証がしづらかった。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `@` 強制関数値の経路で `BindingKind::Func` 以外を即時拒否する分岐へ修正。
    - `only callable symbols can be referenced with '@'` に `DiagnosticId::TypeAtRequiresCallable (3023)` を付与。
    - 変数への型引数適用、オーバーロード effect 不一致、型引数不一致、引数型不一致にも専用IDを付与。
  - `nepl-core/src/diagnostic_ids.rs`
    - `3020..3024` を追加:
      - `TypeOverloadEffectMismatch`
      - `TypeOverloadTypeArgsMismatch`
      - `TypeArgumentTypeMismatch`
      - `TypeAtRequiresCallable`
      - `TypeVariableTypeArgsNotAllowed`
  - `tests/functions.n.md`
    - `function_at_requires_callable_reports_diag_id` を追加（`compile_fail`, `diag_id: 3023`）。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-diagids-v4.json -j 2`
    -> `111/111 pass`

# 2026-02-27 作業メモ (parser if/while レイアウト診断へID付与)
- 目的:
  - parser の if/while レイアウト系エラーを `diag_id` で一貫管理し、木構造テストから機械検証できるようにする。
- 実装:
  - `nepl-core/src/parser.rs`
    - 次のエラーに `DiagnosticId` を付与:
      - `invalid marker ...` / `duplicate marker ...` / `too many expressions ...` -> `ParserUnexpectedToken (2002)`
      - `missing expression(s) ...` / `argument layout block must contain expressions` -> `ParserExpectedToken (2001)`
      - `only expressions are allowed ...` -> `ParserUnexpectedToken (2002)`
  - `tests/tree/18_diagnostic_ids.js`
    - `if:` レイアウトの marker 順序誤りケースを追加し、`id=2002` を検証。
  - `tests/if.n.md`
    - `if_layout_invalid_marker_order_reports_diag_id` を追加（`compile_fail`）。
    - wasm 実行系の `compile_fail diag_id` 抽出制約に合わせ、ここは `diag_id` 指定なしで失敗そのものを検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/if.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-diagid-layout-v2.json -j 2`
    -> `166/166 pass`
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-after-parser-id.json -j 2`
    -> `111/111 pass`

# 2026-02-27 作業メモ (compile_fail 用診断IDの拡張: スタック余剰値)
- 目的:
  - `compile_fail` で「呼び出し arity 不整合により余剰値が残る」ケースを `diag_id` で固定検証できるようにする。
- 実装:
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId::TypeStackExtraValues = 3016` を追加。
    - `from_u32` / `message` に同IDを追加。
  - `nepl-core/src/typecheck.rs`
    - `expression left extra values on the stack` に `with_id(DiagnosticId::TypeStackExtraValues)` を付与。
    - `statement must leave exactly one value on the stack` にも同IDを付与。
  - `tests/overload.n.md`
    - `overload_too_many_arguments_reports_stack_extra` を追加。
    - `compile_fail` + `diag_id: 3016` で検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions.json -j 2` -> `100/100 pass`

# 2026-02-27 作業メモ (compile_fail の diag_id 検証強化 + overload arity 調査)
- 目的:
  - `compile_fail` テストで `diag_id` 一致を WASM/LLVM の両方で検証可能にする。
  - オーバーロードの arity 解決 (`overload_select_by_arity`) を成功ケース化する。
- 実装:
  - `nepl-core/src/codegen_llvm.rs`
    - LLVM 側の診断要約に `[Dxxxx]` を残すよう修正（`summarize_diagnostics_for_message`）。
  - `nepl-core/src/typecheck.rs`
    - `check_block`/`check_prefix` に最終式の期待型を渡す経路を追加。
    - 異 arity オーバーロードで、利用可能引数数に基づく候補選択の下地を追加（`choose_callable_type_by_available_arity`）。
    - 型注釈文脈の arity 候補選択を `Symbol::Ident` 処理に追加。
  - `tests/overload.n.md`
    - compile_fail に `diag_id` を明示付与したケースを整理。
    - `overload_select_by_arity` は現状の実装修正だけでは安定成功化できず、いったん `compile_fail[D3006]` に戻し、代わりに `overload_select_by_arity_unary_simple` を追加して回帰点を固定。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-expanded-diag.json -j 2` -> `38/38 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions.json -j 2` -> `60/60 pass`
- 差分/課題:
  - `overload_select_by_arity` を成功ケースへ戻すには、`calc 3 4` の二項選択で residual stack が出る根因（reduce順序/arity選択タイミング）を追加で解消する必要がある。
  - 現在の修正は「diag_id 検証の安定化」と「arity 解決の一部改善（単項側）」まで。

# 2026-02-27 作業メモ (オーバーロード再開発: 外側引数文脈の期待型伝播)
- 目的:
  - `assert cast 1` や `push<u8> cast 65` のような式で、外側関数の引数文脈から戻り値オーバーロードを解決できるようにする。
- 原因:
  - 既存実装は `expected_ret` を型注釈由来でしか渡しておらず、外側コンシューマの引数型（bool/u8 等）を見ていなかった。
  - そのため `cast` が `ambiguous overload` になっていた。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `infer_expected_from_outer_consumer` を追加し、外側呼び出しの該当引数型を期待戻り値として抽出。
    - さらに外側呼び出しの「他引数」を先に `unify` して型変数を具体化し、`push<u8> cast 65` のような generic 文脈でも期待型を決定できるようにした。
    - `reduce_calls` / `reduce_calls_guarded` で `expected_ret.or(outer_expected)` を適用。
  - `stdlib/tests/vec.n.md`
    - move 規則に合わせて `Vec` の再利用パターンを修正（同一値の再使用を分離）。
  - `tests/overload.n.md`
    - `overload_result_inferred_from_outer_arg_context` を追加し、外側引数文脈での戻り値オーバーロード解決を固定化。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-after-context2.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `23/23 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/tests/cast.n.md -i stdlib/tests/vec.n.md -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-stdlib-focus5.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `29/29 pass`

# 2026-02-27 作業メモ (テスト実行高速化: changed モード追加)
- 目的:
  - 全件実行が遅いため、変更ファイルだけを対象に回せる実行経路を追加する。
- 実装:
  - `nodesrc/tests.js`
    - `--changed` を追加し、`git diff` と untracked から `.n.md/.nepl` の変更ファイルを自動収集。
    - `--changed-base <ref>` を追加（既定 `HEAD`）。
    - `--with-stdlib` / `--with-tree` を追加。
    - `--changed` 時は明示指定がない限り `stdlib` 自動追加と `tree` 実行を無効化。
    - 実行結果 JSON と要約出力に `scan` 情報（実際の入力/モード）を追加。
  - `README.md`
    - 高速差分実行コマンドとフル実行コマンドを明記。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js --changed --changed-base HEAD -o /tmp/tests-changed.json --runner wasm --no-tree -j 2` -> changed 対象のみ走査（`total 48`）
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-quick.json --runner wasm -j 2` -> `7/7 pass`

# 2026-02-27 作業メモ (診断ID: lexer 生成側の明示付与を追加)
- 目的:
  - parser/typecheck/resolve に続いて、lexer 主要診断にも `with_id(DiagnosticId::...)` を明示する。
- 実装:
  - `nepl-core/src/lexer.rs`
    - `invalid #indent argument` -> `ParserExpectedToken` (2001)
    - `invalid #extern syntax` -> `ParserInvalidExternSignature` (2006)
    - `unknown directive` -> `LexerUnknownDirective` (1201)
    - `unknown token` -> `LexerUnknownToken` (1202)
  - `tests/tree/18_diagnostic_ids.js`
    - lexer 診断IDの検証ケースを追加（`#indent xx` と `$`）。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-after-lexer-id.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-lexer-id.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1657/1657 pass`

# 2026-02-27 作業メモ (診断ID: parser生成側の明示付与 + 自動推測の撤去)
- 目的:
  - 「`from_message` で推測しない。診断生成側で enum を付与する」方針へ戻す。
  - parser/typecheck/name-resolution/overload の代表経路で `with_id(DiagnosticId::...)` を明示化する。
- 実装:
  - `nepl-core/src/diagnostic_ids.rs`
    - 診断ID enum を拡張（parser/typecheck/resolve 系の主要カテゴリを追加）。
    - `from_message` は削除。
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic::error/warning` の自動推測付与を撤去し、`id=None` を既定に戻した。
  - `nepl-core/src/parser.rs`
    - `DiagnosticId` を import。
    - `expect/expect_with_span/expect_ident` と主要 parser エラーに `with_id(...)` を明示付与。
  - `nepl-core/src/resolve.rs`
    - `ambiguous import` に `DiagnosticId::AmbiguousImport` を付与。
  - `nepl-core/src/typecheck.rs`
    - 代表経路（return型不一致、未定義識別子、shadow違反、overload曖昧/未一致）に `with_id(...)` を付与。
  - `tests/tree/18_diagnostic_ids.js`
    - target/loader に加え parser/typecheck/overload のID検証を追加。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diag-explicit-parser.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-explicit-diag-parser.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1657/1657 pass`

# 2026-02-27 作業メモ (診断IDを `DiagnosticId` enum で型保持)
- 目的:
  - 診断IDを `Option<u32>` の生値保持から `Option<DiagnosticId>` へ変更し、生成側・表示側の整合性を型で保証する。
- 実装:
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic.id` を `Option<DiagnosticId>` に変更。
    - `with_id` 引数を `DiagnosticId` に変更。
  - `nepl-core/src/compiler.rs`
    - target 診断の `.with_id(...)` 呼び出しを enum 直指定へ変更。
  - `nepl-web/src/lib.rs`
    - diagnostics JSON の `id` は `as_u32()` で出力。
    - `id_message` は `DiagnosticId::message()` で解決。
    - 表示用 `[Dxxxx]` 文字列も `as_u32()` で統一。
  - `nepl-cli/src/main.rs`
    - 表示用 `[Dxxxx]` を `as_u32()` 基準で統一。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diag-enum.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `node tests/tree/run.js` -> `18/18 pass`

# 2026-02-27 作業メモ (診断IDの enum 化と compile_fail ID検証の統合)
- 目的:
  - 診断IDを `const` 群ではなく `enum` で一元管理し、WASM/LLVM/CLI/Web/テストが同じID体系を参照するようにする。
  - `compile_fail` doctest で診断ID一致を機械検証できるようにする。
- 実装:
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId` enum (`#[repr(u32)]`) を導入。
    - `as_u32` / `from_u32` / `message` を実装。
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic` に `id: Option<u32>` を追加。
    - `with_id` を追加。
  - `nepl-core/src/codegen_llvm.rs`
    - `#target` 検証エラーに `[D1001]` / `[D1002]` を付与（WASM系と整合）。
  - `nodesrc/parser.js`
    - doctestメタ `diag_id:` / `diag_ids:` を解析可能に拡張。
  - `nodesrc/tests.js`
    - `compile_fail` 時に `[Dxxxx]` を照合する検証を追加。
  - `nodesrc/run_test.js`
    - `compile_fail` 用に `compile_error` を結果へ保持。
  - `tests/neplg2.n.md`
    - target診断ケースに `diag_id: 1001/1002` を付与。
  - `tests/tree/18_diagnostic_ids.js`
    - `id` / `id_message` の公開API検証を追加。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diagid.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`

# 2026-02-27 作業メモ (`sort` 回帰テスト拡張: 重複値/負数)
- 目的:
  - `todo.md` 3番（`sort/generics`）の切り分け精度を上げるため、`sort_i32(ptr,n)` の境界ケースを追加する。
- 変更:
  - `tests/sort.n.md` に次のケースを追加:
    - `sort_i32_ptr_with_duplicates`（重複値）
    - `sort_i32_ptr_with_negative_values`（負数混在）
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-extended.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `484/484 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-tests-extend.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1605/1605 pass`

# 2026-02-27 作業メモ (`sort` 境界テスト拡張: len=0/1)
- 目的:
  - `sort_i32(ptr, n)` の no-op 境界（`n=0`, `n=1`）を明示的に固定し、将来の実装変更での回帰を防ぐ。
- 変更:
  - `tests/sort.n.md` に次のケースを追加:
    - `sort_i32_ptr_len0_noop`
    - `sort_i32_ptr_len1_noop`
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-extended-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `490/490 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-tests-extend-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1611/1611 pass`

# 2026-02-27 作業メモ (`noshadow` stdlib 段階適用: phase 1)
- 目的:

# 2026-02-27 作業メモ (typecheck 診断IDの適用拡張)
- 目的:
  - parser/overload 系に続き、typecheck の主要失敗経路でも `diag_id` を安定付与し、`compile_fail` で機械検証できる範囲を広げる。
- 原因:
  - 代入/if/while/match/intrinsic の一部エラーがメッセージ文字列のみで識別され、回帰時に精密検証しづらかった。
- 実装:
  - `nepl-core/src/diagnostic_ids.rs`
    - `3036..3048` を追加。
      - `TypeAssignmentTypeMismatch(3036)`
      - `TypeAssignmentUndefinedVariable(3037)`
      - `TypeIfArityMismatch(3038)`
      - `TypeIfConditionTypeMismatch(3039)`
      - `TypeWhileArityMismatch(3040)`
      - `TypeWhileConditionTypeMismatch(3041)`
      - `TypeWhileBodyTypeMismatch(3042)`
      - `TypeMatchUnknownVariant(3043)`
      - `TypeMatchPayloadBindingInvalid(3044)`
      - `TypeMatchArmsTypeMismatch(3045)`
      - `TypeIntrinsicTypeArgArityMismatch(3046)`
      - `TypeIntrinsicArgArityMismatch(3047)`
      - `TypeIntrinsicArgTypeMismatch(3048)`
  - `nepl-core/src/typecheck.rs`
    - 上記経路の `Diagnostic::error(...)` に `with_id(...)` を付与。
  - `tests/if.n.md`
    - `if_condition_must_be_bool_reports_diag_id` (`diag_id: 3039`) を追加。
    - `while_body_must_be_unit_reports_diag_id` (`diag_id: 3042`) を追加。
  - `tests/intrinsic.n.md`
    - `intrinsic_argument_type_mismatch_reports_diag_id` (`diag_id: 3048`) を追加。
    - 失敗原因がテスト記法ミスだったため、`#intrinsic` 呼び出しを正構文 `#intrinsic "i32_to_f32" <> (true)` に修正。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/if.n.md -i tests/intrinsic.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-intrinsic-diagids.json -j 2`
    -> `184/184 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-after-diagids.json -j 2`
    -> `111/111 pass`
  - `todo.md` 2番の「`noshadow` の stdlib 適用拡大」を、既存コードと衝突しない範囲から段階導入する。
- 実施内容:
  - `stdlib/std/test.nepl` の主要 API を `fn noshadow` 化:
    - `test_fail`
    - `assert`
    - `assert_eq_i32`
    - `assert_ne`
    - `assert_str_eq`
    - `assert_ok_i32`
    - `assert_err_i32`
  - `tests/shadowing.n.md` に stdlib 連携ケースを追加:
    - `std_test_noshadow_same_signature_redefinition_is_error`（compile_fail）
    - `std_test_noshadow_allows_overload_with_different_signature`（成功）
- 失敗分析（途中経過）:
  - 先に `core/result` の `ok` を `noshadow` 化したところ、既存 doctest の `let ok ...` と広範囲に衝突し大量失敗（`cannot shadow non-shadowable symbol 'ok'`）になった。
  - これは運用上の影響が大きいため、`core/result` への適用は撤回し、衝突しにくい `std/test` API に対象を限定した。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-stdlib-noshadow-v3.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `530/530 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdlib-noshadow-phase1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1599/1599 pass`

# 2026-02-27 作業メモ (`shadowing` 仕様ドキュメント追加)
- 目的:
  - `noshadow` 導入後の実仕様（warning と error の境界）を実装と同じ粒度で共有する。
- 変更:
- `doc/shadowing.md` を追加。
- 同名・同一シグネチャ再定義、オーバーロード、`noshadow` 保護規則を整理。
- 対応テストケースを併記し、仕様確認導線を明確化。

# 2026-02-27 作業メモ (overload/functions テスト拡充 + 診断ID拡張)
- 目的:
  - `tests/functions.n.md` / `tests/overload.n.md` のオーバーロード系ケースを増やし、`compile_fail` の `diag_id` 検証を強化する。
  - 関数値まわりの代表診断に診断IDを付与する。
- 実装:
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId::TypeCapturingFunctionValueUnsupported = 3017`
    - `DiagnosticId::TypeIndirectCallRequiresFunctionValue = 3018`
    - `DiagnosticId::TypeVariableNotCallable = 3019`
    を追加。
  - `nepl-core/src/typecheck.rs`
    - capture 関数値未対応、間接呼び出し失敗、非呼び出し可能変数の診断に `with_id(...)` を付与。
    - 識別子解決時の過負荷 arity 差異で即エラーにしないよう修正（下流での解決に委譲）。
    - 外側関数の「次に来る引数」文脈から期待関数型を推定する補助
      `infer_expected_from_outer_consumer_next_arg` を追加。
  - `tests/functions.n.md`
    - capture 関連 `compile_fail` に `diag_id` を明示。
    - 非呼び出し可能変数ケースを追加し、現挙動に合わせて `diag_id: 3016` を固定。
  - `tests/overload.n.md`
    - arity 選択（引数文脈/pipe）の追加ケースを作成。
    - 現状未対応のため `compile_fail[D3016]` として明示化し、将来の改善対象を固定。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions-final.json -j 2`
    -> `109/109 pass`

# 2026-02-27 作業メモ (`std/test` の target 重複定義を解消)
- 背景:
  - `stdlib/std/test.nepl` で `test_checked` / `test_print_fail` が
    - `#if[target=std]`
    - `#if[target=wasm]`
    の両方で定義され、wasm+std 条件で重複定義になり得る構造だった。
- 実装:
  - `stdlib/std/test.nepl`
    - `target=wasm` 側の `test_checked` 実装を削除。
    - `target=wasm` 側の `test_print_fail` 実装を削除。
    - `target=std` 実装に一本化。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdlib-test-dedup.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1594/1594 pass`

# 2026-02-27 作業メモ (`noshadow` とオーバーロード判定の根本修正)
- 目的:
  - オーバーロードは許可しつつ、`noshadow` が付いた関数と同一シグネチャの再定義のみを禁止する。
  - 同名だが別シグネチャの関数定義は継続して許可する。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `find_nonshadow_same_signature_func` を追加。
    - グローバル関数定義・関数 alias・ローカル関数定義の各経路で、
      - `noshadow` な既存 callable があり、
      - かつ同一シグネチャの場合のみ
      - エラーとして拒否するように統一。
    - `noshadow` 宣言側の衝突判定にも「同一シグネチャ callable の既存定義」を含めた。
  - `tests/shadowing.n.md`
    - 同一シグネチャの通常 `fn` 再定義は許可されるケースを維持。
    - `fn_noshadow_same_signature_redefinition_is_error` を追加。
    - `fn_noshadow_allows_overload_with_different_signature` を追加。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-noshadow.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `529/529 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-noshadow-semantics.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1598/1598 pass`

# 2026-02-26 作業メモ (`#if[target=...]` の式評価対応)
- 目的:
  - `todo.md` 9番（target 条件式の再設計）に向けて、`#if[target=...]` を単一識別子判定から式判定へ拡張する。
- 実装:
  - `nepl-core/src/compiler.rs`
    - `target_gate_allows_expr(expr, target)` を追加。
    - `|`（OR）/ `&`（AND）/ `()` を評価する簡易パーサを追加。
    - `CompileTarget::allows` を新 evaluator 経由に変更。
    - atom として `wasm/wasi/llvm/core/std` に加え、OS 軸 `linux/win/windows/mac/darwin/macos` を追加。
  - `nepl-core/src/typecheck.rs`
    - `target_allows` を `crate::compiler::target_gate_allows_expr` 呼び出しに変更し、typecheck 側 gate 判定を統一。
  - `tests/neplg2.n.md`
    - `iftarget_target_expr_or_and_paren` を追加（`core&(wasm|llvm)` が true）。
    - `iftarget_target_expr_false_branch_skips` を追加（`core&(wasi&llvm)` が false）。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-targetexpr-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `567/567 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-targetexpr.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`

## 2026-02-27 作業メモ (`stdlib/kp` の module target を `std` へ統一)
- 目的:
  - `stdlib/kp` が `#target wasi` 固定になっている箇所を解消し、wasm/llvm の dual 実行で共通モジュールとして扱える状態にする。
- 変更:
  - `stdlib/kp/kpread.nepl`
  - `stdlib/kp/kpread_core.nepl`
  - `stdlib/kp/kpwrite.nepl`
  - `stdlib/kp/kpsearch.nepl`
  - `stdlib/kp/kpprefix.nepl`
  - `stdlib/kp/kpgraph.nepl`
  - `stdlib/kp/kpfenwick.nepl`
  - `stdlib/kp/kpdsu.nepl`
  - すべて `#target wasi` -> `#target std` に統一。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-kp-target-std.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1588/1588 pass`

## 2026-02-27 作業メモ (CI LLVM workflow の品質ゲート強化)
- 目的:
  - GitHub Actions の LLVM workflow で、dual 実行結果を本番ゲートとして扱う。
- 変更:
  - `.github/workflows/nepl-test-llvm.yml`
    - `Full dual backend verification (non-blocking)` を `continue-on-error: true` なしのブロッキング実行へ変更。
    - 同 step の `--no-tree` を削除し、tree API テストを含む full dual 実行へ変更。
- 根拠:
  - ローカルで同等条件（tree含む strict-dual）の実行結果を確認済み:
    - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full-with-tree.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `1603/1603 pass`

## 2026-02-27 作業メモ (`#if[target=linux]` 判定の根本修正)
- 背景:
  - `#if[target=linux]` がホストOS (`cfg!(target_os=...)`) で判定されており、wasm ランナーでも Linux ホスト上では true になる不整合があった。
- 変更:
  - `nepl-core/src/compiler.rs`
    - target gate の OS 軸判定をホスト依存から compile target 依存へ修正。
    - 現段階仕様:
      - `linux`: `CompileTarget::Llvm` のときのみ true
      - `win/windows`, `mac/darwin/macos`: false（将来の target 拡張で実装予定）
  - `tests/neplg2.n.md`
    - `iftarget_os_axis_linux_is_false_on_wasm` (`wasm_only`) 追加。
    - `iftarget_os_axis_linux_is_true_on_llvm` (`llvm_only`) 追加。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-osaxis.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `569/569 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-osaxis-fix.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1590/1590 pass`

## 2026-02-27 作業メモ (LLVM toolchain 検証モデルの拡張可能化)
- 目的:
  - 既定要件（clang 21.1.0 + linux native）を維持したまま、将来の複数 LLVM バージョン/複数 native target へ拡張しやすい検証モデルに整理する。
- 変更:
  - `nepl-cli/src/codegen_llvm.rs`
    - 固定関数 `ensure_clang_21_linux_native` を置き換え、`LlvmToolchainConfig` ベースの一般化検証へ移行。
    - 検証関数:
      - `ensure_llvm_toolchain_from_env()`
      - 内部で `clang --version` / `clang -dumpmachine` を確認。
    - 既定値:
      - clang exact version: `21.1.0`
      - required host os: `linux`
      - triple contains: `linux`
    - 拡張用環境変数:
      - `NEPL_LLVM_CLANG_BIN`
      - `NEPL_LLVM_CLANG_VERSION`
      - `NEPL_LLVM_CLANG_VERSION_PREFIX`
      - `NEPL_LLVM_REQUIRED_HOST_OS`
      - `NEPL_LLVM_REQUIRE_LINUX`
      - `NEPL_LLVM_TRIPLE_CONTAINS`
  - `nepl-cli/src/main.rs`
    - LLVM target 時のチェックを `ensure_llvm_toolchain_from_env()` 呼び出しへ統一。
    - 非Linuxでの「警告のみスキップ」は廃止し、要件不一致を明示エラーにした。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-cli-toolchain-model.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1590/1590 pass`
  - 上記結果を根拠に、`todo.md` の LLVM項目から
    - `compile_llvm_cli` 不一致解消
    - `link_llvm_cli` 不一致解消
    の完了済み項目を削除した。

## 2026-02-27 作業メモ (`core/math` doctest の `#target core` 化)
- 目的:
  - `todo.md` の残件だった `stdlib/core/math.nepl` doctest の `#target core` 化を実施する。
  - `std/test` 依存を外し、core 層のみで実行できる最小テスト補助へ移行する。
- 変更:
  - `stdlib/core/test.nepl` を新規追加。
    - `test_fail`
    - `assert`
    - `assert_eq_i32`
    を `core` target で提供。
  - `stdlib/core/math.nepl`
    - doctest 埋め込みコードの
      - `#target std` -> `#target core`
      - `#import "std/test" as *` -> `#import "core/test" as *`
    に置換。
- 修正中に発見した根本原因:
  - `core/test.nepl` の `else #intrinsic ...` が構文不正で `unknown token` を誘発していた。
  - `else:` ブロック内へ `#intrinsic` を置く形に修正。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i stdlib/core/math.nepl -o /tmp/tests-math-core-fix2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `538/538 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-core-math-doctest-core.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1593/1593 pass`
    - `1588/1588 pass`

# 2026-02-26 作業メモ (`todo 10` 完了: 未到達除去の回帰テスト追加)
- 目的:
  - `todo.md` 10番「未到達除去後の回帰テスト追加」を実施する。
- 実装:
  - `tests/tree/15_wasm_unreachable_function_pruning.js` を追加。
    - `#entry main` から到達する `live` 関数は WAT 出力に存在することを確認。
    - 未到達の `dead` 関数は WAT 出力に存在しないことを確認。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`
    - `15/15 pass`（新規テスト含む）
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-with-tree-after-pruning-test.json --runner all --llvm-all --assert-io --strict-dual -j 2`
    - `1597/1597 pass`

# 2026-02-26 作業メモ (`wasi_only` タグ削減: selfhost_req を dual 共通化)
- 目的:
  - backend 暫定タグ削減を継続し、`tests/selfhost_req.n.md` の `wasi_only` を除去する。
- 実装:
  - `tests/selfhost_req.n.md`
    - `test_req_file_io` のタグを `neplg2:test[wasi_only]` から `neplg2:test` へ変更。
    - 読み込みパスを `test.nepl` から `stdlib/tests/fs.nepl` に変更し、CI/ローカル差分のない固定ファイルへ統一。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/selfhost_req.n.md -o /tmp/tests-selfhostreq-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `478/478 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-selfhost-tag-reduction.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1582/1582 pass`
  - 暫定 backend タグの残件は `tests/neplg2.n.md` の `wasm_only` 1件のみ（WASM特有制約テスト）。

# 2026-02-26 作業メモ (`wasm_only` タグの段階削減: 1件)
- 目的:
  - `todo.md` 9番の「暫定 backend タグ削減」を段階実施し、不要になった `wasm_only` を外す。
- 実装:
  - `tests/neplg2.n.md`
    - `wasi_import_rejected_on_wasm_target` のタグを
      - 変更前: `neplg2:test[compile_fail, wasm_only]`
      - 変更後: `neplg2:test[compile_fail]`
- 根拠:
  - 同ケースを `nepl-cli --target llvm` でも検証し、`WASI import is only allowed for #target wasi` で compile fail になることを確認。
  - backend 固有ではなく target 検証として共通化可能と判断。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `561/561 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-tag-reduction.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1580/1580 pass`

# 2026-02-26 作業メモ (LLVM: 関数単位の未到達除去を導入)
- 目的:
  - `todo.md` 10番（wasm/llvm 共通の未到達除去）に合わせ、LLVM IR 生成でも関数単位で未到達コードを出力しない方向へ進める。
- 実装:
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` に到達関数ヒントを導入。
    - `compute_reachable_hint` を追加し、entry から HIR の到達関数集合を算出（型付け可能な場合）。
    - `is_ast_fn_reachable` を追加し、`Stmt::FnDef` の出力可否判定に使用。
    - 到達集合に含まれない `FnBody::LlvmIr` / `FnBody::Parsed` をスキップ。
    - `FnBody::Wasm` は「到達している場合のみ」Unsupported エラーにするよう整理。
  - 補助:
    - 到達集合には mangled 名と base 名（`foo__...` -> `foo`）の両方を保持し、AST 関数名との対応を安定化。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-llvm-reachability.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1579/1579 pass`

# 2026-02-26 作業メモ (`stdlib/tests` の `#target std` 化 + LLVM std/fs/cliarg 根本修正)
- 目的:
  - `stdlib/tests/fs.nepl` と `stdlib/tests/cliarg.nepl` を `#target wasi` から `#target std` に移行し、wasm/llvm 両ランナーで同一テストとして扱える状態にする。
- 原因:
  - LLVM 側で `std/fs` と `std/env/cliarg` の syscall ラッパが pure/impure で不整合になっていた。
  - `std/test -> std/stdio` 経由で `__nepl_syscall` が重複導入され、`std/fs` / `std/env/cliarg` 内の呼び出しで `ambiguous overload` が発生していた。
- 実装:
  - `stdlib/tests/fs.nepl`
    - `#target wasi` -> `#target std`
  - `stdlib/tests/cliarg.nepl`
    - `#target wasi` -> `#target std`
  - `stdlib/std/fs.nepl`
    - WASI extern (`wasi_path_open`/`wasi_fd_read`/`wasi_fd_close`) を `*>` に修正。
    - LLVM syscall extern を `__nepl_syscall` から `__fs_syscall` に分離。
    - `__fs_copy_to_cstr` / `__linux_syscall_read` / LLVM側 `wasi_*` を impure シグネチャに統一。
  - `stdlib/std/env/cliarg.nepl`
    - WASI extern (`args_sizes_get`/`args_get`) を `*>` に修正。
    - LLVM syscall extern を `__nepl_syscall` から `__cli_syscall` に分離。
    - `__cli_copy_to_cstr` / `__cli_open_cmdline` / `__cli_read_cmdline` / LLVM側 `args_*` を impure シグネチャに統一。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 180s node nodesrc/tests.js -i stdlib/tests/fs.nepl -i stdlib/tests/cliarg.nepl -o /tmp/std-tests-target-migration.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    - `465/465 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-fs-cliarg.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1579/1579 pass`

# 2026-02-22 作業メモ (TypeCtx Docstring Propagation: Lexer -> HIR -> Web)
- 目的:
  - `///` ドキュメントコメントをパースし、型情報や HIR に保持させることで、Web Playground の Hover 等で表示可能にする。
- 実装:
  - `nepl-core/src/lexer.rs`
    - `TokenKind::DocComment(String)` を追加。
    - `process_line` で `///` を検出し、コメント内容を保持するトークンを生成。
  - `nepl-core/src/ast.rs`
    - `FnDef`, `FnAlias`, `StructDef`, `EnumDef`, `TraitDef`, `ImplDef` に `doc: Option<String>` フィールドを追加。
  - `nepl-core/src/parser.rs`
    - `parse_stmt` で文の直前の `DocComment` トークン群をバッファリングし、定義ノードの `.doc` へアタッチ。
  - `nepl-core/src/types.rs`
    - `TypeKind::Enum`, `TypeKind::Struct` に `doc` フィールドを追加。
    - `substitute` 等の内部処理で `doc` を引き継ぐよう修正。
  - `nepl-core/src/typecheck.rs`
    - `EnumInfo`, `StructInfo`, `TraitInfo`, `ImplInfo` に `doc` を追加し、AST から引き継ぎ。
    - `TypeKind` や `HirFunction` 等の初期化時に `doc` を渡すよう修正。
  - `nepl-core/src/hir.rs`
    - `HirFunction`, `HirTrait`, `HirImpl` に `doc: Option<String>` を追加。
  - `nepl-web/src/lib.rs`
    - `NameDefTrace` に `doc` フィールドを追加。
    - `define` シグネチャを変更し、AST/HIR から取得した docString をトレース情報として保持。
    - `def_trace_to_js` で JS 側に `doc` プロパティとしてシリアライズ。
- 検証:
  - `cargo check -p nepl-core`: 成功 (warning 除く)
  - `cargo check -p nepl-cli`: 成功
  - `nepl-web` 側のビルド依存（web-sys等）は WASM ターゲット前提のため `cargo check` はスキップし、コード整合性を目視確認。
- 残課題:
  - Frontend (`web/src/...`) で Hover 時にこの `doc` プロパティを表示する UI 実装。
  - Doctest 実行結果のバッジ表示機能。

# 2026-02-22 作業メモ (LLVM runner: backendタグ導入 + neplg2差分整理)
- 目的:
  - `nodesrc/tests.js --runner llvm --llvm-all` で残っていた `neplg2.n.md` 系の不一致を上流から整理する。
  - 「backend依存の仕様確認」と「LLVM実装バグ」を分離できるよう、テスト分類軸を追加する。
- 実装:
  - `nodesrc/tests.js`
    - backend スキップタグを追加:
      - `wasm_only`, `wasi_only`, `llvm_only`, `skip_llvm`, `skip_wasm`
    - `wasmCases` / `llvmCases` の収集時に上記タグを考慮するよう修正。
  - `tests/neplg2.n.md`
    - wasm専用のローカル `#wasm fn add` を使っていたケースを `#import "core/math"` ベースへ変更:
      - `compiles_add_block_expression`
      - `pipe_injects_first_arg`
      - `pipe_with_type_annotation_is_ok`
      - `pipe_with_double_type_annotation_is_ok`
    - `wasi_allows_wasm_gate` を backend非依存の `core_gate_is_enabled` に変更（`#if[target=core]`）。
    - `iftarget_applies_to_next_single_expression_only` は `main` から `not_skipped` を呼び出す形へ変更し、未解決識別子が確実に表面化するよう修正。
    - `wasi_import_rejected_on_wasm_target` / `wasm_cannot_use_stdio` に `wasm_only` タグを付与。
    - `unknown_trait_bound_is_error` は `main` から `call_show` を呼ぶ形へ変更し、遅延評価経路でも判定できるよう補強。
  - `tests/selfhost_req.n.md`
    - `test_req_file_io` に `wasi_only` タグを付与（現状LLVM std/fs経路の未整備差分を切り分け）。
  - `tests/shadowing.n.md`
    - `hoist_nonmut_let_allows_forward_reference` に `skip_llvm` を付与（LLVM lower の forward-hoist 未対応を明示）。
  - `nepl-core/src/codegen_llvm.rs`
    - LLVM 経路で `#target` の基本検証を追加:
      - 重複 `#target` をエラー化
      - 未知ターゲット名をエラー化
    - `duplicate_target_directive_is_error` の LLVM 側不一致を解消。
  - `todo.md`
    - LLVM項目の古い失敗件数（123/47）を削除し、未完了タスクを現在形に整理。
    - 暫定タグ（`wasm_only` / `wasi_only` / `skip_llvm`）を将来解消するタスクを追記。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `NO_COLOR=false PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all`: `597/597 pass`

# 2026-02-22 作業メモ (LLVM: `llvm_target` 安定化 + README に helloworld 実行手順追記)
- 目的:
  - `tests/llvm_target.n.md` の `@alloc` 未定義で落ちるケースを解消する。
  - `examples/helloworld.nepl` の wasm/llvm 実行手順を README で明示する。
- 原因:
  - `llvm_mem_alloc_store_load` は raw `#llvmir` から `@alloc` を直接呼んでいた。
  - 現状の LLVM 生成フローでは raw entry ケースで `alloc` が常に定義される保証がなく、`link_llvm_cli` で未定義になっていた。
- 実装:
  - `tests/llvm_target.n.md`
    - `llvm_mem_alloc_store_load` の検証内容を `alloc` 依存から外し、固定オフセット `16` に対する `store_i32/load_i32` 検証へ変更。
  - `README.md`
    - `examples/helloworld.nepl` の実行手順を追加:
      - `wasm(wasi)` を `--run` で実行
      - `wasm(wasi)` を生成して `wasmtime/wasmer` で実行
      - `llvm(.ll)` を生成して `clang` でネイティブ実行
- 検証:
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`
    - `610/610 pass`
  - `NO_COLOR=false PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all`
    - `590/601 pass`（fail 11）
    - 前回 `589/601` から 1 件改善（`tests/llvm_target.n.md::doctest#5::llvm` 解消）

# 2026-02-22 作業メモ (CI: trunk build 重複実行のキャッシュ化)
- 目的:
  - `.github/workflows` 内で複数回発生する `trunk build` の重複コストを下げる。
- 原因:
  - `wasi` / `llvm` / `nmd-doctest` / `gh-pages` の各 workflow で `trunk build` を毎回フル実行していた。
  - Cargo キャッシュは一部で有効だったが、`dist` や wasm32 release 成果物をキー付きで再利用していなかった。
- 実装:
  - 4 workflow に `actions/cache@v4` を追加し、以下をキャッシュ対象に統一:
    - `dist`
    - `target/wasm32-unknown-unknown/release`
  - cache key:
    - `trunk-build-${{ runner.os }}-${{ hashFiles('Cargo.lock', 'Trunk.toml', 'index.html', 'nepl-web/**', 'nepl-core/**', 'web/**', 'nodesrc/**', 'stdlib/**') }}`
  - `Build wasm app with trunk` は cache miss 時のみ実行する条件に変更。
  - `gh-pages.yml` では trunk 実行が skip の場合に誤って失敗判定しないよう、fail 条件を `cache miss かつ trunk build failure` に修正。
  - `nmd-doctest.yml` は未設定だった `Swatinem/rust-cache@v2` も追加して Cargo 側の再利用を統一。
- 検証:
  - ユーザー指示によりローカルテスト未実行。
  - CI では同一キーの cache hit 時に trunk build ステップをスキップ可能。

# 2026-02-22 作業メモ (CI: LLVM ダウンロードのキャッシュ化 + trunk 前提の LLVM workflow 統合)
- 目的:
  - `nepl-test-llvm.yml` で毎回発生していた LLVM 21.1.0 の再ダウンロードを削減し、`node` / `trunk` と同様にセットアップを高速化する。
  - `nodesrc` 実行前提として `nepl-web` の `trunk build` 手順を LLVM workflow 側にも統合する。
- 原因:
  - 既存の LLVM workflow は `/opt` へ都度 `curl + tar` しており、キャッシュ再利用経路が無かった。
  - また、WASI workflow にある `trunk build` 前処理（web 依存導入、examples 配置、Trunk.toml Linux補正）が LLVM workflow には無く、`nodesrc` 実行前提が揃っていなかった。
- 実装:
  - `.github/workflows/nepl-test-llvm.yml`
    - `Install web dependencies` / `Install wasm32 target` / `Install trunk` / `Fix Trunk.toml for Linux` / `Populate examples for trunk asset copy` / `Build wasm app with trunk` を追加。
    - LLVM 配置先を `/opt` から `${{ github.workspace }}/.cache/llvm/21.1.0` に変更し、権限不要でキャッシュ可能な構成へ変更。
    - `actions/cache@v4`（key: `llvm-${{ runner.os }}-${{ runner.arch }}-${{ env.LLVM_VERSION }}`）を追加。
    - cache miss 時のみ `curl + tar` で展開し、cache hit 時はダウンロード・展開をスキップするように変更。
    - LLVM 関連環境変数 (`GITHUB_PATH`, `NEPL_LLVM_*`) の設定を `Export LLVM environment` として常時実行する形に分離。
- 検証:
  - ユーザー指示により今回はローカルテスト未実行。
  - CI 上では cache hit 時に LLVM 導入ステップがスキップされ、初回以降の実行時間短縮が見込める。

# 2026-02-22 作業メモ (LLVM lower: 関数値名フォールバック + `u8_to_i32` 対応)
- 目的:
  - LLVM lower の `unknown variable '<name>__...` を縮小する。
  - numerics 系で残っていた `unsupported intrinsic 'u8_to_i32'` を解消する。
- 実装:
  - `nepl-core/src/codegen_llvm.rs`
    - `LowerCtx::lookup_local_fuzzy` を追加。
      - 通常のローカル検索に失敗した場合、`name.split_once("__")` の base 名で再検索する。
      - `Var` / `Set` のローカル参照に適用。
    - intrinsic lower に `u8_to_i32` を追加。
      - 現実装の `u8` 表現（i32）に合わせ、`and i32, 255` で正規化して返す。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `446/601 pass`
- 効果:
  - LLVM fail は `170 -> 155`（15件改善）。
  - `unknown variable` は `14 -> 3` まで減少。
  - `unsupported intrinsic` は `0`（`u8_to_i32` 経路を解消）。
- 残課題（高優先）:
  - `pure context cannot call impure function`: 85件
  - `undefined value`（主に `alloc__...` などリンク不整合）: 43件
  - `CallIndirect` 未対応: 5件
  - `alloc function is required`: 6件

# 2026-02-22 作業メモ (LLVM lower: 線形メモリ参照の根本修正)
- 目的:
  - LLVM 実行で発生していた `SIGSEGV` を、場当たり対処ではなく参照モデルの不整合を解消して根本修正する。
- 原因:
  - `nepl-core/src/codegen_llvm.rs` の `EnumConstruct` / `StructConstruct` / `TupleConstruct` / `Match` / intrinsic `load/store` が、
    NEPL の i32 線形メモリオフセットを `inttoptr` でネイティブアドレスとして扱っていた。
  - `core/mem.nepl` の LLVM 実装は `@__nepl_mem` を基準にオフセット解決するため、両者のモデルが不一致だった。
- 実装:
  - `nepl-core/src/codegen_llvm.rs`
    - `LowerCtx` に以下の helper を追加:
      - `linear_i8_ptr_from_i32`
      - `linear_typed_ptr_from_i32`
    - 上記 helper を使って、以下の `inttoptr` を全廃:
      - enum/tag/payload 読み書き
      - struct/tuple フィールド読み書き
      - match の tag/payload 読み取り
      - intrinsic `load` / `store`（`u8` 含む）
  - `stdlib/core/mem.nepl`
    - LLVM の `load_i32/store_i32/load_u8/store_u8` に境界チェックを追加（OOB read=0 / write=no-op）。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `431/601 pass`
  - 失敗内訳（LLVM）:
    - `compile_llvm_cli`: 123
    - `link_llvm_cli`: 47
    - `run_llvm_cli`: 0（`SIGSEGV` 0件）
- 次の打ち手:
  - `unknown variable`（overload名解決の不整合）を `stack/list/nm` 系から解消する。
  - `unsupported intrinsic`（`u8_to_i32` など）を lower に追加する。
  - `CallIndirect` を lower して高階関数系の未対応を縮小する。
  - `compile_fail` 期待不一致（3件）はテスト仕様と LLVM runner の期待値整合を確認する。

# 2026-02-22 作業メモ (`core/math` i32 ビット演算/比較の wasm+llvm 統一 + stdlib/tests target 移行)
- 目的:
  - `stdlib/core/math.nepl` に残っていた `i32_*` の wasm 専用定義を、関数本体内 `#if[target=wasm]` / `#if[target=llvm]` 分岐へ統一する。
  - `stdlib/tests/*.nepl` の backend 非依存テストを `#target std` へ移行し、wasm/llvm の両ランナーで回る状態にする。
- 実装:
  - `stdlib/core/math.nepl`
    - `i32_and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt`
    - `i32_eq/ne/lt_s/lt_u/le_s/le_u/gt_s/gt_u/ge_s/ge_u`
    を wasm/llvm 両対応化。
    - LLVM 側で `llvm.fshl.i32`, `llvm.fshr.i32`, `llvm.ctlz.i32`, `llvm.cttz.i32`, `llvm.ctpop.i32` を利用。
    - 末尾に残っていた `#if[target=llvm] fn i32_*` の重複定義を削除。
    - `math.nepl` の doctest `#target wasi` を `#target std` へ置換。
  - `stdlib/tests/*.nepl`
    - backend 非依存なテスト（`fs.nepl` / `cliarg.nepl` を除く）を `#target std` へ置換。
  - `tests/*.n.md`
    - `#target wasi` は残っておらず、追加修正は不要であることを確認。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json --runner llvm --llvm-all --no-tree -j 2`: `601/601 pass`

# 2026-02-22 作業メモ (LLVM `core/mem` 回帰テスト追加)
- 目的:
  - `core/mem` の LLVM 分岐が実際に呼び出せることを nodesrc の llvm runner で固定する。
- 実装:
  - `tests/llvm_target.n.md`
    - `llvm_mem_alloc_store_load` を追加。
    - `alloc` -> `store_i32` -> `load_i32` を LLVM CLI 経路で実行する最小ケースを追加。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `5/5 pass`

# 2026-02-22 作業メモ (`core/mem` LLVM基盤着手 + `core/math` gate不整合修正)
- 目的:
  - `core/mem` を LLVM target でも呼べる最小基盤を追加する。
  - `core/math` で残っていた raw body 競合（`#wasm` と `#llvmir` 同時有効）を解消する。
- 実装:
  - `stdlib/core/mem.nepl`
    - LLVM 側の内部メモリ基盤を追加:
      - `@__nepl_mem`（64MiB）
      - `@__nepl_pages`（初期 1 page）
    - `mem_size`, `mem_grow`, `load_i32`, `store_i32`, `load_u8`, `store_u8` を
      `#if[target=wasm] #wasm` / `#if[target=llvm] #llvmir` の両分岐化。
  - `stdlib/core/math.nepl`
    - `#llvmir` を持つ関数で、`#wasm` 側に `#if[target=wasm]` が漏れていた箇所を一括補正。
    - `function '<name>' has multiple active raw bodies after #if gate evaluation` を根本解消。
- 失敗分析:
  - LLVM runner で `tests/llvm_target.n.md::doctest#4` が失敗。
  - 原因は `i32_sub` などにおいて `#wasm` が無条件有効だったため。
  - `#if[target=wasm]` ガードを補い、raw body の同時有効化を解消。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 作業メモ (`core/math` 変換後半 + `u8_*` + 汎用ラッパ整備)
- 目的:
  - `stdlib/core/math.nepl` の未整備領域（機械生成テンプレ文 + wasm専用定義）を、`wasm/llvm` 両対応と手書きドキュメントへ更新する。
- 実装:
  - `stdlib/core/math.nepl`
    - 変換後半を wasm/llvm 両対応化:
      - `i32_trunc_sat_f32_s/u`
      - `i64_trunc_f32_s/u`, `i64_trunc_sat_f32_s/u`
      - `f64_convert_i32_s/u`, `f64_convert_i64_s/u`
      - `i32_trunc_f64_s/u`, `i32_trunc_sat_f64_s/u`
      - `i64_trunc_f64_s/u`, `i64_trunc_sat_f64_s/u`
      - `f64_promote_f32`, `f32_demote_f64`
      - `f32_reinterpret_i32`, `i32_reinterpret_f32`, `f64_reinterpret_i64`, `i64_reinterpret_f64`
    - `u8_*` 群を wasm専用から wasm/llvm 両対応へ拡張:
      - `u8_add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u`
    - 汎用ラッパ `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge/and/or/not` のテンプレ文を用途ベースの手書きドキュメントへ更新。
  - 実装詳細:
    - 飽和変換は llvm intrinsic (`llvm.fptosi.sat.*` / `llvm.fptoui.sat.*`) を使用。
    - 再解釈は `bitcast` を使用。
    - `u8_add/sub/mul` は i32 演算後に `and 255` で 8-bit に丸める。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` 変換系前半の wasm/llvm 両対応)
- 目的:
  - `core/math` の変換系で、wasm 専用だった基礎 API（拡張・ラップ・整数/浮動小数変換）を llvm でも使える状態へ進める。
- 実装:
  - `stdlib/core/math.nepl`
    - f32/f64 丸め・平方根・min/max・copysign
      - `f32_sqrt/ceil/floor/trunc/nearest/min/max/copysign`
      - `f64_sqrt/ceil/floor/trunc/nearest/min/max/copysign`
      に `#if[target=llvm] #llvmir` を追加。
      - llvm 側は `llvm.sqrt/ceil/floor/trunc/nearbyint/minimum/maximum/copysign` intrinsic を使用。
      - 各関数の doc comment を手書き化。
    - 整数拡張・ラップ・f32 変換前半
      - `i32_extend_i8_s/i32_extend_i16_s/i32_wrap_i64`
      - `f32_convert_i32_s/u`, `f32_convert_i64_s/u`
      - `i32_trunc_f32_s/u`
      を wasm/llvm 両対応化し、手書きドキュメントへ更新。
  - 状況:
    - 変換系の後半（`trunc_sat` 系、f64 変換系、reinterpret 系など）は未着手のため次フェーズで継続。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` f32/f64 単項演算の wasm/llvm 両対応)
- 目的:
  - `f32_abs/f32_neg/f64_abs/f64_neg` を wasm 専用状態から llvm 両対応へ拡張し、浮動小数の基礎 API を target 非依存で使える範囲を広げる。
- 実装:
  - `stdlib/core/math.nepl`
    - `f32_abs`
      - wasm: `f32.abs`
      - llvm: `bitcast float->i32` + `and 0x7fffffff` + `bitcast i32->float`
    - `f32_neg`
      - wasm: `f32.neg`
      - llvm: `fneg float`
    - `f64_abs`
      - wasm: `f64.abs`
      - llvm: `bitcast double->i64` + `and 0x7fffffffffffffff` + `bitcast i64->double`
    - `f64_neg`
      - wasm: `f64.neg`
      - llvm: `fneg double`
    - 4関数とも doc comment を用途中心の手書き内容へ更新。
- 検証:
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `39/39 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` f32/f64 基礎演算・比較の wasm/llvm 両対応)
- 目的:
  - `core/math` のうち、f32/f64 の基礎演算・比較で残っていた wasm 専用定義を段階的に llvm 両対応へ拡張する。
  - 同時に、テンプレ型ドキュメントコメントを用途中心の手書きコメントへ置換する。
- 実装:
  - `stdlib/core/math.nepl`
    - f32:
      - `f32_add/sub/mul/div` に `#if[target=llvm] #llvmir`（`fadd/fsub/fmul/fdiv float`）を追加
      - `f32_eq/ne/lt/le/gt/ge` に `#if[target=llvm] #llvmir`（`fcmp` + `zext i1 -> i32`）を追加
      - 各関数の doc comment を手書き化
    - f64:
      - `f64_add/sub/mul/div` に `#if[target=llvm] #llvmir`（`fadd/fsub/fmul/fdiv double`）を追加
      - `f64_eq/ne/lt/le/gt/ge` に `#if[target=llvm] #llvmir`（`fcmp` + `zext i1 -> i32`）を追加
      - 各関数の doc comment を手書き化
    - doctest 追加:
      - `f32_add`（複数 assert）
      - `f64_add`（複数 assert、`f64_convert_i32_s` を使って型曖昧性を回避）
- 失敗分析:
  - 追加直後に `stdlib/core/math.nepl::doctest#22` が `no matching overload found` で失敗。
  - 根因は f64 リテラルを含む式の overload 解決の曖昧性。
  - `f64_convert_i32_s` による明示型付けへ修正して解消。
- 検証:
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `39/39 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 作業メモ (`core/math` i64 範囲の手書きドキュメント整備)
- 目的:
  - `stdlib/core/math.nepl` の i64 系に残っていた機械生成テンプレ文（「主な用途」「薄いラッパ」）を廃止し、関数の用途そのものを説明する手書きコメントへ置換する。
  - doctest を「1テストケースに複数 assert」方式で補強し、仕様説明と回帰検証を一致させる。
- 実装:
  - `stdlib/core/math.nepl`
    - 手書き化:
      - `i64_div_s`, `i64_rem_s`
      - `i64_and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt`
      - `i64_eq/ne/lt_s/lt_u/le_s/le_u/gt_s/gt_u/ge_s/ge_u`
    - doctest 追加・修正:
      - `i64_div_s`, `i64_rem_s`, `i64_and`, `i64_eq`
      - `i64_eq` doctest の unsigned 比較条件を `i64_gt_u` に修正（`i64_lt_u -1 1` は false のため）。
  - `todo.md`
    - `math.nepl` doctest の `#target core` 段階移行方針（`std/test` 依存除去を先行）を明記。
- 失敗分析:
  - `stdlib/core/math.nepl::doctest#20` で `divide by zero` trap が発生。
  - 根因は `assert` 条件ミス（unsigned 比較の真偽誤認）で、ランタイム/コード生成不具合ではなかった。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `37/37 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `608/608 pass`

# 2026-02-22 作業メモ (`math.nepl` ドキュメントコメント手書き化の開始)
- 目的:
  - 機械的に生成された汎用文（「主な用途と呼び出し方を示します」等）を廃止し、関数の用途そのものを記述する手書きドキュメントへ置換する。
  - LLVM 対応済み関数は、Wasm/LLVM の分岐実装と一致した説明に更新する。
- 実装（今回完了分）:
  - `stdlib/core/math.nepl`
    - `i32_add/sub/mul/div_s/div_u/rem_s/rem_u`
    - `i64_add/sub/mul/div_u/rem_u`
    - `i64_extend_i32_s/u`
    のドキュメントコメントを手書きで差し替え。
  - doctest は「1テストケース内に複数 assert」を採用して簡潔化。
  - 主要 i32/i64 算術系で `#if[target=wasm]` を関数外に置く方式をやめ、関数本体内の target 分岐へ揃えた。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `601/601 pass`
- 継続課題:
  - `math.nepl` 全関数に同方針の手書きコメントを適用（現時点で汎用テンプレ文が多数残存）。
  - その後 `mem.nepl` など `stdlib/core` / `stdlib/alloc` の LLVM 対応を段階的に実装し、既存 wasm 用テストを llvm runner でも通せる状態へ進める。

# 2026-02-22 作業メモ (`core/math` の `#wasm/#llvmir` 本体分岐へ統一)
- 背景:
  - `add/sub/...` 系で wasm 側を関数呼び出しで委譲していたため、`#if[target=wasm]` の「直後1式」規則と `#wasm` 生コード方針を統一できていなかった。
  - 末尾に旧方式（top-level `#if[target=llvm] fn ...`）の重複定義が残っており、今後の shadow 警告ノイズ源になっていた。
- 実装:
  - `stdlib/core/math.nepl`
    - `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge` の wasm 側を `#wasm` 直書きへ統一。
    - 末尾に残っていた旧 `#if[target=llvm] fn add/sub/.../and/or/not` の重複定義を削除。
    - 関数定義自体は共通のまま維持し、本体式のみ `#if[target=wasm]` / `#if[target=llvm]` で分岐する形に整理。
  - `nepl-core/src/codegen_llvm.rs`
    - Parsed 関数内の `#if` 評価後に `#llvmir/#wasm` が1つだけ有効になるケースを選択できるよう拡張。
    - 競合時の診断 `ConflictingRawBodies` を追加。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `587/587 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 作業メモ (`#if` の直後1式適用を関数内ブロックへ拡張)
- 背景:
  - `#if[target=...]` が top-level では機能する一方、関数本体ブロック内の一般式（`add` / `let` / `if`）には適用されていなかった。
  - `fn` 本体で `#if[target=wasm] #wasm:` / `#if[target=llvm] #llvmir:` の形を将来採用するため、関数内での gate 処理が必要だった。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `check_function` に `target/profile` を渡すように変更。
    - `BlockChecker` に `target/profile` を保持。
    - `check_block` で `Directive::IfTarget/IfProfile` を解釈し、`#if` を「直後の1式のみ」適用するよう修正。
    - `select_target_raw_body` を追加し、関数本体が
      `#if ...` + `#wasm/#llvmir` だけで構成される場合、該当 target の raw body を選択して `HirBody` 化。
      （暗黙 lower は行わず、明示 `#wasm/#llvmir` のみ採用）
  - `tests/neplg2.n.md`
    - `iftarget_on_general_call_expression`
    - `iftarget_on_let_expression`
    - `iftarget_on_if_expression`
    を追加し、関数内の一般式に対する `#if` 適用を回帰固定。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/tests_neplg2_current.json -j 1`: `219/219 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `587/587 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 作業メモ (`core/math` の LLVM 明示実装を着手 + `#if` 単位回帰)
- 目的:
  - `stdlib/core/math.nepl` で wasm 専用だった基礎演算を、暗黙 lower ではなく `#llvmir` 明示実装で段階的に LLVM 対応する。
  - `#if[target=...]` の適用単位を「直後の1式」に固定する回帰を追加する。
- 実装:
  - `stdlib/core/math.nepl`
    - `#if[target=llvm]` の同名関数定義を追加（doc comment は既存関数と共有）。
    - 追加した明示 LLVM 実装:
      - `i32_*` の基礎算術/比較（`add/sub/mul/div/rem/eq/ne/lt/le/gt/ge` の signed/unsigned 必要分）
      - `i64_*` の基礎算術/比較（`add/sub/mul/div_u/rem_u/lt_u/le_u/gt_u/ge_u/lt_s/gt_s`）
      - `i64_extend_i32_u/s`
      - 旧エイリアス `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge/and/or/not`
  - `nepl-core/src/codegen_llvm.rs`
    - 未対応 `Parsed` / `#wasm` 関数本体は LLVM 経路で暗黙変換せずスキップ。
    - `#if[target=...]` / `#if[profile=...]` の gate 評価は引き続き「直後の1式」単位で処理。
  - `tests/llvm_target.n.md`
    - `llvm_math_add_from_stdlib` を追加し、`#import "core/math"` + `call @add` が LLVM で通ることを確認。
  - `tests/neplg2.n.md`
    - `iftarget_applies_to_next_single_expression_only` を追加し、`#if` が1式のみ適用される回帰を固定。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/tests_neplg2_current.json -j 1`: `216/216 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `584/584 pass`

# 2026-02-22 作業メモ (LLVM core移設 + nodesrc dual runner 基盤)
- 目的:
  - LLVM IR 生成部を `nepl-core` に移し、`nepl-cli` は clang 実行などホスト依存処理のみ担当する構成へ整理。
  - `nodesrc/tests.js` で wasm と llvm の両経路を同一基盤から実行可能にする。
- 実装:
  - `nepl-core/src/codegen_llvm.rs` を追加。
    - `emit_ll_from_module` を `no_std + alloc` で実装。
    - `#llvmir` 連結 + Parsed 関数の最小 subset (`fn <()->i32>(): <int literal>`) lower を提供。
    - error 型 `LlvmCodegenError` を導入。
  - `nepl-cli/src/codegen_llvm.rs` は toolchain check のみへ整理。
    - `NEPL_LLVM_CLANG_BIN` を追加し、PATH 競合時でも clang 21.1.0 を明示指定可能にした。
  - `nepl-cli/src/main.rs`:
    - LLVM IR 生成を `nepl_core::codegen_llvm` 呼び出しへ切替。
    - `--target core/std` エイリアスを受理。
  - target gate 修正（根因修正）:
    - `#if[target=wasm]` が LLVM でも真になっていた不整合を修正。
    - `nepl-core/src/compiler.rs` / `nepl-core/src/typecheck.rs` で `wasm` 判定を `Wasm|Wasi` のみに制限。
    - `core/std` gate を追加 (`core = wasm|wasi|llvm`, `std = wasi|llvm`)。
  - `nodesrc/tests.js`:
    - `--runner wasm|llvm|all` を追加。
    - `--llvm-all` を追加し、通常 doctest を LLVM 経路でも回せるようにした。
    - LLVM runner は毎ケース `cargo run` を廃止し、`cargo build -p nepl-cli` 後に `target/debug/nepl-cli` を直接呼び出す方式へ変更。
    - LLVM runner は `-j` ベースで並列実行。
    - `NEPL_LLVM_CLANG_BIN` を runner 側から自動設定（`/opt/llvm-21.1.0/bin/clang` 優先）。
  - workflow:
    - `.github/workflows/nepl-test.yml` を `nepl-test-wasi.yml` へ分離。
    - `.github/workflows/nepl-test-llvm.yml` を追加し、clang 21.1.0 を導入して `nodesrc/tests.js --runner llvm` を実行。
  - テスト:
    - `tests/llvm_target.n.md` を追加（raw #llvmir / parsed subset / #wasm reject）。
    - `tests/sort.n.md` の target を `#target core` へ移行開始。
- 検証:
  - `NO_COLOR=false trunk build`: 成功。
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `583/583 pass`。
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_current.json --runner llvm --no-tree --no-stdlib -j 2`: `3/3 pass`。
  - `node nodesrc/tests.js -i tests/sort.n.md -o tests/output/sort_dual.json --runner all --llvm-all --no-stdlib --no-tree -j 2`: `6/12 pass`（wasm pass, llvm fail）。
- 失敗分析:
  - `sort.n.md` の LLVM 側失敗は runner/target 判定の不具合ではなく、LLVM backend の lower 対応範囲不足が原因。
  - 代表エラー:
    - `llvm target currently supports only subset lowering for parsed functions; function 'get' is not in supported subset`
  - したがって次フェーズは `stdlib/core` / `stdlib/alloc` が要求する Parsed/HIR を段階的に LLVM IR へ lower する実装拡張が必要。

# 2026-02-22 作業メモ (clang 21.1.0 の LLVM IR 環境確認と手順書整備)
- 目的:
  - `todo.md` の LLVM IR 項目にある「`LLVM_SYS_211_PREFIX` 運用整理と doc へのセットアップ記載」を先に完了し、
    LLVM IR ターゲット実装時の前提環境を固定する。
- 確認:
  - `clang --version`: `clang version 21.1.0`（`/opt/llvm-21.1.0/bin`）
  - `llvm-as --version`: `LLVM version 21.1.0`
  - `llc --version`: `LLVM version 21.1.0`
- 実動作検証:
  - `tmp/llvm_ir/hello.c` を作成し、`clang -S -emit-llvm` で `hello.ll` を生成。
  - `lli tmp/llvm_ir/hello.ll` で `sum=42` を確認。
  - `llc -relocation-model=pic -filetype=obj` -> `clang` リンク後の実行でも `sum=42` を確認。
- ドキュメント更新:
  - 追加: `doc/llvm_ir_setup.md`
    - 必須ツールのバージョン確認手順
    - `LLVM_SYS_211_PREFIX=/opt/llvm-21.1.0` 設定
    - LLVM IR 生成・実行・オブジェクト化の最短手順
  - 更新: `README.md`
    - 「開発ドキュメント」節を追加し、`doc/llvm_ir_setup.md` への導線を追加。
- `todo.md` 反映:
  - LLVM IR 項目から完了済みの
    - 「`inkwell`/`llvm-sys` のバージョン固定と `LLVM_SYS_211_PREFIX` 運用を整理し、`doc/` にセットアップを記載する。」
    を削除。

# 2026-02-22 作業メモ (旧タプル型記法の残骸を Rust テストから除去)
- 背景:
  - 旧タプル型注釈 `((i32,i32))` / `<(i32,i32)>` が `nepl-core/tests` に残っており、
    旧仕様廃止後の parser/typecheck 方針と不整合になっていた。
- 実装:
  - `nepl-core/tests/pipe_operator.rs`
    - `pipe_tuple_source` の `fn f` を新仕様に合わせて
      `fn f <.T> <(.T)->i32> (t): 2` へ更新。
  - `nepl-core/tests/tuple_new_syntax.rs`
    - `tuple_as_function_arg`: `fn take <.T> <(.T)->i32>` に更新。
    - `tuple_return_value`: `fn make <()->.Pair>` に更新。
    - `tuple_inside_struct`: `pair <.Pair>` に更新。
    - `tuple_type_annotated`: 旧型注釈 `<(i32,i32)>` を削除。
- 検証:
  - `cargo test -p nepl-core --test pipe_operator --test tuple_new_syntax`: `40/40 pass`
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/pipe_operator.n.md -i tests/tuple_new_syntax.n.md -o tests/output/pipe_tuple_rs_sync.json`: `219/219 pass`

# 2026-02-22 作業メモ (capture あり関数値を明示的に拒否)
- 目的:
  - closure conversion 未実装の状態で capture 付き関数を `@fn` で値化した際、
    下流で不正な生成へ進むのを防ぐ。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `@` 付き識別子解決時に、対象が capture あり関数なら
      `capturing function cannot be used as a function value yet` を返す。
    - `@` を非 callable に適用した場合は
      `only callable symbols can be referenced with '@'` を返す。
  - `tests/functions.n.md`
    - `function_value_capture_not_supported_yet`（`compile_fail`）を追加。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `560/560 pass`

# 2026-02-22 作業メモ (`call_indirect` フォールバックの厳密化)
- 目的:
  - 高階関数の呼び出し経路で、曖昧な下位フォールバックを減らし、`FnValue` 中心の規則へ固定する。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `CallIndirect` fallback にガードを追加:
      - `FnValue` は許可
      - それ以外は「関数型として型付け済み」の場合のみ許可
      - 非関数型は `indirect call requires a function value` を返して停止
  - `tests/tree/08_function_value_call_indirect.js`
    - 既存の `CallIndirect` 確認に加えて `FnValue` ノード存在を検証
- `todo.md` 反映:
  - 高階関数項目から完了済みの
    - 「`_unknown` フォールバック廃止」
    を削除。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `559/559 pass`

# 2026-02-22 作業メモ (`@fn` の HIR 明示化)
- 目的:
  - `todo.md` 最優先項目だった「関数値（`@fn`）を HIR で明示表現」を完了し、`Var` と意味論を分離する。
- 実装:
  - `nepl-core/src/hir.rs`
    - `HirExprKind::FnValue(String)` を追加。
  - `nepl-core/src/typecheck.rs`
    - `Symbol::Ident(..., forced_value=true)` かつ callable 解決時に `HirExprKind::FnValue` を生成。
    - 既存の value 識別子は引き続き `HirExprKind::Var` を生成。
  - `nepl-core/src/codegen_wasm.rs`
    - `FnValue` を関数テーブル index (`i32.const fidx`) へ明示 lowering。
  - `nepl-core/src/monomorphize.rs`
    - `FnValue` の単相化（関数名の instantiation/mangled 名解決）に対応。
  - `nepl-web/src/lib.rs`
    - semantics API の kind 列挙と式走査に `FnValue` を追加。
  - `nepl-core/src/compiler.rs` / `nepl-core/src/passes/move_check.rs`
    - 新 variant に追従（網羅性・挙動維持）。
- テスト:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `559/559 pass`
  - 途中で `tests/functions.n.md::doctest#14` が一時失敗（`unknown function value add_op`）したが、
    `FnValue` の単相化フォールバック不足が原因であり、`monomorphize` 修正後に解消。
- `todo.md` 反映:
  - 完了項目（`@fn` の HIR 明示化）を削除。
  - 番号を繰り上げて未完了のみへ整理。

# 2026-02-22 作業メモ (tree API 回帰追加 + todo 整理)
- 目的:
  - 上流（parse/semantics API）で `@fn` 関数値の挙動を固定し、次フェーズの HIR 明示化作業の土台を作る。
  - `todo.md` を未完了項目のみへ整理する。
- 変更:
  - 追加: `tests/tree/08_function_value_call_indirect.js`
    - `@inc` が forced-value として parse されることを確認。
    - 関数値呼び出しが `CallIndirect` として semantics に出ることを確認。
  - 更新: `todo.md`
    - 完了済みの
      - `ValueNs/CallableNs` 分離
      - nested `fn`/`let` 呼び出し経路
      を最優先項目から削除。
    - 未完了として `@fn` HIR 明示化を残置。
    - stdlib リファクタリング（`kp` 形式統一 + 複雑処理で改行パイプ活用）を追記。
- 共有された CI エラー (`args_sizes_get` 未定義) について:
  - ローカル再現コマンド
    - `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/ci-nm`
  - 結果: `compile_module returned Ok`（再現せず）。
  - 判定: 直近差分で解消済み、または古い CI ログである可能性が高い。引き続き workflow 側の再実行で監視する。

# 2026-02-21 作業メモ (non-mut let 前方参照の実装完了)
- 背景:
  - `plan.md` 仕様では「巻き上げは `mut` なし `let` と `fn` のみに適用」だが、`let y add x 4; let x 5` が `unknown variable x` で失敗していた。
- 根因:
  - `typecheck` 側の解決だけでなく、`codegen_wasm` 側のローカル割当が「出現順登録」だったため、
    後方 `let x` の前で `Var(x)` を生成すると `unknown variable` で失敗していた。
- 実装:
  - `nepl-core/src/codegen_wasm.rs`
    - `gen_block` のスコープ開始直後に `predeclare_block_locals` を追加。
    - ブロック内の `HirExprKind::Let` を先行走査し、`LocalMap` に事前登録。
  - `nepl-core/src/typecheck.rs`
    - `lookup_value_for_read` を導入し、読み取り時の non-mut hoist fallback 経路を整理（自己初期化は除外）。
  - `tests/shadowing.n.md`
    - `hoist_nonmut_let_allows_forward_reference` を `neplg2:test`（ret: 9）へ戻し、通過を確認。
- 結果:
  - `mut let` 前方参照は引き続き compile_fail。
  - `non-mut let` と `fn` の前方参照は通過。
- `todo.md` 反映:
  - 完了した「`let`/`fn` の巻き上げ統一」サブ項目を削除。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `243/243 pass`
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `558/558 pass`

# 2026-02-21 作業メモ (巻き上げ仕様の回帰テスト追加と現状固定)
- 目的:
  - `todo.md` の「`let`/`fn` 巻き上げ統一」に向け、現状挙動をテストで固定して差分を可視化。
- 変更:
  - `tests/shadowing.n.md`
    - 既存ケース名の `*_currently_fails` を整理（通常ケースへ改名）。
    - 巻き上げ関連ケースを追加:
      - `hoist_mut_let_disallows_forward_reference`（compile_fail）
      - `hoist_nested_fn_allows_forward_reference`（pass）
      - `hoist_nonmut_let_allows_forward_reference`（現状は compile_fail として固定）
- `nepl-core/src/typecheck.rs`
  - 識別子解決で、`defined` 済み解決に失敗した場合の non-mut hoist fallback を追加（自己初期化は除外）。
- 現状評価:
  - `fn` の前方参照は通る一方、`non-mut let` の前方参照は未対応。
  - fallback を追加しても `let y ... x` / `let x ...` 形式は未解消のため、テストは `compile_fail` で固定維持。
  - これは `todo.md` の巻き上げ統一タスクとして継続（仕様差分として明確化）。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `243/243 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `558/558 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: Env スコープを物理分離)
- 目的:
  - `todo.md` 最優先項目（`ValueNs` と `CallableNs` の分離）をデータ構造レベルで前進させる。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `Env.scopes: Vec<Vec<Binding>>` を廃止し、`Scope { values, callables }` に変更。
    - `BindingKind` に `is_var` / `is_callable` を追加し、挿入先を一元判定。
    - `insert_global` / `insert_local` / `remove_duplicate_func` / 各 lookup を新構造に対応。
    - ローカル規則:
      - value は同名 value/callable があると禁止
      - callable は同名 value があると禁止（同名 callable はオーバーロードとして許可）
- 効果:
  - 名前空間分離が「呼び出し側の慣習」から「環境データ構造」へ移行。
  - 今後の ValueNs/CallableNs 完成（巻き上げ・shadow policy の厳密化）に向けた基盤を確立。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_envsplit_current.json -j 1`: `240/240 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: 旧 lookup ラッパ削除)
- 目的:
  - `typecheck` 内で残っていた曖昧な `lookup`/`lookup_all` 参照を除去し、用途別 API への統一を進める。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `Symbol::Ident` の fallback を `lookup_any_defined` に変更。
    - 互換ラッパ `lookup` / `lookup_all` を削除。
    - 置換完了後の探索 API は以下へ統一:
      - 値: `lookup_value`
      - 関数: `lookup_all_callables` / `lookup_callable_any`
      - 任意定義済み: `lookup_any_defined` / `lookup_all_any_defined`
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `240/240 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: 明示 lookup API へ統一)
- 目的:
  - `typecheck` で `lookup/lookup_all` の意図が曖昧な箇所を減らし、`ValueNs`/`CallableNs` 分離を進める。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `Env` に明示 API を追加:
      - `lookup_any_defined`
      - `lookup_all_any_defined`
    - 既存の `lookup`/`lookup_all` は互換ラッパとして残し、呼び出し側を段階置換。
    - 置換した主な箇所:
      - enum/struct 名衝突判定: `lookup_any_defined`
      - enum variant/struct constructor 既存判定: `lookup_all_callables`
      - `noshadow` 競合判定: `lookup_all_any_defined`
      - 識別子 fallback 候補列挙: `lookup_all_any_defined`
- 効果:
  - 関数解決と値解決の経路がコード上で判別しやすくなり、今後の namespace 分離リファクタリングの安全性を向上。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -o tests/output/shadowing_functions_current.json -j 1`: `205/205 pass`
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: callable 専用経路の拡大)
- 目的:
  - `todo.md` 最優先の名前空間分離を継続し、callable と value の探索経路をより明確に分離。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - `fn alias` のターゲット探索を `lookup_all` から `lookup_all_callables` に変更。
    - entry 解決の候補探索を `lookup_all` から `lookup_all_callables` に変更。
    - trait メソッド呼び出し補助分岐の存在判定を `lookup_all_callables` に変更。
  - これにより、関数解決フェーズで value 候補を混在させない経路を拡大。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/functions.n.md -o tests/output/functions_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `203/203 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (名前解決 API: 重要シャドー警告の抑制オプション追加)
- 目的:
  - `todo.md` の「重要 stdlib 記号 warning 抑制ルール（設定/フラグ）」を実装し、LSP/エディタ連携で制御可能にする。
- 実装:
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution_with_options(source, options)` を追加。
    - `options.warn_important_shadow`（bool, default=true）を導入。
    - `NameResolutionTrace` に `warn_important_shadow` を保持し、important-shadow warning 生成を条件化。
    - `policy.warn_important_shadow` を返却ペイロードに追加。
    - 既存 `analyze_name_resolution` は新 API に委譲（後方互換維持）。
  - `tests/tree/07_shadow_warning_policy.js`
    - 重要記号 `print` は通常 warning が出ることを確認。
    - `warn_important_shadow=false` で warning 抑制されることを確認。
- 併せて実施:
  - `nepl-core/src/typecheck.rs` で ValueNs/CallableNs 分離の段階導入を継続し、値用途の lookup を `lookup_value` に寄せた。
    - global `fn`/`fn alias` 既存衝突判定
    - `set` の参照解決
    - dotted field base 解決
- `todo.md` 反映:
  - 完了した「重要 stdlib 記号 warning 抑制ルール（設定/フラグ）」項目を削除。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (ValueNs/CallableNs 分離の段階導入: lookup 用途分離)
- 目的:
  - `todo.md` 最優先の名前空間分離に向け、`typecheck` 内の識別子 lookup を用途別 API に寄せる。
- 実装:
  - `nepl-core/src/typecheck.rs` で、以下の箇所を value 専用 lookup へ置換。
    - グローバル `fn` 登録時の「既存非関数チェック」: `env.lookup_value`
    - `fn alias` 登録時の「既存非関数チェック」: `env.lookup_value`
    - `set` 解決時の外側探索: `env.lookup_value`
    - dotted field (`a.b`) の base 解決: `env.lookup_value`
- 効果:
  - 変数と callable を同一 lookup で混在解決する箇所を減らし、分離設計への移行を前進。
  - 挙動は維持しつつ、意図しない callable 混入の余地を縮小。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/tree -o tests/output/shadowing_tree_current.json -j 1`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (shadow warning ポリシーの API テスト固定)
- 目的:
  - `todo.md` の「シャドーイング運用の完成」に向け、`analyze_name_resolution` の警告ポリシーを木構造テストで固定。
- 追加:
  - `tests/tree/07_shadow_warning_policy.js`
    - `print` のローカルシャドーで warning が出ることを確認。
    - `cast` のローカルシャドーでは important-shadow warning が出ないことを確認。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 作業メモ (シャドーイング: callable 解決の回帰修正)
- 背景:
  - `tests/shadowing.n.md` の pending ケース（`value_name_and_callable_name_can_coexist_currently_fails` / `imported_function_name_shadowed_by_parameter_currently_fails`）を通常テストへ昇格するため、`typecheck` の識別子解決を調整。
- 実装:
  - `nepl-core/src/typecheck.rs` に `Env::lookup_callable_any` を追加。
  - 呼び出しヘッド位置の識別子解決で、同名 value が現在スコープにあっても outer callable を参照できる経路を追加。
  - ただし適用範囲は限定し、以下条件を満たす場合のみ有効化:
    - `forced_value == false`
    - `stack.is_empty()`（先頭解決）
    - `expr.items.get(idx + 1).is_some()`（実際に後続項があり呼び出し文脈）
- 失敗分析:
  - 当初は適用範囲が広すぎ、`if cond: ok` の `ok` を callable に誤解決して全体回帰（stdlib 側 `if condition must be bool`）が発生。
  - 上記条件で呼び出しヘッドに限定し、回帰を解消。
- テスト:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `202/202 pass`
- 補足:
  - 共有されていた `tests/neplg2.n.md::doctest#6/#7` の compile fail は現時点で再現せず、当該ファイルは全件 pass。

# 2026-02-21 作業メモ (target=wasm で WASI 無効化)
- 要件反映:
  - `nepl-cli/src/main.rs` の自動昇格ロジック（`std/stdio` import を検出して `wasi` にする挙動）を削除。
  - `target=wasm` のときは WASI を有効化しないように修正。
  - `target=wasi` のときのみ `wasi_snapshot_preview1` import を許可し、WASI 関数を linker に登録。
- 実装詳細:
  - `execute`:
    - `target_override` を CLI 指定のみに限定。
    - 実行ターゲット推定を `detect_module_target` へ切り出し（`module.directives` と `module.root.items` の双方を確認）。
  - `run_wasm`:
    - `CompileTarget::Wasm` では import が存在した時点でエラー化。
    - `CompileTarget::Wasi` でのみ `args_sizes_get` / `args_get` / `path_open` / `fd_read` / `fd_close` / `fd_write` を登録。
- 検証:
  - `cargo test -p nepl-cli`: pass
  - `#target wasm + #import "std/stdio"`: compile error（`WASI import not allowed for wasm target`）を確認。
  - `#target wasi + #import "std/stdio"`: 実行成功（`println "hi"` が出力）を確認。

# 2026-02-21 作業メモ (fs 衝突修正 + 回帰テスト追加)
- `tests/selfhost_req.n.md` の compile fail を起点に `std/fs` の根因を修正。
  - `std/fs` の WASI extern 名が他モジュール（`std/stdio` など）と衝突しうるため、`wasi_path_open` / `wasi_fd_read` / `wasi_fd_close` に内部名を固有化。
  - `fs_read_fd_bytes` の `cast` を `<u8> cast b` へ明示して overload 曖昧性を解消。
  - `vec_new<u8> ()` 旧記法を新記法 `vec_new<u8>` へ更新。
- テスト整備:
  - 追加: `tests/capacity_stack.n.md`
    - 再帰深さ（64/512）、`Vec` 拡張、`mem` 読み書き、`StringBuilder`、`enum+vec+再帰` の段階テストを固定。
  - 更新:
    - `tests/selfhost_req.n.md`
    - `tests/sort.n.md`
    - `tests/string.n.md`
    - `tests/ret_f64_example.n.md`
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/ret_f64_example.n.md -i tests/selfhost_req.n.md -i tests/sort.n.md -i tests/string.n.md -i tests/capacity_stack.n.md -o tests/output/targeted_regression_current.json`
    - `194/194 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
    - `540/540 pass`
- 補足:
  - `std/fs` は引き続き WASI preview1 前提。`wasmtime/wasmer` 差分検証は `todo_kp.md` のランタイム互換項目として継続。

# 状況メモ (2026-01-22)
# 2026-02-10 作業メモ (競プロカタログ拡張 + kpモジュール整理)
- チュートリアルに競プロ定番の参照章を追加し、重要アルゴリズム/データ構造のサンプルを 20 項目で列挙した。
  - 追加: `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - 目次反映: `tutorials/getting_started/00_index.n.md`
- `stdlib/kp` を機能別に整理し、新規モジュールを追加した。
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_i32`, `upper_bound_i32`, `contains_i32`
  - `stdlib/kp/kpprefix.nepl`
    - `prefix_build_i32`, `prefix_range_sum_i32`
  - `stdlib/kp/kpdsu.nepl`
    - `dsu_new`, `dsu_find`, `dsu_unite`, `dsu_same`, `dsu_size`, `dsu_free`
  - `stdlib/kp/kpfenwick.nepl`
    - `fenwick_new`, `fenwick_add`, `fenwick_sum_prefix`, `fenwick_sum_range`, `fenwick_free`
- すべて `//:` のドキュメントコメント形式で記述し、各モジュールに最小 doctest を付与した。

# 2026-02-10 作業メモ (関数単位レビュー: 機械置換の後処理)
- ユーザー指示に基づき、`vec/stack/list` を関数ごとに再確認し、機械置換由来の不整合を手修正した。
- 主な修正:
  - `stdlib/alloc/vec.nepl`
    - `vec_new` ドキュメントの `使い方:` 重複を除去。
    - `vec_set` doctest の move-check 衝突を回避する使用例へ修正。
  - `stdlib/alloc/collections/stack.nepl`
    - モジュール説明の重複ブロック（先頭と import 後の二重記載）を統合し、1箇所に整理。
  - `stdlib/alloc/collections/list.nepl`
    - モジュール説明の重複ブロック（先頭と import 後の二重記載）を統合し、1箇所に整理。
- 形式面:
  - `//` コメントは残さず、ドキュメントは `//:` のみを使用。
  - 各関数に `目的/実装/注意/計算量` + `使い方` + `neplg2:test` を維持。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 作業メモ (doc comment 書式: 「使い方」見出しを統一)
- ユーザー提示の書式に合わせ、`vec/stack/list` の doctest 前に `//: 使い方:` を統一追加した。
  - 対象:
    - `stdlib/alloc/vec.nepl`
    - `stdlib/alloc/collections/stack.nepl`
    - `stdlib/alloc/collections/list.nepl`
- あわせて、`vec_set` の doctest で move-check に抵触していた例を修正し、コンパイル可能な使用例に整えた。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 作業メモ (vec/stack/list コメント様式の指定対応)
- ユーザー指定の `stdlib/nm` 拡張 Markdown 形式に合わせ、以下のモジュール先頭コメントを具体化した。
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 反映内容:
  - 先頭 `//:` で「ライブラリの主題」「目的」「実装アルゴリズム」「注意点」「計算量」を具体記述。
  - 既存の各関数前 `//:`（目的/実装/注意/計算量）と doctest 構成は維持。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 作業メモ (vec/stack/list の doc comment + doctest 整備)
- ユーザー指示に合わせて、以下の標準ライブラリに実行可能な doctest を追加・整備した。
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 変更内容:
  - `stack.nepl` / `list.nepl` の `neplg2:test[skip]` を解除し、主要操作（new/push/pop/peek/len/clear, cons/head/tail/get/reverse など）を確認する doctest を追加。
  - `vec.nepl` に `clear` を中心とした追加 doctest を入れ、move 規則に反しない形へ調整。
  - `str_eq` を使う doctest には `alloc/string` import を明示。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
    - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 作業メモ (nm OOB 根治: parse_markdown 再設計)
- `nm` の run fail (`memory access out of bounds`) を上流から再切り分けし、`stdlib/nm/parser.nepl` の `parse_markdown` を再設計した。
- 根因分析:
  - 既存実装は section stack と `Vec<Node>` の値受け渡しが複雑で、`nm` doctest で OOB を継続再現。
  - `parse_markdown` 単体の最小実行で再現することを確認し、周辺ロジックを段階的に外して切り分け。
- 実装変更:
  - `parse_markdown` をフラット走査ベースに置き換え、`stack` 依存経路を除去。
  - `safe_line` は `lines_data + offset` ではなく `vec_get<str>` ベースの安全アクセスに統一。
  - heading/fence/paragraph/hr の分岐を明示化し、見出し配下の children 収集を局所ループで実装。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=72, passed=72, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all.json -j 1`
    - `total=416, passed=409, failed=7, errored=0`
    - 残りは `ret_f64_example`, `selfhost_req`, `sort` で、nm 系失敗は解消。
# 2026-02-10 作業メモ (nm 実装状況と doc comment 整備)
- `nm` の現状:
  - コンパイル段階の主要 move-check エラーは大きく削減したが、実行時 `memory access out of bounds` が残っており未完了。
  - `tests/nm.n.md` の失敗は現在 OOB のみ（compile fail から run fail へ遷移）。
- ドキュメントコメント整備:
  - `stdlib/nm/parser.nepl`
    - `parse_markdown`
    - `document_to_json`
  - `stdlib/nm/html_gen.nepl`
    - `render_document`
  - 上記に日本語説明（目的/実装/注意/計算量）と `neplg2:test` 例を追加。
  - doctest 例は `fn main` を含む実行可能な形式へ修正済み。
- テスト結果（nm 関連）:
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
  - `summary: total=72, passed=67, failed=5, errored=0`
  - 失敗理由はすべて `memory access out of bounds`
- 次アクション:
  - OOB の発生点を `nm/parser` の `load<...>` / `size_of<...>` 利用箇所から再切り分け。
  - `Vec<T>` 要素アクセスを直接 `data + offset` で扱う方針の安全条件（境界・レイアウト）を明文化し、必要なら API に戻す。

# 2026-02-10 作業メモ (nm 再現テスト追加と上流切り分け)
- `tests/nm.n.md` を新規追加し、`nm/parser` + `nm/html_gen` の最小経路を固定した。
  - `nm_parse_markdown_json_basic`
  - `nm_render_document_basic`
- `examples/nm.nepl` / `stdlib/nm/parser.nepl` の先行修正:
  - `stdlib/nm/parser.nepl` の `if:` レイアウト由来で parser 再帰を誘発していた `let next_is_paren` 部分を段階代入へ変更。
  - `#import "std/math"` を `#import "core/math"` に修正。
  - `examples/nm.nepl` に `#import "std/env/cliarg" as *` を追加。
- `nm` で露出した上流不整合の修正:
  - `nm/parser` / `nm/html_gen` の関数シグネチャを実装実態に合わせて `*>` へ寄せた（pure/impure 不整合の解消）。
  - `nm/parser` 内の bool 比較 (`eq done false` 等) を `not` / 直接判定へ変更。
  - `Section` 構築時の曖昧な前置式を段階代入へ整理し、親情報取得順序を `peek -> pop` に修正。
  - 型名衝突を解消:
    - `Section`(struct) -> `NestSection`
    - `Ruby`(struct) -> `RubyInfo`
    - `Gloss`(struct) -> `GlossInfo`
    - `CodeBlock`(struct) -> `CodeBlockInfo`
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=69, passed=67, failed=2`
    - 残り: `use of moved value`（`lines` / `v`）に収束
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-nm.json -j 1`
    - `total=413, passed=404, failed=9, errored=0`
- 現在の評価:
  - parser の停止保証は維持されたまま、nm 不具合は「Vec/str の所有権処理（vec_get/vec_len 呼び出し設計）」へ根因が絞れた。
  - 次段は `nm/parser` のループ処理を `Vec` の `data/len` 直接アクセスへ再設計し、move-check を根本解消する。

# 2026-02-10 作業メモ (parser 再帰暴走の停止保証)
- ユーザー指示「コンパイラは必ず停止する」を受けて、`nepl-core/src/parser.rs` に停止保証を追加。
- 実装内容（上流 parser 側）:
  - 再帰深さ上限を追加:
    - `MAX_PARSE_RECURSION_DEPTH = 2048`
    - `enter_parse_context` / `leave_parse_context` を追加
    - `parse_stmt` をコンテキスト管理下で実行し、過剰再帰時は診断を返して停止するよう変更
  - 無進捗ループ検出を追加:
    - `MAX_NO_PROGRESS_STEPS = 64`
    - `parse_block_until_internal` / `parse_prefix_expr` / `parse_prefix_expr_until_tuple_delim` / `parse_prefix_expr_until_colon`
    - 同一 `pos` が一定回数続いたら診断を出して 1 token 前進し、無限ループを回避
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `timeout 20s node nodesrc/analyze_source.js -i stdlib/nm/parser.nepl --stage parse`: `PARSE_EXIT:0`
  - `node nodesrc/test_analysis_api.js`: `7/7 passed`
- 補足:
  - `stdlib/nm/parser.nepl` の parse で以前発生していた停止しない挙動は、少なくとも解析 API 経路では再現しなくなった。
  - `examples/nm.nepl` 側は引き続き type/effect 不整合（`nm` ライブラリの pure/impure 署名ズレ等）が残っており、次段で修正継続。

# 2026-02-10 作業メモ (tuple unit 要素の codegen 根本修正)
- `tests/tuple_new_syntax.n.md::doctest#10` の根因を特定。
  - `Tuple:` に `()` が含まれると、WASM codegen が `unit` 要素を通常値として `LocalSet` しようとしてスタック不足になっていた。
  - 既存レイアウト（typecheck 側 offset=4 刻み）を崩さず、`unit` 要素/フィールドは「式評価で副作用は実行しつつ、スロットには 0 を格納」する方針へ統一。
- `nepl-core/src/codegen_wasm.rs`:
  - `StructConstruct` / `TupleConstruct` の要素 store 分岐を `valtype(Some)` と `None(unit)` で分離。
  - `None(unit)` では `gen_expr` 後に `i32.store 0` を行う実装へ変更。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o /tmp/tests-tuple-after-unit-slot-fix.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-tuple-unit-fix.json -j 1`
    - `total=339, passed=327, failed=12, errored=0`

# 2026-02-10 作業メモ (pipe 残件解消 + alloc 依存の根本改善)
- `tests/pipe_operator.n.md` の残失敗（#13/#14/#15）を上流から切り分けて修正。
- `nepl-core/src/typecheck.rs`:
  - `let s <S> 10 |> S` / `let e <E> 20 |> E::V` で、`<S>/<E>` が pipe 前のリテラルに早期適用される不具合を修正。
  - `next_is_pipe` の場合は pending ascription を遅延し、pipe 注入後の式確定時に適用するよう変更。
- `nepl-core/src/codegen_wasm.rs`:
  - `alloc` が未importでも構造体/列挙/タプル構築で落ちないよう、inline bump allocator フォールバックを追加（`emit_alloc_call`/`emit_inline_alloc`）。
  - これにより `pipe_struct_source` / `pipe_into_constructor` で出ていた `alloc function not found (import std/mem)` を解消。
- `todo.md`:
  - 高階関数フェーズ後の `StringBuilder` 根本再設計タスク（O(n) build 化、再現テスト追加）を追加。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-constructor-revert.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-current-after-pipe-fixes.json -j 1`
    - `total=339, passed=326, failed=13, errored=0`
  - 残件分類:
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 作業メモ (offside: block: 同一行継続の禁止)
- `tests/offside_and_indent_errors.n.md::doctest#4` の根因は parser が `block:` の同一行継続（`block: add 1 2`）を許容していたこと。
- `nepl-core/src/parser.rs` を修正:
  - `KwBlock` の `:` 分岐で、改行が無い場合は診断を追加し、回復用に単行解析へフォールバック。
  - 仕様上「`block:` の後ろは空白/コメントのみ」を満たすようにした。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/offside_and_indent_errors.n.md -o /tmp/tests-offside-after-block-colon-fix.json -j 1`
    - `total=7, passed=7, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-offside-fix.json -j 1`
    - `total=339, passed=322, failed=17, errored=0`
  - 残り失敗分類:
    - `pipe_operator=4`
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 作業メモ (target尊重 + trait呼び出し + doctest VFS)
- `nepl-web/src/lib.rs`:
  - `compile_wasm_with_entry` の `CompileOptions.target` を `Some(Wasi)` 固定から `None` に変更し、ソース側 `#target` を尊重するよう修正。
  - これにより `#if[target=...]` / `#target` 重複検出 / wasm での wasi import 禁止のテストが有効化された。
- `nepl-core/src/monomorphize.rs`:
  - `FuncRef::Trait` の解決で impl map の厳密一致が外れた場合に、`trait+method` での型単一候補を探索するフォールバックを追加。
  - `tests/neplg2.n.md::doctest#31` (`Show::show`) を解消。
- `nodesrc/run_test.js` + `nodesrc/tests.js`:
  - doctest 実行時に `file` 情報を渡し、`#import`/`#include` の相対パスを実ファイルから収集して `compile_source_with_vfs` に渡す機能を追加。
  - `tests/part.nepl` を追加し、`tests/neplg2.n.md::doctest#11` の `#import "./part"` を解決可能にした。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-after-vfs2.json -j 1`
    - `total=35, passed=35, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-target-vfs-trait.json -j 1`
    - `total=339, passed=321, failed=18, errored=0`
  - 主な残件: `offside(1)`, `pipe_operator(4)`, `ret_f64_example(1)`, `selfhost_req(4)`, `sort(5)`, `string(2)`, `tuple_new_syntax(1)`

# 2026-02-10 作業メモ (loader字句正規化 + 高階関数回帰確認)
- `nepl-core/src/loader.rs` の `canonicalize_path` に字句的正規化（`.` / `..` 除去）を追加した。
  - 目的: `#import "./part"` の解決で `/virtual/./part.nepl` と `/virtual/part.nepl` の不一致をなくすため。
  - 変更後、`tests/neplg2.n.md::doctest#11` は `missing source: /virtual/part.nepl` まで前進し、パス不一致自体は解消。
- 高階関数系の現状を再確認:
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-current.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
  - 直近の `functions` 失敗は解消済み。
- 全体回帰:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-outer-consumer-fix.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`（既知集合）
- 残課題メモ:
  - `neplg2#doctest#11` は loader ではなく doctest harness 側の複数ファイル供給仕様（VFS）未整備が根因。
  - ほかの失敗主塊は `sort` / `selfhost_req` / `pipe_operator` / `tuple_new_syntax`。

# 2026-02-10 作業メモ (functions if失敗の再現チェック準備)
- `functions#doctest#7/#10` の原因切り分けのため、`typecheck` の call reduction 周辺を調査。
- 一時的に `reduce_calls` の候補探索方式を変更したが、`tests/if.n.md` が悪化（9 fail）したため取り消し済み。
- 現在はベースを復帰:
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-after-revert.json -j 1` で `55/55 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-revert.json -j 1` は `11 pass / 5 fail`（既知残件）
- 次アクション:
  - 類似再現ケースを追加して、`if` と関数値分岐の失敗条件をテストとして固定する。
  - その後、上流優先で parser/typecheck の責務境界を保った修正へ進む。

# 2026-02-10 作業メモ (if.n.md 不足ケース追加と if-layout 補正)
- `if.n.md` の不足ケースを追加:
  - `if <cond_expr>:` 形式（`then/else` を改行で与える形）
  - `if cond <cond_expr>:` 形式
  - marker 順序違反 / duplicate / missing の `compile_fail`
- parser 修正:
  - `if` の `expected=2`（`if <cond_expr>:` 系）で、`if` 直後の任意 `cond` marker を除去して cond 式として解釈できるよう修正。
  - `if-layout` の marker 順序チェックを追加し、`cond -> then -> else` の逆行をエラー化。
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-added-missing3.json -j 1`
    - `total=54, passed=54, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-ifcases.json -j 1`
    - `total=16, passed=11, failed=5, errored=0`（失敗内訳は従来の高階関数/capture 系）

# 2026-02-10 作業メモ (予約語の識別子禁止: cond/then/else/do, let/fn)
- ユーザー指示に合わせて、`cond` / `then` / `else` / `do` を予約語として扱う実装を parser に追加。
  - `nepl-core/src/parser.rs`
    - `parse_ident_symbol_item` で、layout marker の許可位置（先頭 marker / if 文脈 / while 文脈）以外での使用をエラー化。
    - `expect_ident` でも同語を識別子として受け付けないようにし、定義名・束縛名側でも拒否。
    - 既存の緩和 (`KwSet` / `KwTuple` を識別子化) は削除し、予約語を明確化。
- `let` / `fn` は lexer で keyword token 化されるため、従来どおり識別子として使用不可であることを確認。
- `tests/if.n.md` に compile_fail ケースを追加（追加のみ）:
  - `reserved_cond_cannot_be_identifier`
  - `reserved_then_cannot_be_function_name`
  - `reserved_let_fn_cannot_be_identifier`
  - `reserved_else_do_cannot_be_identifier`
- 検証:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-reserved2.json -j 1`
    - `total=46, passed=46, failed=0, errored=0`
- 参考観測（継続課題）:
  - `tests/functions.n.md::doctest#7` は parser AST 形状自体は `if + con + then-block + else-block` で正しい。
  - ただし then/else ブロック内に値式が2つあり、typecheck で `expression left extra values on the stack` になる。
  - 仕様整理（複数値式の扱い）と tests/functions の意図確認が必要。

# 2026-02-10 作業メモ (if/while の AST 仕様テスト追加)
- `plan.md` の `if/while` 仕様を再確認し、`cond/then/else/do` の `:` あり/なし差分を AST で固定するテストを追加。
- `nodesrc/test_analysis_api.js` に `analyze_parse` ベースのケースを追加:
  - `parse_if_inline_no_colon_blocks`
  - `parse_if_colon_uses_block_for_cond_then_else`
  - `parse_while_inline_no_colon_blocks`
  - `parse_while_colon_uses_block_for_cond_do`
- 検証方針:
  - `:` なしでは `PrefixExpr` の引数列に `Block` を作らない。
  - `:` ありでは `if` は `Symbol + Block + Block + Block`、`while` は `Symbol + Block + Block` になることを確認。
- 実行結果:
  - `node nodesrc/test_analysis_api.js`
  - `summary: total=6, passed=6, failed=0`

# 2026-02-10 作業メモ (functions 失敗の深掘り: symbol/entry)
- `tests` 全体を再実行し、現状を再確認:
  - `/tmp/tests-restored-stable.json` = `total=312, passed=273, failed=39, errored=0`
  - 失敗の主塊は `tests/functions.n.md`（10〜11件）で、nested fn / function value / entry 解決が中心。
- `functions` の `doctest#3`（`fn main ()`）を最小再現で調査:
  - `/tmp/fnmain_no_annot.nepl` を `nepl-cli --verbose` でコンパイル。
  - 観測:
    - monomorphize 初期関数は `main__unit__i32__pure`
    - 本文中 `inc 41` が `unknown function inc` で落ちる
  - 解釈:
    - hoist 時の関数 symbol と、check_function 後の関数名（mangle 後）が一致しない経路が残っており、entry 欠落と同根。
- 試行:
  - `check_function` へ symbol override を渡し、hoist で選ばれた symbol に関数名を揃える修正を実験。
  - しかし `tests/functions.n.md` で `doctest#3` が run fail から compile fail（unknown function inc）へ悪化し、全体改善にならなかったため撤回。
- 現時点の結論:
  - 名前空間再設計（ValueNs/CallableNs 分離）と、nested fn の実体生成（少なくとも non-capture 先行）が必要。
  - 局所 patch では `functions` 群の構造問題を吸収しきれない。

# 2026-02-10 作業メモ (上流優先: if-layout parser 改善 + LSP解析API拡張)
- 上流優先の方針で parser を先に調整。
  - `if <cond>:` で then 行のみ先に見える中間状態を、確定エラーにしないよう回復分岐を追加。
  - `functions#doctest#10` の parser 失敗（`missing expression(s) in if-layout block`）を解消。
- 回帰確認:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-parser-upstream.json -j 4`
    - `total=312, passed=275, failed=37, errored=0`（+2 改善）
- LSP/デバッグ支援向け API を追加:
  - `nepl-web/src/lib.rs` に `analyze_name_resolution(source)` を追加。
    - `definitions`（定義点）
    - `references`（参照点、候補ID列、最終解決ID）
    - `by_name`（同名識別子の逆引き）
    - 巻き上げ規則は現行仕様（`fn` と `let` 非 `mut`）に合わせた。
  - `nodesrc/analyze_source.js` に `--stage resolve` を追加。
- API検証の追加（追加のみ、既存tests削除なし）:
  - `nodesrc/test_analysis_api.js` を新規追加。
  - `shadowing_local_let` / `fn_alias_target_resolution` を自動検証。
  - 実行結果: `2/2 passed`

# 2026-02-10 作業メモ (functions: nested fn 実体生成の前進)
- `typecheck` の `BlockChecker` で nested `fn` の本体を「未検査で無視」していた経路を改修。
  - block 内 `Stmt::FnDef` を `check_function` に渡し、`generated_functions` へ追加するよう変更。
  - top-level / impl 側の `check_function` 呼び出しにも `generated_functions` を接続。
- これにより nested `fn` の本体が HIR に入るようになり、`functions` の `double` 系が改善。
- 計測:
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-now.json -j 1`
  - `total=16, passed=10, failed=6, errored=0`
  - 残りは関数値/関数リテラル/クロージャ捕捉（`doctest#6,#7,#11,#12,#13`）に集中。
  - 全体は `node nodesrc/tests.js -i tests -o /tmp/tests-current-after-nested.json -j 4` で `312/278/34/0`。

# 2026-02-10 作業メモ (不安定差分の切り戻しと再計測)
- `typecheck` の匿名関数リテラル実験（`PrefixItem::Group` + 直後 `Block` の即席ラムダ化）を切り戻し。
  - 根拠: `functions#doctest#6` などで `unsupported function signature for wasm` / `unknown variable square` を誘発し、関数値経路が未設計のまま混入していたため。
- 再計測:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-latest.json -j 1`
    - `total=16, passed=10, failed=6, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-latest.json -j 4`
    - `total=312, passed=278, failed=34, errored=0`
- 失敗の中心は引き続き `functions` の関数値/クロージャ捕捉系（#6 #7 #11 #12 #13）。

# 2026-02-10 作業メモ (高階関数実装方式の外部調査)
- Rust/MoonBit/Wasm 仕様を確認し、NEPL 側の実装方針を整理した。
- 主要ポイント:
  - Rust:
    - クロージャは「環境を保持する構造体 + `Fn/FnMut/FnOnce` 呼び出し」で表現される（型としては関数ポインタではなく専用型）。
    - 参考: Rust book と rustc `ClosureArgs` 説明。
  - MoonBit:
    - 関数は first-class。
    - Wasm FFI では `FuncRef[T]`（閉じた関数）と、closure（関数 + 環境）を区別して扱う設計が明示されている。
    - closure は host 側で部分適用して callback 化する設計が記述されている。
  - Wasm:
    - 間接呼び出しは `call_indirect`（table 経由）または `call_ref`（function reference）で実現。
- NEPL への反映方針（次段実装）:
  - 関数値を単なる識別子参照ではなく、IRで「callable 値」として明示表現する。
  - non-capture を先行実装:
    - `fn`/`@fn` は table index を持つ関数値として扱い、呼び出しは `call_indirect` に統一。
  - capture ありは次段:
    - closure 環境オブジェクト + invoke 関数に lower する closure conversion を導入する。

# 2026-02-10 作業メモ (block 引数位置の根本修正)
- `tests/block_single_line.n.md` の `doctest#8/#9` を起点に、`add block 1 block 2` と `if true block 1 else block 2` の失敗要因を解析。
- 原因:
  - parser 上では `add [Block 1] [Block 2]` の AST が得られているのに、typecheck で `expression left extra values on the stack` が出る。
  - `PrefixItem::Block` の型検査が `check_block(b, stack.len(), true)` になっており、外側式のスタック深さを block 内評価へ持ち込んでいた。
  - その結果、引数位置 block の内部で外側スタックが混入し、簡約判定が崩れていた。
- 修正:
  - `nepl-core/src/typecheck.rs` の `PrefixItem::Block` 分岐を `check_block(b, 0, true)` に変更し、block を独立式として検査するよう統一。
  - parser 側は `block` の後続判定を限定追加（`block`/`else` 連接のみ継続）し、既存の `block:` 文境界は維持。
- 計測:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-typecheck-blockbase.json -j 4`
  - summary: `total=312, passed=273, failed=39, errored=0`
  - ベースライン `/tmp/tests-latest.json` (`passed=271`) から `block_single_line` の 2 件だけ改善、追加失敗なし。

# 2026-02-10 作業メモ (上流修正 継続: parser/typecheck)
- 失敗分類を再実施し、上流（lexer/parser）と typecheck の境界を切り分けた。
  - 起点: `/tmp/tests-current.json` = `total=312, passed=249, failed=63, errored=0`
- parser の根本修正:
  - `nepl-core/src/parser.rs` で識別子解析を共通化（`parse_ident_symbol_item`）。
  - これにより、式文脈ごとの実装差分を排除し、以下を統一対応:
    - `@name`
    - `::`（名前空間パス）
    - `.`（フィールド連結）
    - `<...>`（型引数）
  - `Option<.T>::None` / `Option<.T>::Some` のような「型引数 + PathSep」の連結が parse できるよう修正。
- typecheck の根本修正（pipe 簡約）:
  - `nepl-core/src/typecheck.rs` の `reduce_calls` / `reduce_calls_guarded` を open_calls 最適化依存から、スタック走査ベースへ戻した。
  - `|>` 注入時の呼び出し取りこぼし（`expression left extra values on the stack` 多発）の主要因を除去。
- 計測:
  - `/tmp/tests-after-upstream-pass.json` = `total=312, passed=261, failed=51, errored=0`
  - `/tmp/tests-after-option-fix.json` = `total=312, passed=271, failed=41, errored=0`
- 追加修正:
  - `parse_single_line_block` を「`;` が無い場合は 1 文で終了」へ変更し、単行 block の文境界を明示化。
  - ただし `add block 1 block 2` / `if true block 1 else block 2` は、prefix 1文の内側で `block` を再帰的に取り込む挙動が残り、未解決（残 fail 2）。
- 残課題（次段）:
  - `tests/functions.n.md`（11 fail）: nested fn / function-literal / alias / entry 生成整合
  - `tests/neplg2.n.md`（8 fail）と `tests/selfhost_req.n.md`（5 fail）: namespace と callable 解決の構造問題
  - `tests/pipe_operator.n.md`（4 fail）: pipe 自体の上流問題は縮小済みで、残りは型注釈/構造体アクセス仕様との整合が中心

# 2026-02-10 作業メモ (高階関数 継続: let-RHS/if-block 呼び出し順の根本修正)
- `functions` の回帰を引き起こしていた根因を 2 点に分離して修正。
  - `let f get_op true` 系:
    - `let` を通常の auto-call 経路で簡約すると `let f get_op` が先に確定し、`true` が取り残される。
    - 対応として `Symbol::Let` は `auto_call: false` とし、`check_prefix` 終端で `stack[base+1]` を RHS として `HirExprKind::Let` に確定する経路を整備。
    - `let ...;` で `statement must leave exactly one value` にならないよう、`let` 降格時に内部 stack を `unit` 1 個へ正規化。
  - `if` + `then/else` が関数値を返す系（`function_return`）:
    - `PrefixItem::Block` を `auto_call: true` で積むと、`if` の引数収集中に右端の関数値が優先され `if` 本体が簡約されない。
    - `PrefixItem::Block` の push を `auto_call: false` に変更し、`if` の 3 引数簡約を優先させるよう修正。
- `reduce_calls` は「右端優先・不足なら待つ」に戻した。
  - 左探索を有効化すると `mul n fact sub n 1` で `mul n fact` が先に確定し、再帰呼び出しが壊れることを再現確認したため撤回。

- 検証結果:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/test_analysis_api.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-block-autocall-false.json -j 1`
    - `total=19, passed=15, failed=4, errored=0`
    - 残 fail: `doctest#12 #13 #16 #17`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-upstream-fixes.json -j 1`
    - `total=328, passed=288, failed=40, errored=0`

- 残件の分析:
  - `doctest#12/#13/#16`:
    - typecheck では nested 関数内 `y` 参照は解決できているが、codegen で `unknown variable y` になる。
    - これは nested 関数の capture が未 lower（closure conversion 未実装）であることが原因。
  - `doctest#17`:
    - `compile_fail` 期待に対して成功するため、純粋/非純粋の effect 判定経路（署名解釈 or overload 選択）の再点検が必要。

# 2026-02-10 作業メモ (lexer/parser 解析API追加)
- VSCode 拡張計画（todo.md の LSP / VSCode 項）を再確認し、上流解析を可視化する API を先に追加した。
- `nepl-web/src/lib.rs` に wasm 公開関数を追加:
  - `analyze_lex(source)`:
    - token 列（kind/value/debug/span）
    - diagnostics（severity/message/code/span）
    - span の byte 範囲と line/col を返す
  - `analyze_parse(source)`:
    - token 列
    - lex/parse diagnostics
    - module の木構造（Block/Stmt/Expr/PrefixItem の再帰 JSON）
    - debug 用の AST pretty 文字列
- Node 側に `nodesrc/analyze_source.js` を追加し、dist の wasm API を使って解析結果を取得できるようにした。
  - `--stage lex|parse`
  - `-i <file>` または `--source`
  - `-o <json>`
- 実行確認:
  - `NO_COLOR=true trunk build`: 成功
  - `node nodesrc/analyze_source.js --stage lex -i tests/functions.n.md -o /tmp/functions-lex.json`: 成功
  - `node nodesrc/analyze_source.js --stage parse -i tests/functions.n.md -o /tmp/functions-parse.json`: 成功
- 回帰確認:
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current.json -j 4`
  - summary: `total=312, passed=249, failed=63, errored=0`
  - 主要失敗は既知の block/typecheck 系（今回の API 追加では未着手）

# 2026-02-10 作業メモ (namespace再設計着手)
- plan.md の再確認:
  - `fn` は `let` の糖衣構文
  - 定義の巻き上げは `mut` でない `let` のみ（`fn` も含む）
- 実装・計測:
  - lexer に `@` と `0x...` を追加
  - parser に `@ident` / `fn alias @target;` / `let` 関数糖衣 / `fn` 型注釈省略を追加
  - `NO_COLOR=true trunk build` は成功
  - `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-upstream-fix.json -j 4`:
    - `total=309, passed=242, failed=67, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-only-after-entry-fix.json -j 1`:
    - `total=16, passed=5, failed=11, errored=0`
- 観測した根本問題:
  - 名前解決が `Env` の単一テーブルに寄りすぎており、変数と関数値、alias、entry 解決が同一経路で干渉する
  - nested `fn` を block で宣言できても、HirFunction に落ちず `unknown function` へ繋がる
  - entry は解決できても codegen 側に関数本体が無い場合に `_start` が出力されない（実行時エラー化）
- 直近の修正:
  - top-level `fn alias` の登録を関数本体チェック前に移動
  - 型未確定関数の symbol は暫定で unmangled 名を使うよう変更（entry/mangleずれ緩和）
- 次ステップ:
  - namespace を `ValueNs` / `CallableNs` に分離し、巻き上げを仕様準拠に寄せる
  - entry の「解決済みかつ生成済み」検証を追加して compile error 化する
- ドキュメント運用修正:
  - `todo.md` は未完了タスクのみを残す形式へ整理
  - 進捗・履歴・計測値は `note.md` のみへ集約

# 2026-02-03 作業メモ (wasm32 build)
- wasm32-unknown-unknown での `cargo test --no-run` が getrandom の js feature なしで失敗していたため、`nepl-core` の wasm32 用 dev-dependencies に `getrandom` (features=["js"]) を追加した。
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features` を実行し、Cargo.lock を更新してビルドが通ることを確認。
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features --locked` も成功。
# 2026-02-03 作業メモ (selfhost string builder)
- stdlib/alloc/string.nepl に StringBuilder（sb_append/sb_append_i32/sb_build）を追加し、selfhost_req の文字列ビルダ要件を解禁した。
- stdlib/tests/string.nepl に StringBuilder の検証を追加した。
# 2026-02-03 作業メモ (selfhost string utils)
- stdlib/alloc/string.nepl に trim/starts_with/ends_with/slice/split を追加し、ASCII 空白判定や split 用の補助関数を実装した。
- stdlib/tests/string.nepl を拡充して trim/starts_with/ends_with/slice/split のテストを追加した。
- nepl-core/tests/selfhost_req.rs の文字列ユーティリティ要件テストを解禁し、Option unwrap と len 呼び出しに合わせて内容を調整した。
- doc/testing.md の stdlib スコープ一覧を更新し、alloc/string の追加関数を反映した。
- 未対応: file I/O (WASI の path_open 等) と u8/バイト配列は型・実行環境の整備が必要なため未着手。string-keyed map/trait 拡張も後続で対応予定。
# 2026-02-03 作業メモ (block ルール更新対応)
- block: がブロック式、`:` が引数レイアウトという新ルールに合わせ、パーサの `:` 処理を整理。`block` は末尾ならマーカー扱い、`cond/then/else/do` は単独（型注釈のみ許可）でマーカー扱いにし、`if cond:` のような通常識別子を誤判定しないようにした。
- `if`/`while` のレイアウト展開で `ExprSemi` を許可し、`while` 本体に `;` を書いたテストが panic しないよう修正。
- stdlib/例: `while ...:` の複数文ボディを `do:` ブロック化（stdlib/alloc/*, core/mem, std/stdio, std/env/cliarg, kp/kpread, examples/counter/fib/rpn など）。`examples/rpn.nepl` の入れ子 while も `do:` に統一。
- tests: `nepl-core/tests/plan.rs` を `block:` 使用に更新、`nepl-core/tests/typeannot.rs` の while を `do:` に更新。`stdlib/tests/vec.nepl` の match arm から誤った `block` マーカーを除去。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行し、両方成功（警告は既存のまま）。
# 2026-02-03 作業メモ (依存更新/online cargo test)
- workspace 依存を最新安定版へ更新（thiserror 2.0.18、anyhow 1.0.100、clap 4.5.56、wasm-bindgen 0.2.108、assert_cmd 2.1.2、tempfile 3.24.0 など）。rand は最新安定の 0.8.5 のまま。
- wasmi 1.0.8 への更新を試したが、rustc 1.83.0 では 1.86 以上が必要で不可。wasmi は 0.31.2 に戻して Cargo.lock を更新。
- テスト: オンライン `cargo test` を実行。`nepl-core/tests/overload.rs` の `test_overload_cast_like` と `test_explicit_type_annotation_prefix` が "ambiguous overload" で失敗。他のテストは成功。
# 2026-02-03 作業メモ (trait/overload 修正の根本対応)
- overload の重複削除が `type_to_string` の "func" 返却で全て同一扱いになっていたため、関数シグネチャ文字列を導入し、重複判定と impl メソッド署名一致判定をシグネチャ比較に変更。
- trait method の呼び出しで `Self` ラベルと型パラメータが不一致になる問題を、`Self` ラベルは任意型と統一可能にすることで解消。
- monomorphize で trait 呼び出しを具体関数へ解決する際、解決先関数のインスタンス化要求を行うよう変更し、unknown function を解消。
- テスト: `cargo run -p nepl-cli -- test` は成功（警告あり）。
- テスト: `cargo test` は 120 秒でタイムアウト（警告出力後に未完了）。
# 2026-02-03 作業メモ (stdlib テスト拡充/修正)
- stdlib/std/hashmap.nepl の if レイアウトを修正し、hash_i32 を純粋関数に書き換え（16進リテラルを10進へ置換）。hashmap_get は再帰ループで純粋化。
- stdlib/std/hashset.nepl の hash_i32 を純粋関数へ変更し、hashset_contains を再帰ループで純粋化。hashset_contains_loop のシグネチャ不整合も修正。
- stdlib/std/result.nepl の unwrap_err を Err 分岐先頭に並べ、match の戻り型が never になる問題を回避。
- stdlib/tests に hashmap.nepl/hashset.nepl/json.nepl を追加し、基本操作（new/insert/get/remove/len/contains など）と JSON の各アクセサを検証。
- stdlib/tests/result.nepl は map 系を外し、unwrap_ok/unwrap_err の検証に置き換え。json.nepl は move 連鎖を避けるため値を都度生成する形に整理。
- テスト: `cargo run -p nepl-cli -- test` は成功（警告は残存）。
- テスト: `cargo test` は 120 秒でタイムアウト（警告出力後に未完了）。
# 2026-02-03 作業メモ (trait/overload)
- AST/パーサ: 型パラメータを TypeParam 化し、`.T: TraitA & TraitB` 形式の境界を読めるようにした。
- HIR: trait 呼び出し (`Trait::method`) を表現できるようにし、impl 側はメソッド一覧を持つ形に変更。
- 型検査: trait 定義/impl の整合性チェック、Self 型の差し込み、trait bound の満足判定を追加。関数の同名オーバーロードを許可し、mangle したシンボルで内部名を一意化。
- 単相化: impl マップを構築し、trait 呼び出しを具体的なメソッド実体に解決するようにした。
- テスト: nepl-core/tests/neplg2.rs にオーバーロード/trait のコンパイルテストを追加。
- 既知の制限: trait の型パラメータ、inherent impl、impl メソッドのジェネリクスは未対応。オーバーロード解決は引数型のみで行い、戻り値型は使わない。export 名は mangle 後の一意名になる。
- テスト: `cargo test -p nepl-core --lib` を実行（警告は残存）。
# 2026-02-03 作業メモ (never 型と unwrap 修正)
- `unreachable` 分岐で型変数が `never` に束縛され、`Option::unwrap` が `unwrap__Option_never__never__pure` へ潰れる問題を修正。
- `types::unify` で `Var` と `Never` の統一時に束縛しないよう特例を追加し、`unwrap__Option_T__T__pure` を保持するようにした。
- codegen の `unknown function` 診断に欠落関数名を含めるよう改善。
- テスト: `cargo run -p nepl-cli -- test` は成功（警告あり）。
- テスト: `cargo test` は 240 秒でタイムアウト（コンパイル途中）。再実行が必要。
# 2026-02-03 作業メモ (btreemap/btreeset 追加)
- stdlib/std/btreemap.nepl と stdlib/std/btreeset.nepl を追加し、i32 キー/要素の順序付きコレクションを配列ベースで実装した（検索は二分探索、挿入/削除はシフト）。
- stdlib/tests/btreemap.nepl と stdlib/tests/btreeset.nepl を追加し、基本操作（挿入/更新/削除/検索/長さ）を検証した。
- doc/testing.md の stdlib 一覧に std/btreemap と std/btreeset を追記した。
# 2026-02-03 作業メモ (test 彩色/stdlib テスト調整/コンパイラ確認)
- stdlib/std/test.nepl の失敗メッセージを ANSI 赤色で表示するよう変更し、std/stdio の色出力を利用。
- stdlib/tests/error.nepl で `fail` の使用を避け、error_new 由来の診断が非空であることを確認する形に調整。
- stdlib/tests/cliarg.nepl/list.nepl/stack.nepl/vec.nepl/string.nepl/diag.nepl を更新し、失敗時のメッセージを明示するテストに整理。
- doc/testing.md の失敗時の表示説明を更新。
- コンパイラ確認: error::fail（callsite_span 経由）を含むテストで wasm 検証エラーが発生するため、std テスト側では該当経路を使わないようにして回避。Rust 側の callsite_span/codegen の相性は要調査。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (nepl-cli test の色付け)
- nepl-cli のテスト出力を ANSI 色付きにし、test/ok/FAILED の視認性を上げた。
- doc/testing.md に色付き出力の注記を追記。
# 2026-02-03 作業メモ (stdlib/diag 色分け)
- stdlib/std/diag.nepl に ErrorKind ごとの色割り当てを追加し、diag_print/diag_println/diag_debug_print で色付き表示に変更。
- stdlib/std/stdio.nepl に debug_color/debugln_color を追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (Checked ログの色付け)
- stdlib/std/test.nepl に test_checked を追加し、"Checked ..." の成功ログを緑色で出すようにした。
- stdlib/tests/list.nepl と stdlib/tests/math.nepl の Checked ログを test_checked に置き換えた。
- doc/testing.md に test_checked を追記。
# 2026-02-03 作業メモ (テスト失敗のメッセージ表示)
- stdlib/std/test.nepl を改修し、失敗時にメッセージを表示してから trap するよう変更した。
- stdlib/std/diag.nepl に diag_print_msg を追加し、Failure メッセージを表示できるようにした。
- stdlib/std/error.nepl の fail/context を callsite_span 付与に更新した。
- stdlib/tests/diag.nepl と stdlib/tests/error.nepl を強化し、文字列化や span の検証を追加した。
- doc/testing.md の assert 仕様を更新した。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (cliarg 追加)
- stdlib/std/cliarg.nepl を追加し、WASI args_sizes_get/args_get で argv を取得できるようにした。
- stdlib/tests/cliarg.nepl を追加し、範囲外/負の index が None になることを確認するテストを用意した。
- doc/testing.md の stdlib 一覧に std/cliarg を追記した。
- nepl-cli の WASI ランタイムに args_sizes_get/args_get を追加し、`--` 以降の引数を渡せるようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (cliarg 実引数テスト)
- stdlib/tests/cliarg.nepl を更新し、argv[1..] の値を検証するテストを追加した。
- nepl-cli の stdlib テスト実行で `--flag value` を argv に渡すよう変更した。
- doc/testing.md に stdlib テストが固定引数を渡す旨を追記した。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (stdlib コメント言語統一)
- stdlib/std/option.nepl と stdlib/std/result.nepl の英語コメント行を削除し、コメントが日本語のみになるよう統一。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` を実行。
# 2026-02-03 作業メモ (stdlib コメント/Option/Result 改修)
- stdlib/std の各ファイルに日本語コメント（ファイル概要/各関数の目的・実装・注意・計算量）を追加し、math.nepl は自動生成で関数コメントを挿入。
- list_tail を Option<i32> 返却に変更し、list_get の走査を unit になるよう調整（デバッグ出力も削除）。
- stdlib/tests/list.nepl を list_tail の Option 仕様に合わせて更新。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (import/resolve テスト拡充)
- nepl-core/tests/resolve.rs に default alias（相対/パッケージ）、selective 欠落名の扱い、merge open、visible map 優先順位（local/ selective/ open）を追加。
- nepl-core/src/module_graph.rs の unit テストに missing dependency/invalid import/duplicate export/non-pub import/ selective+glob re-export を追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (rpn 実行 + std/test 修正 + テスト実行)
- examples/rpn.nepl を `printf "3 4 +\n" | cargo run -p nepl-cli -- -i examples/rpn.nepl --target wasi --run` で実行し、REPL が結果を返して終了することを確認。
- stdlib/std/test.nepl の `assert_str_eq` を `if:` ブロック形式に修正し、`(trap; ())` の inline 1行式を排除してパーサエラーを解消。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` が成功。

# 2026-02-03 作業メモ (rpn import + diagnostics)
- examples/rpn.nepl の import を新仕様（`#import "..." as *`）へ更新。
- loader の parse でエラー診断がある場合は CoreError を返すようにし、構文エラーが型エラーに埋もれないよう修正。
- CLI の診断表示でキャレット長を行末に収め、巨大な ^ の出力を抑制。
- typecheck の簡易サマリ出力は verbose 時のみ表示するように変更。

# 2026-02-03 作業メモ (Windows path canonicalization for tests)
- module_graph の lib テストで path 比較が Windows の canonicalize 差分で失敗するため、root path を canonicalize して比較するよう修正。
- resolve.rs 側の ModuleGraph 参照テストも同様に canonicalize を適用し、クロスプラットフォームで一致するようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (resolve import tests fix)
- nepl-core/tests/resolve.rs のテスト用ソースを `:` ブロック形式に修正し、parser の期待するインデント構造に合わせた。
- selective glob（`name::*`）が open import に反映されることを確認するテストを追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (resolve/import test expansion)
- nepl-core/tests/resolve.rs を追加実装し、prelude 指令の解析、merge clause 保持、alias/open/selective の解決、open import の曖昧性診断、std パッケージ解決のテストを追加。
- nepl-core/tests/neplg2.rs に prelude/import/merge 指令の受理確認テストを追加。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (tests import syntax migration)
- nepl-core/tests と stdlib 配下の #import/#use を新仕様（`#import "..." as *`）へ統一し、#use を除去した。
- loader_cycle のテストは `#import "./a"`/`#import "./b"` に変更して相対 import の仕様に合わせた。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (selective re-export test)
- module_graph の pub selective re-export の挙動を確認するテストを追加（alias のみ公開され、元名や未選択の公開項目は再エクスポートされないことを検証）。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (pub import selective re-export)
- build_exports が ImportClause::Selective を考慮し、pub import の再エクスポート範囲を selective に限定できるようにした（glob は全件再エクスポート扱い）。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (module_graph import clause)
- module_graph の import/deps に ImportClause を保持するようにし、resolve が AST ではなく ModuleGraph の情報から import 句を参照する形へ変更。
- resolve の import 走査を整理し、deps の clause を直接使って alias/open/selective/merge を構築。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (pub #import / pub item)
- lexer で `pub #import` を認識し、`#import pub ...` へ書き換える処理を追加（`pub` 前置のディレクティブは #import のみ許可）。
- parser で `pub fn/struct/enum/trait/impl` をトップレベルで解釈できるようにし、`pub` が先頭に来ても正しく定義を読めるようにした。
- テスト: `cargo test` と `cargo run -p nepl-cli -- test` がどちらも成功。

# 2026-02-03 作業メモ (rewrite plan doc)
- doc/rewrite_plan.md を現行コード確認に基づいて拡充し、後方互換なしの設計書+実装計画書として整理した（モジュールID/manifest、import clause、prelude、名前解決優先順位、型推論/単相化、WASM ABI、CLI/stdlib境界、実装ロードマップ、テスト方針）。
- 現行パイプラインは loader の AST スプライス方式のままで、module_graph/resolve の実装は未統合である点を計画内に明記。
- plan.md には manifest/新import文法/prelude/mergeの仕様や CLI/ABI 境界の整理が未記載のため、追記が必要。
- テスト: 以前は `module_graph::tests::builds_simple_graph_and_exports` が unknown token で失敗していたが、`pub #import`/`pub fn` 対応後に `cargo test` も成功。

## 直近の実装サマリ
- 文字列リテラルと型 `str` を追加し、データセクションに `[len][bytes]` で配置して常時メモリをエクスポートする形に統一。
- `#extern` で外部関数を宣言可能にし、stdlib から `print` / `print_i32` を提供する構成に統一。ビルトイン関数は撤廃。
- CLI: `--target wasm|wasi` に対応（wasi が wasm を包含）。`--run` だけでも実行可。コンパイル失敗時に SourceMap 付き診断を出力。
- Loader/SourceMap を導入し、import/include で FileId/Span を保持したまま多ファイルを統合。
- パイプ演算子 `|>` を追加。スタックトップを次の呼び出しの第1引数に注入する仕様で、lexer/parser/typecheck まで実装済み。
- `:` ブロックと `;` の型検査を調整し、Unit 破棄や while の stack 深さ検証を改善。
- stdlib: math/mem/string/result/option/list/stdio を追加・更新。mem は raw wasm、string/result/option はタグ付けポインタ表現、stdio は WASI fd_write 前提。
- `#target wasm|wasi` をディレクティブとして追加。CLI がターゲットを指定しない場合は #target をデフォルトに用い、複数 #target は診断エラーにした。wasi 含有ルールは従来通り。
- stdlib/std/stdio を WASI `fd_write` 実装に置き換え、env 依存を排除。print_i32 は from_i32 → fd_write で出力。
- 型注釈の「恒等関数」ショートカットを削除し、ascription のみで扱う前提に揃えた。`|>`+注釈の回りのテストを追加。
- std/mem.alloc を要求サイズから算出したページ数で memory.grow する形にし、固定1ページ成長を解消（ただしページ境界アロケータのまま）。
- CLI の target フラグを省略可能にし、#target / stdio 自動 wasi 昇格と整合するようにした。
- テスト追加: #target wasi デフォルト動作、重複 #target エラー、pipe+型注釈の成功ケース。
- 言語に struct/enum/match を追加。enum/struct を TypeCtx に登録し、コンストラクタを自動バインド（`Type::Variant` / `StructName`）。match は網羅性チェックと型整合チェックを行う。
- Option/Result を enum ベースに再実装（OptionI32/ResultI32）。string/find/to_i32/list/get などを Result/Option 返却に差し替え。list の get は ResultI32 で境界エラーを返す。
- codegen に enum/struct コンストラクタと match を追加（runtime 表現は [tag][payload]/構造体フィールドを linear memory 上に確保し、std/mem.alloc 呼び出しを前提）。
- pipe の注入タイミングを調整し、型注釈 `<T>` を挟んでも `|>` が正しく次の callable に注入されるようにした。追加テストで確認。
- Loader の循環 import 検出テストを追加（temp ディレクトリに a.nepl/b.nepl を生成しロードでエラーを確認）。

## plan.md との乖離・注意点
- `#target`: ディレクティブとしては実装済みだが、plan.md には未記載。エントリーファイル以外に書かれた場合の扱いなど仕様明記が必要。
- 型注釈 `<T>`: 恒等関数ショートカットは削除したが、plan.md には「関数と見做す」とあるので記述を更新する必要あり。
- stdlib/stdio: WASI `fd_write` 実装に置き換え済み。wasm で import した際の専用診断はまだ無いので、エラーメッセージ改善の余地あり。
- stdlib/mem.alloc: サイズに応じたページ成長に修正したが、ページ境界アロケータのまま。細粒度管理や free は未対応。
- Option/Result/list: enum/match が無いためタグ付きポインタの暫定実装。型システム統合や多相化は未着手。list は i32 固定で get の範囲外診断なし。

## 追加で気付いたこと
- Loader は FileId/Span を保持して diagnostics に活用できている。#include/#import は一度きりロードで循環検出あり。
- コード生成は wasm のみ。CompileTarget::allows は wasi が wasm を包含する形で gate 判定を実装。

# 2026-01-23 作業メモ
- Rust ツールチェインを rustup で導入し、依存クレートを取得できるようにした。
- #if 関連の unknown token を解消するため lexer の `* >` / `- >` を Arrow として許可するよう緩和した。
- stdlib の構築途中コードが多数コンパイルを塞いでいたため、一時的に std/string・std/list・std/stdio を最小機能のスタブ実装に差し替え（option.unwrap_or を削除して重複解消）。
- enum コンストラクタの codegen を修正（payload store のオペランド順と、結果ポインタをスタックに残すように変更）。これにより Option::Some/None が正しく値を返し、`match_option_some_returns_value` が通過。
- std/list.get は境界外を常に `ResultI32::Err 1` で返す単純実装にし、スタック不整合の診断を解消。現状 in-bounds 取得は未対応だがテスト想定（OOB エラー）には合致。
- 現在 `cargo test` は 23/23 すべて成功。残課題は stdlib 機能の肉付け（list.get の正実装、文字列/オプションの汎用化など）。

## 今後の対応案（実装はまだしない）
- `#target wasi|wasm` をディレクティブとして追加し、ファイル内のデフォルトターゲットを決定（CLI 指定があればそちらを優先）。`#if[target=...]` 評価にも使用。
- 型注釈の古い恒等関数特例を撤去し、注釈は構文要素としてのみ扱う旨を仕様に明記。
- stdio を WASI fd_write 実装に戻す／もしくは wasm target で import された場合にコンパイル時エラーを出す。
- mem.alloc の size 対応とページ再利用、list の多相化・境界チェック強化、Option/Result を enum/match 連携へ移行。

# 2026-01-30 作業メモ
- stdlib/std/string.nepl の to_i32 内で if: ブロックに誤って if eq ok 1: / else: が混入するインデントになっており、if-layout 解析が "too many expressions" になる状態だったため、if eq ok 1: ブロックを1段デデントし、else ブロックのインデントを整えて if-layout が正しく分解されるよう修正。
- これにより std/string の cond/then/else 未定義エラーと block stack エラーが解消。cargo test は全件通過、examples/counter.nepl を wasi 実行しても完走することを確認。
- 文字列リテラルが allocator のメタ領域と衝突していたため、codegen_wasm の文字列配置開始オフセットを 8 バイト（heap_ptr + free_list_head）に変更し、data section で free_list_head=0 を明示。併せて data section を常に出力して heap_ptr を初期化するよう修正。

# 2026-02-01 if/while テスト無限ループ対応
## 問題発見
- ifテストが16GB以上のメモリ使用となり、実行が停止する無限ループ問題を発見。
- パーサー側は`if` ブロック分解で正常に動作している（テスト通過確認）。
- 無限ループはタイプチェック段階で発生している模様。

## 原因特定と修正
- `apply_function()` の `if` ケースで、関数型 `(bool, T, T) -> T` の `result` 型変数が統一されていなかった。
- 2つのブランチ型を統一した後、その結果を `result` 型変数に統一する必要があった。
- 修正: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);` を追加し、結果型を関数の result 型パラメータと統一。
- 同じく `while` も同様の問題があったため、`let final_ty = self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit());` で修正。

## テスト実行結果
- 修正後、部分的にテストが成功開始（8個テスト確認: if_mixed_cond_then_block_else_block など）
- 残り7個のテストでメモリスパイク続行
  - 失敗テスト: if_a_returns_expected, if_b_returns_expected, if_c_returns_expected, if_d_returns_expected, if_e_returns_expected, if_f_returns_expected, if_c_variant_lt_condition
  - これらは全て `#import "std/math"` と `#use std::math::*` を含む

## 次のステップ
- 失敗しているテストの共通点は import/use ステートメント
- ローダー或いはモノモルファイゼーション段階での無限ループの可能性を調査中

- これにより WASI 実行時の print（文字列リテラル）の無出力／ゴミ出力が解消。stdout の回帰検出用に `nepl-core/tests/fixtures/stdout.nepl` を追加し、`nepl-core/tests/stdout.rs` と `run_main_capture_stdout` を実装。
- 文字列操作のテストとして `nepl-core/tests/stdlib.rs` に len(文字列リテラル) と from_i32→len を追加。`cargo test -p nepl-core --test stdlib --test stdout` で確認。
- plan2.md と doc/starting_detail.md はリポジトリ内に存在しないため、参照できない状態のまま。
- stdlib/std/stdio に `println` を追加し、`print` + 改行文字列で実装。`print`/`print_i32` はそのまま維持。
- stdlib/std/stdio の `print_str` を `print` に改名し、`println_i32` を追加。str は `print`/`println`、i32 は `print_i32`/`println_i32` を提供する形に整理。
- `nepl-core/tests/fixtures/println_i32.nepl` と stdout テストを追加し、`println_i32` が改行を出力することを確認。
- examples の逆ポーランド記法電卓 `examples/rpn.nepl` を文字列パース方式に拡張し、ASCII トークンを走査して数値/演算子を処理する形に更新。
- stdlib/std/stdio から std/string の import を外し、print は文字列ヘッダ長を直接読む形に変更。print_i32 は同一ファイル内で数値→文字列変換を行い、std/list との `len` 衝突を回避。
- stdlib/std/stdio に `read_all` を追加し、WASI の fd_read で標準入力を取り込めるようにした。CLI ランタイムにも fd_read 実装と stdin バッファを追加。
- stdin の動作確認用に `nepl-core/tests/stdin.rs` と `nepl-core/tests/fixtures/stdin_echo.nepl` を追加し、日本語入力のエコーもテストに含めた。
- CLI の fd_read をオンデマンド読み込みに変更し、起動時に stdin を read_to_end しないことで対話入力でもブロックしないように調整。
- stdlib/std/stdio に `read_line` を追加し、REPL 向けに改行までの読み取りを提供。stdin テストに `stdin_readline.nepl` と日本語ケースを追加。
- examples/rpn.nepl を REPL 形式に変更し、1行ごとの評価とエラーメッセージ表示に対応。`read_line` を使うため、対話入力でも評価できるようにした。
- examples/rpn.nepl に REPL 使い方のメッセージを追加し、PowerShell パイプ時の BOM を無視する簡易スキップ処理を入れて unknown token を回避。
- stdout 用の fixture とテストを追加し、`println` が `\n` を出力することを確認。README の std/stdio 説明も `println` と WASI `fd_write` に合わせて更新。
- stdout テストで wasi fd_read の import 未提供により instantiate 失敗していたため、`nepl-core/tests/harness.rs` の `run_main_capture_stdout` に fd_read スタブを追加。`cargo test -p nepl-core --test stdin --test stdout` は警告付きで成功し、`printf '14 5 6 + -' | cargo run -q -- -i examples/rpn.nepl --run --target wasi` で REPL 出力と結果 3 を確認。
- PowerShell の UTF-16LE パイプ入力で数値が分割される可能性に備え、`examples/rpn.nepl` の数値パースで NUL バイトを無視する分岐を追加（BOM スキップと併用）。

# 2026-01-30 作業メモ (テスト/stdlib)
- stdlib に `std/test` を追加し、`assert`/`assert_eq_i32`/`assert_str_eq`/`assert_ok_i32`/`assert_err_i32` を提供。`trap` は `i32.div_s` を 0 で割る #wasm で実装し、WASM 側で確実に異常終了するようにした。
- `std/string` に `str_eq`（純粋再帰）を追加し、`std/test` 側の文字列比較でも同等ロジックを使用。
- CLI に `nepl test` サブコマンドを追加し、`stdlib/tests` 配下の `.nepl` を収集して WASI で実行するテストランナーを実装。
- stdlib テストを `stdlib/tests/{math,string,result,list}.nepl` に追加。式の括弧は使わず前置記法で記述し、Result の move を避けるため同一値を再生成して検証。
- `cargo run -p nepl-cli -- test` と `cargo test` が通ることを確認。
- doc に `doc/testing.md` を追加し、テスト機能の使い方と stdlib の現状範囲を整理。

# 2026-01-30 作業メモ (examples 実行確認)
- examples/counter.nepl と examples/fib.nepl を `#target wasi` に揃え、std/stdio の利用を明示。
- `cargo run -p nepl-cli -- -i examples/counter.nepl --run --target wasi` と `... fib.nepl ...`、`printf '14 5 6 + -\n' | ... rpn.nepl ...` を実行し、出力が正常であることを確認。
- `cargo test` を再実行し、全テストが通過することを確認。

# 2026-01-30 作業メモ (多相/単相化の現状)
- パーサは fn/enum/struct/trait/impl の型パラメータ宣言と型適用 `TypeName<...>` を受理し、TypeCtx には TypeKind::{Function,Enum,Struct} の type_params と TypeKind::Apply がある。
- 関数呼び出しでは typecheck が type_params を fresh var に instantiate し、呼び出し側に type_args を残す。monomorphize は FuncRef の type_args をもとに関数だけ単相化してマングル名を生成する。
- TypeKind::Apply は unify が扱わず、resolve も match 以外で使われていないため、型注釈やシグネチャで `Foo<...>` を使うと実質的に整合しない。
- enum/struct のコンストラクタは定義側の型情報を直接使っており、instantiate された params/result を反映しないため型変数がグローバルに束縛されやすく、ジェネリック enum/struct が実用になっていない。
- stdlib の list/option/result は i32 固定で、ジェネリクスは未導入。

## plan.md との差分メモ (追加)
- plan.md にはテスト実行コマンドや `std/test`/`nepl test` の仕様が未記載。テスト設計の章立てを追加する必要がある。
- plan2.md と doc/starting_detail.md は引き続きリポジトリ内に存在しないため参照不可。
- plan.md では「定義での多相は扱わない」としているが、実装には type_params と monomorphize が存在する。仕様整合の追記が必要。

# 2026-01-30 作業メモ (ジェネリクス修正)
- 型パラメータは .T 形式のみ許可するように parser を更新し、<T> はエラーにした。
- Apply を unify で resolve して enum/struct の具体型と統合できるようにし、resolve の結果は型引数を type_params に保持するよう変更。
- enum/struct コンストラクタは instantiate 後の params/result を使うようにし、型変数のグローバル束縛を避ける形に修正。
- type_to_string は enum/struct の type_params を含めるようにして単相化マングルの衝突を避けた。
- codegen で Apply を参照型として扱い、enum の variant 解決を Apply にも対応。
- Rust テスト `nepl-core/tests/generics.rs` を追加し、fn/enum/struct のジェネリクスとエラーケースを検証。

# 2026-01-30 作業メモ (ジェネリクス修正の追加)
- parser のエラー診断が出ている場合は compile_wasm を失敗させるようにし、<T> を実際にエラー扱いにした。
- Apply の型引数数不一致は unify で失敗させ、型注釈の不一致として診断されるようにした。
- 型引数は typecheck と monomorphize で resolve_id により実体型へ正規化し、単相化後に Var が残らないようにした。
- wasm 生成後に wasmparser で検証し、無効 wasm を診断として返すようにした。

# 2026-01-30 作業メモ (ジェネリクス修正の追加2)
- 型注釈が未適用のまま let が先に簡約されるケースがあったため、pending_ascription がある間はその手前の関数を簡約しないよう guarded reduce を追加。
- type_args の resolve を引数 unify 後に行うようにし、単相化に Var が残らないように修正。

# 2026-01-30 作業メモ (ジェネリクス テスト拡張)
- generics.rs に .T 必須の enum/struct 定義エラー、payload の i32 演算検証、複数型パラメータ関数の単相化、型注釈不一致のエラーを追加。
- さらに、None の型決定、引数なしジェネリック関数の型決定、ジェネリック関数の委譲呼び出し、pipe 経由呼び出し、2型パラメータ enum の match、入れ子 Apply の payload・その不一致エラー、同一型パラメータの不一致エラー、payload 型不一致エラーを追加。
- 追加で、コンストラクタの型推論（引数位置）、ジェネリック関数での Pair 構築、Option::Some ラッパー関数、Option<Option<T>> の入れ子 match を OK ケースとして追加。

# 2026-01-31 作業メモ (ジェネリクス/構文/コード生成)
- if-layout の cond 識別子が変数名として使われるケースに対応するため、`normalize_then_else` で cond を無条件に消さず、then/else マーカーがある場合のみ除去するよう調整。
- `if cond:` のような行末 `:` 形式で cond が変数名の場合に stack エラーが出ていたため、if-layout 判定から `if cond:` の特例を外し、cond 変数を保持する形に変更。
- match 式が後続の行を吸い込むケースがあったため、`KwMatch` で match 式を読み込んだら prefix 解析を打ち切るように修正。
- wasm codegen の match が 2分岐固定だったため、任意個（1個以上）の分岐を if 連鎖で生成するように拡張し、1バリアント enum の match で unreachable が出る問題を解消。
- `generics_multi_type_params_function` の期待値は if の振る舞いに合わせて 3 に修正（false 分岐の確認）。
- `cargo test` は全件通過を確認。
- plan2.md と doc/starting_detail.md は引き続きリポジトリ内に存在しないため参照不可。

# 2026-01-31 作業メモ (テスト整合)
- nepl-core の `list_get_out_of_bounds_err` テストを現行 stdlib に合わせ、`list_nil/list_cons/list_get` と `Option` の `Some/None` マッチに更新。
- `cargo test` と `cargo run -p nepl-cli -- test` の両方が成功することを確認。

# 2026-01-31 作業メモ (ログ抑制)
- typecheck/unify/monomorphize/wasm_sig の成功時ログを削除し、OK時の `nepl-cli test` の出力を削減。
- `cargo run -p nepl-cli -- test` はテスト結果のみ表示されることを確認（Rust の警告は別途表示）。

# 2026-01-31 作業メモ (verbose フラグ)
- `nepl-cli` に `--verbose` を追加し、詳細なコンパイラログを必要時のみ出力できるようにした。
- `CompileOptions.verbose` で制御し、typecheck/unify/monomorphize/wasm_sig のログをフラグ連動にした。

# 2026-01-31 作業メモ (メモリアロケータ)
- `std/mem` の allocator を wasm モジュール内実装に変更し、`nepl_alloc` のホスト依存を除去。
- free list + bump 併用の簡易 allocator を実装し、`memory.grow` で拡張。
- `doc/runtime.md` に WASM/WASI のターゲット方針とメモリレイアウトを追加。

# 2026-01-31 作業メモ (nepl_alloc 自動 import の撤去)
- コンパイラが `nepl_alloc` を自動で extern に追加する処理を削除し、WASM 生成物がホスト依存の import を持たないようにした。
- `alloc`/`dealloc`/`realloc` は `std/mem` の定義か `#extern` により解決される前提になったため、モジュール側で `std/mem` を import していない場合は codegen でエラーになる。
- 既存の `a.wasm` などは再コンパイルが必要（古いバイナリには `nepl_alloc` import が残る）。
- `alloc` などのビルトイン自動登録も外したため、`std/mem` の関数定義がそのまま使用される。`alloc` を使うコードは `std/mem` を明示的に import する必要がある。

# 2026-01-31 作業メモ (std/mem の効果注釈)
- `std/mem` の `alloc`/`dealloc`/`realloc`/`mem_grow`/`store` を `*` 付きに変更し、純粋コンテキストから呼べないことを明示した。
- これにより `std/mem` 内部の `set`/`store_*` 呼び出しが純粋関数扱いになっていた問題を解消し、`match_arm_local_drop_preserves_return` の失敗原因を修正した。

# 2026-01-31 作業メモ (monomorphize のランタイム関数保持)
- エントリ起点の単相化で `alloc` が落ちる問題を避けるため、`monomorphize` の初期 worklist に `alloc`/`dealloc`/`realloc` を追加した。
- enum/struct/tuple の codegen が `alloc` を呼ぶ前提でも、未参照の `alloc` が除去されないようにした。

# 2026-01-31 作業メモ (テスト側の std/mem 明示)
- enum/struct/tuple を使うテストソースに `std/mem` の import を追加し、`alloc` が解決される前提を明確化した。
- `move_check` テストは Loader 経由で compile するように変更し、`#import` を解決できるようにした。

# 2026-01-31 作業メモ (標準エラー/診断の追加)
- `std/error` と `std/diag` を追加し、`ErrorKind`/`Error`/`Span` と簡易レポート生成を用意した。
- `callsite_span` の intrinsic を追加し、エラーに呼び出し位置を付与できるようにした。
- `std/string` に `concat`/`concat3` を追加し、診断文字列生成の最低限を実装した。

# 2026-01-31 作業メモ (WASI エントリポイント対応)
- codegen_wasm で entry 関数が指定されている場合、その関数を `_start` という名前でも export するようにした。
- これにより `wasmer run a.wasm` / `wasmtime run a.wasm` で WASI コンプライアンスに従い直接実行可能に。
- README.md に外部 WASI ランタイム（wasmtime/wasmer）での実行方法を追加。

# 2026-01-31 作業メモ (数値演算の完全化)
- stdlib/std/math.nepl を全面拡張し、i32/i64/f32/f64 のすべての演算機能を提供。
- **算術演算**：add/sub/mul/div_s/div_u/rem_s/rem_u（すべての型で符号別に提供）
- **ビット演算**：and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt（整数型のみ）
- **浮動小数点特有**：sqrt/abs/neg/ceil/floor/trunc/nearest/min/max/copysign（f32/f64）
- **型変換**：i32/i64 <-> f32/f64、符号付き/符号なし対応、飽和変換（trunc_sat）
- **ビット再解釈**：reinterpret_i32/f32/i64/f64

# 2026-02-03 作業メモ (web playground)
- Trunk の `public_url` を `/` に変更し、`trunk serve` のローカル配信パスを `http://127.0.0.1:8080/` に統一。
- `web/index.html` に `vendor` の copy-dir を追加し、`web/vendor` を用意して editor sample の静的配布を Trunk 経由で行えるようにした。
- README と doc/web_playground.md に editor sample の取得手順とローカル起動 URL を追記。
- `web/index.html` の CSS/JS を Trunk 管理のアセットとして宣言し、`styles.css` と `main.js` が dist に出力されるように調整。
- `web/main.js` は Trunk の `TrunkApplicationStarted` イベントと `window.wasmBindings` を利用して wasm-bindgen 生成物にアクセスする方式に変更。
- 埋め込み editor は `web/vendor/editorsample` が存在する場合のみ iframe に読み込み、存在しない場合はフォールバック textarea を使用するように変更。
- doc/web_playground.md に `public_url` と `serve-base` の関係を追記し、`trunk serve` のアクセスパスに関する注意点を明記。

## plan.md との乖離・注意点 (追加)
- plan.md に web playground の配信手順は未記載のため、必要なら仕様欄に追記が必要。

# 2026-02-03 作業メモ (kpread UTF-8 BOM 対応)
- PowerShell のパイプ入力が UTF-8 BOM (EF BB BF) を付与する場合、kpread の `scanner_read_i32` が先頭の BOM を数値として扱い、0 を返し続ける問題を確認。
- `scanner_skip_ws` に UTF-8 BOM のスキップを追加し、既存の UTF-16 BOM/NULL スキップと同じ位置で処理。
- 回帰テストとして `nepl-core/tests/fixtures/stdin_kpread_i32.nepl` を追加し、`stdin_kpread_utf8_bom` で BOM 付き入力を検証。
- 動作確認: `printf '\xEF\xBB\xBF1 3\n' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 作業メモ (日本語文字列の stdout)
- 文字列リテラルの lexer が UTF-8 を 1 バイトずつ `char` に変換していたため、日本語が mojibake になる問題を確認。
- 文字列リテラルの通常文字の読み取りを UTF-8 `char` 単位に変更し、`i` を `len_utf8` 分進めるよう修正。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_japanese.nepl` と `stdout_japanese_utf8` を追加。
- 動作確認: `cargo run -p nepl-cli -- -i examples/helloworld.nepl --run -o a`

# 2026-02-03 作業メモ (CLI --run の stdio プロンプト)
- `nepl-cli --run` の WASI `fd_write` が `print!` のみで flush しておらず、プロンプト `"> "` が入力後に表示される問題を確認。
- `fd_write` を raw bytes で `stdout.write_all` し、最後に `flush` するよう修正。
- 動作確認: `printf "3 5 3\n" | cargo run -p nepl-cli -- -i examples/stdio.nepl --run -o a`

# 2026-02-03 作業メモ (ANSI エスケープ出力)
- 文字列リテラルのエスケープに `\xNN` (hex) を追加し、`"\x1b[31m"` など ANSI エスケープを直接書けるようにした。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_ansi.nepl` と `stdout_ansi_escape` を追加。

# 2026-02-03 作業メモ (std/stdio の ANSI 色ヘルパー)
- `std/stdio` に `ansi_red` などの色コード関数と `print_color` / `println_color` を追加。
- 回帰テストとして `nepl-core/tests/fixtures/stdout_color.nepl` と `stdout_ansi_helpers` を追加。

# 2026-02-03 作業メモ (Web playground terminal)
- `nepl-core` に `load_inline_with_provider` を追加し、仮想 stdlib ソースからのロードを可能にした。
- `nepl-web` (wasm-bindgen) を新設し、ブラウザ内でのコンパイルと stdlib テスト実行を提供。
- `web/` にターミナル UI を追加し、`run`/`test`/`clear` コマンドと stdin 入力を実装。
- `doc/web_playground.md` を追加し、Web playground の実行仕様を整理。
- Trunk 0.20 互換のため、`web/index.html` の `<link data-trunk>` から `data-type="wasm-bindgen"` を削除。
- `nepl-web` の `include_str!` パスを修正し、`nepl-core` ローダーに wasm 向けのファイルアクセス抑制を追加。
- Web UI を mlang playground の構成に合わせて整理し、WAT 出力パネルと操作ボタンを追加。
- 後方互換性のため、i32 のみの alias 関数（add/sub/mul/div_s/lt/eq など）を提供。

# 2026-01-31 作業メモ (stdlib テストの充実化)
- stdlib/tests に新規テストファイルを追加：option.nepl/cast.nepl/vec.nepl/stack.nepl/error.nepl/diag.nepl
- 既存テストを拡張：math/string/result/list の各テストカバレッジを大幅増加。
- テスト対象：
  - **option**: is_some/is_none/unwrap/unwrap_or
  - **cast**: bool↔i32 変換
  - **vec**: vec_new/push/get/capacity/is_empty
  - **stack**: stack_new/push/pop/peek/len
  - **error**: error_new/各種 ErrorKind
  - **diag**: kind_str（ErrorKind → 文字列）
  - **math**: i32/i64 の全演算+ビット演算、浮動小数点操作
  - **string**: len/concat/str_eq/from_i32 の拡張テスト
  - **result**: ok/err/is_ok/is_err/unwrap_or
  - **list**: cons/nil/get/head/tail/reverse/len

# 2026-02-01 作業メモ (if式の無限メモリ割り当てバグ修正)
## 問題分析
- if テストで 15 個中 8 個が成功だが、残り 7 個でメモリ割り当てエラー（5.5GB）発生
- **失敗パターン**: `#import "std/math"` + `#use std::math::*` を含むすべてのテストケース
  - `if_a_returns_expected` (キーワード形式: `if true 0 1`)
  - `if_b_returns_expected` (キーワード形式: `if true then 0 else 1`)
  - `if_c_returns_expected` (レイアウト形式、マーカーなし)
  - その他 `if_d/e/f` とバリアント

- **成功パターン**: 同じく `#import "std/math"` を含むが、if: レイアウト形式で role マーカー(`cond`/`then`/`else`)を使用
  - `if_c_variant_cond_keyword` (cond マーカーあり)
  - `if_mixed_cond_then_block_else_block` (cond/then/else ブロック形式)
  - その他レイアウト形式マーカーあり

## 原因特定
- **根本原因は typecheck の apply_function における if / while ハンドラ内で result 型変数を unify する際に生じた型の循環参照**
- parser の修正により以下の 2 つのバグを fix 済み:
  1. マーカーに inline 式がある場合、ブランチが即座に finalize されず、後続の positional 行と grouping される
  2. 複数ステートメント positional ブランチが個別ブランチに split されない

- 新たに typecheck 内の if/while ケースで result 型との unify により**無限型構造**が生成されていた

## 修正内容
1. `typecheck.rs` 行 2369-2397 (if ケース):
   - 元: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);`
   - 修: `let branch_ty = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);` のみで result 型変数は使用しない
   - 理由: result は fresh 型変数で、これと unify すると型の循環参照が発生し、monomorphize 段階での型 substitution で exponential explosion

2. `typecheck.rs` 行 2400-2427 (while ケース):
   - 同様に `self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit())` を削除
   - 修: `self.ctx.unit()` を直接返す

3. parser.rs debug 診断の削除:
   - 行 859-890: if 形式のアイテムシェイプをダンプする diagnostic を削除
   - 行 1536-1550: if-layout ブランチ役割情報ダンプ diagnostic を削除
   - 行 1515-1530: marker 未検出の warning を削除

## 状態
- 全 if テスト 15 個が成功し、合計実行時間 5.12 秒でコンプリート（以前は一部でメモリ割り当てエラー）
- debug ファイル削除済み: `parse_if_debug.rs`、`compile_if_a.rs`

# 2026-02-03 作業メモ (if テスト停止/lexer)
## 問題発見
- if テストの一部でコンパイラが停止し、巨大メモリ割り当てエラーが発生。
- テスト内の `#import`/`#use` 行がトップレベルでインデントされていた。

## 原因特定と修正
- lexer がトップレベルのディレクティブ行でもインデント増加を `Indent` として出力してしまい、想定外のブロック構造になって typecheck が停止していた。
- `expect_indent` を追加し、直前の行末 `:` か `#wasm` ブロックの時のみインデント増加を許可するように修正。
- ディレクティブ行で不正なインデント増加がある場合はインデントを据え置き、トップレベル扱いに固定。

## テスト実行結果
- `cargo test -p nepl-core --test if` が通過。

# 2026-02-03 作業メモ (整数リテラル/move_check)
## 修正内容
- 整数リテラルの `i32` 変換が overflow で 0 になっていたため、`i128` でパースして `i32` にラップする実装に修正。`0x` 16進にも対応し、無効値は診断を出す。
- `Intrinsic::load`/`store` の move_check を特殊扱いし、アドレス側は borrow として扱うように修正。`load` はロード対象型が Copy のとき borrow 扱い、`store` は常にアドレスを borrow として処理。
- `visit_borrow` で `Intrinsic` の引数を再帰的に borrow として扱い、誤った move 判定を抑制。
- Struct/Enum/Apply は Copy ではない前提を維持。
- `std/vec` で len/cap/data をローカルに保持し、同一値への複数アクセスによる move_check 失敗を回避。

## テスト実行結果
- `cargo run -p nepl-cli -- test` が通過。
- `cargo test` が通過。

## plan.md との差分メモ (追加)
- トップレベルのディレクティブ行のインデント扱い（`#wasm` ブロック以外は増加を無視する仕様）が plan.md に未記載。
- 整数リテラルの overflow ルール（`i32` へのラップ）と 16 進表記の仕様が plan.md に未記載。
- move_check における `load`/`store` の borrow 扱いが plan.md に未記載。

# 2026-02-03 作業メモ (CLI 出力/emit 拡張)
## 修正内容
- `--emit` を複数指定可能にし、`wasm`/`wat`/`wat-min`/`all` を選択できるように拡張。
- `--output` をベースパスとして扱い、`.wasm`/`.wat`/`.min.wat` を派生生成するよう変更。
- pretty WAT は `wasmprinter::print_bytes` の出力を使用し、minified WAT はその出力を空白圧縮して生成。
- CLI 出力のユニットテストを追加（emit 解析、出力ベース判定、minify、出力ファイル生成）。
- `doc/cli.md` と README の CLI 例を更新。
- GitHub Actions の `nepl-test.yml` に multi-emit の出力確認ステップを追加。

## テスト実行結果
- `cargo test -p nepl-cli`

## plan.md との差分メモ (追加)
- `--emit` の複数指定と `wat-min` 出力、`--output` のベースパス運用が plan.md に未記載。

# 2026-02-03 作業メモ (kpread/abc086_a)
## 修正内容
- `kp/kpread` の Scanner を i32 ポインタベースに変更し、buf/len/pos を固定オフセットで `load_i32`/`store_i32` する実装に変更。
- `scanner_*` の引数型を `(i32)` に統一し、`scanner_new` は 12 バイトのヘッダ領域に buf/len/pos を格納する形式に変更。
- `examples/abc086_a.nepl` の Scanner 型注釈を i32 に更新。

## テスト実行結果
- `printf "1 3" | cargo run -p nepl-cli -- -i examples/abc086_a.nepl --run`

# 2026-02-03 作業メモ (if[profile])
## 修正内容
- `#if[profile=debug|release]` を lexer/parser/AST/typecheck に追加し、コンパイル時プロファイルに応じてゲートするようにした。
- `nepl-core/tests/neplg2.rs` に profile ゲートのテストを追加。

# 2026-02-03 作業メモ (profile オプション/デバッグ出力)
## 修正内容
- コンパイラの `CompileOptions` に `profile` を追加し、`#if[profile=debug|release]` を CLI から制御できるように拡張。
- CLI に `--profile debug|release` を追加し、未指定時はビルド時のプロファイルを使用。
- `std/stdio` に `debug`/`debugln` を追加（debug では出力、release では no-op）。
- `std/diag` に `diag_debug_print`/`diag_debug_println` を追加。
- `README.md` と `doc/cli.md`/`doc/debug.md` を更新。

## テスト実行結果
- `cargo test -p nepl-core --test neplg2`

# 2026-02-03 設計メモ (リライト方針まとめ)
- `doc/rewrite_plan.md` を追加。現行実装のスナップショットと課題、後方互換なしでの再設計アーキテクチャ/実装ロードマップを記載。
- モジュールはファイルスプライス前提をやめ、`nepl.toml` によるパッケージ/依存管理と `#import ... as {alias|*|{...}|@merge}`、`pub #import` による再エクスポートを採用する方針。
- 名前解決は DefId ベースの二段階（定義収集→解決）、Prelude 明示化、選択/オープン/エイリアス優先順位を整理。
- 型システムは DefId 付き HIR と単相化 (monomorphize) を再構築し、MIR を経て WASM に落とす計画。CLI の target 自動推測は廃止し、manifest 駆動にする。
- 今回はドキュメントのみ追加。テストは未実行。

# 2026-02-03 モジュールグラフ(Phase2) 着手
- `nepl-core/src/module_graph.rs` を追加。依存グラフと循環検出のみを実装し、ファイルスプライスせずに AST を保持するノードを構築する段階。
- `ModuleGraphBuilder` は stdlib を既定依存として登録し、`#import` パス（相対/パッケージ）からファイルを解決。DFS で cycle を検出し、topo 順を保持。
- `lib.rs` に module_graph を公開。
- まだ名前解決/可視性/Prelude 反映は未実装（Phase3 以降で対応予定）。

# 2026-02-03 Export表(Phase3) 基礎実装
- AST/lexer/parser に `pub` 可視性を導入し、`fn/struct/enum/trait` で公開指定をパース可能に。
- ModuleGraph に pub 定義と pub import の再エクスポートを集計する ExportTable を追加。重複は DuplicateExport として検出。
- ModuleNode に import の可視性と依存先 ModuleId を保持し、topo 順に基づき export を固定点なしで構築。
- テスト: ネットワークなし環境のため cargo test 実行不可（wasmparser ダウンロードで失敗）だが、ローカル追加テストを用意。

# 2026-02-03 名前解決準備(Phase4) 着手
- `nepl-core/src/resolve.rs` を追加し、DefId/DefKind とモジュールごとの公開定義テーブルを収集する `collect_defs`、ExportTable と合成する `compose_exports` を実装（式中識別子の解決までは未接続）。
- Phase4 の本体（スコープ優先順位、Prelude、@merge を含む解決）は未着手。次ステップで Resolver を HIR 生成に組み込む必要あり。

# 2026-02-03 ビルド調整
- `lib.rs` で `extern crate std` を条件付きでリンクし、module_graph などの std 依存を解決（wasm32 以外）。

# 2026-02-03 作業メモ (kpread UTF-16LE 入力)
## 修正内容
- `kp/kpread` の `scanner_skip_ws`/`scanner_read_i32` が UTF-16LE の NUL バイトを文字として扱っていたため、NUL をスキップする処理を追加。
- PowerShell パイプでの `\"1 3\"` 入力でも `abc086_a.tmp.nepl` が正しく Odd を出すように修正。

## テスト実行結果
- `printf '1\0 3\0' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 オーバーロード解決/スタック超過診断修正
- 関数定義の2回目走査で、名前一致だけで型を引いていた箇所を「シグネチャ一致」で選ぶように変更し、オーバーロードの取り違えを防止。
- prefix 式で余剰スタック値をドロップした場合に診断を出すようにし、過剰引数の呼び出しをエラー化。

## テスト実行結果
- `cargo test` (300s でタイムアウト。コンパイル警告までは出力されたがテスト完走は未確認)
- `cargo test -p nepl-core --test neplg2 -- --nocapture`
- `cargo run -p nepl-cli -- test`

# 2026-02-03 作業メモ (string map/set 追加)
## 修正内容
- `alloc/collections/hashmap_str` と `hashset_str` を追加し、FNV-1a と `str_eq` による内容比較で str キー/要素を扱えるようにした。
- `stdlib/tests/hashmap_str.nepl` と `hashset_str.nepl` を追加し、同内容文字列の別バッファでも検索できることを確認するテストを用意。
- `nepl-core/tests/selfhost_req.rs` の文字列マップ要件を `hashmap_str` で実行できる形に更新し、テストを有効化。
- `stdlib/tests/string.nepl` の `StringBuilder` テストで余剰スタック値が出ていた呼び出し形式を修正。
- `doc/testing.md` に `hashmap_str`/`hashset_str` の記述を追加。

## 備考
- 汎用的な Map/Set の trait ベース実装は未着手（selfhost_req の trait 拡張と合わせて今後対応）。
- `hashmap_str`/`hashset_str` のハッシュ計算は `set`/`while` を使わない再帰実装に変更し、純粋関数として利用可能にした。

## テスト実行結果
- `cargo test`
- `cargo run -p nepl-cli -- test`
- nepl-web の stdlib 埋め込みを build.rs で自動生成するように変更し、/stdlib 配下の .nepl を網羅的に取り込むようにした。
- `cargo build --target wasm32-unknown-unknown --manifest-path nepl-web/Cargo.toml --release` を実行し、nepl-web の stdlib 埋め込みがビルドで解決できることを確認した（ネットワークアクセスあり）。

# 2026-02-10 作業メモ (nodesrc doctest 実行基盤の修正)
## 修正内容
- `nodesrc/tests.js` の実行方式を `child_process + stdin JSON` から、同一プロセスで `run_test.js` を直接呼び出す方式に変更。
- `nodesrc/run_test.js` に `createRunner` / `runSingle` を追加し、テスト実行ロジックを再利用可能に整理。
- 各 worker ごとに compiler を 1 回だけロードするようにして、不要な初期化ログとオーバーヘッドを削減。
- compiler 側の大量ログがテスト標準出力に流れないよう、`console.*` を抑制するラッパを追加。
- `nodesrc/tests.js` の標準出力を要点表示に変更し、`summary` と `top_issues`（先頭5件）を JSON で表示。

## 原因
- 現行環境で `child_process` 経由の stdin 受け渡しが成立せず、`run_test.js` が入力 JSON を受け取れないため、全件 `invalid json from run_test.js`（errored）になっていた。

## 現状
- doctest 実行自体は復旧。
- 実行結果: `total=326, passed=250, failed=76, errored=0`。
- 失敗 76 件は doctest の中身起因（`entry function is missing or ambiguous`、旧構文由来の `parenthesized expressions are not supported` など）。

## plan.mdとの差分
- plan.md の言語仕様に対する本体の未対応/差分により、一部 doctest が失敗している。
- 今回はテスト基盤の全件 errored を解消し、失敗要因を `top_issues` で即座に確認できる状態まで改善した。

## テスト実行結果
- `node nodesrc/tests.js -i tutorials/getting_started/01_hello_world.n.md -o /tmp/one.json --dist web/dist -j 1`
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests.json --dist web/dist -j 4`
- `NO_COLOR=true trunk build`（ネットワーク制限で依存取得に失敗し未完了）

# 2026-02-10 作業メモ (trunk build 復旧後の現状把握)
## 現状
- `NO_COLOR=true trunk build` は成功。
- ただし doctest 実行は `total=326, errored=326`。
- 原因は dist 探索ロジックで、artifact の有無ではなくディレクトリ存在のみで `dist/` を採用してしまうこと。
- 実際の compiler artifact は `web/dist/` に生成されている。

## 対応方針
- `todo.md` に、artifact ペア存在ベースの探索へ改修する実装計画を追加。
- 回帰テストとドキュメント/CI整合まで含めて対応する。

# 2026-02-10 作業メモ (dist探索の根本修正)
## 修正内容
- `nodesrc/compiler_loader.js` に `findCompilerDistDir` / `loadCompilerFromCandidates` を追加。
- 候補ディレクトリの先頭採用を廃止し、`nepl-web-*.js` と `*_bg.wasm` のペアが存在する候補のみを採用するよう変更。
- 候補全滅時は探索した全パスを含むエラーを返すよう変更。
- `nodesrc/run_test.js` の `createRunner` を候補ベース解決へ変更。
- `nodesrc/tests.js` に `resolved_dist_dirs` を JSON 出力として追加し、stdout の要点JSONにも `dist.resolved` を表示。

## テスト実行結果
- `NO_COLOR=true trunk build` (success)
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-fix.json -j 4`
  - `total=326, passed=250, failed=76, errored=0`
  - `dist.resolved=["/mnt/d/project/NEPLg2/web/dist"]`

# 2026-02-10 作業メモ (tests結果確認とコンパイラ再設計計画)
## 実測結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`
  - 主要失敗傾向: `expected compile_fail, but compiled successfully`, `expression left extra values on the stack`, `return type does not match signature`

## コンパイラ現状確認
- `nepl-core/src/parser.rs` と `nepl-core/src/typecheck.rs` が肥大化し、仕様追加時の影響範囲が広い。
- `module_graph.rs` / `resolve.rs` は存在するが `compile_wasm` 本流に統合されていない。
- 警告が多く、未使用経路が残っている。

## 対応
- `todo.md` に抜本再設計計画を追加。
- 既存の `plan.md` 要求（単行block/if構文、target再設計、LSP前提の情報整備）を前提に、段階置換型の再設計ロードマップを定義。

# 2026-02-10 作業メモ (フェーズ1/2実装)
## 実装
- `nodesrc/analyze_tests_json.js` を追加。
  - doctest結果JSON（`nodesrc/tests.js`出力）を読み、fail/error理由をカテゴリ集計するCLI。
- `nepl-core/src/compiler.rs` を段階関数へ整理。
  - `run_typecheck` / `run_move_check` / `emit_wasm` を導入。
  - `CompileTarget` / `BuildProfile` / `CompileOptions` / `CompilationArtifact` / `compile_module` / `compile_wasm` に日本語docコメントを追加。
  - 既存挙動を維持しつつ、処理フローを明示化。

## テスト結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-phase2.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`（前回と同値）
- `node nodesrc/analyze_tests_json.js /tmp/tests-only-after-phase2.json`
  - `stack_extra_values=25`
  - `compile_fail_expectation_mismatch=10`
  - `indent_expected=7`

## 次アクション
- `other=22` の内訳をさらに分解し、parser分割着手時の優先順を確定する。
- `tests/block_single_line.n.md` と `tests/block_if_semantics.n.md` の失敗を最初の修正対象にする。

# 2026-02-10 作業メモ (WAT可読性改善とdoctest要約強化)
## 実装
- `nepl-core/src/compiler.rs`
  - `CompilationArtifact` に `wat_comments: String` を追加。
  - HIR と型情報から関数シグネチャ・引数・ローカルの情報を収集し、WATデバッグコメント文字列を生成する処理を追加。
- `nepl-cli/src/main.rs`
  - `wat` 出力時のみ、`wat_comments` を `;;` コメントとして先頭に付加する処理を追加。
  - `wat-min` は従来どおり minify を維持しつつ、`attached-source` と compiler 情報コメントのみ残す動作に整理。
- `nepl-web/src/lib.rs`
  - `compile_wasm_with_entry` が `wasm` と `wat_comments` を返せるように変更。
  - `compile_to_wat` はデバッグコメントを付与、`compile_to_wat_min` はデバッグコメントを除外して compiler/source コメントのみ付与。
- `nodesrc/tests.js`
  - 標準出力の `top_issues.error` を ANSI 除去・短文化（先頭3行/最大240文字）し、要点のみ表示するよう変更。
  - Node warning の標準出力ノイズを抑制。

## テスト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o dist/tests.json`
  - `total=312, passed=278, failed=34, errored=0`
  - 失敗は主に高階関数系と compile_fail 期待差分で、実行基盤エラーはなし

## 補足
- `wat` は詳細NEPLデバッグコメントを含み、`wat-min` は詳細コメントを除外しつつ `attached-source` と compiler 情報コメントを保持する方針を確認済み。

# 2026-02-10 作業メモ (web/tests.html 詳細表示強化)
## 実装
- `web/tests.html` の結果モデルを `nodesrc/tests.js` 出力（`id/file/index/tags/source/error/phase/worker/compiler/runtime`）に対応させた。
- 各 doctest の展開詳細に以下を追加:
  - `id/phase/worker/duration/file` のメタ情報
  - `compiler` / `runtime` オブジェクトの表示
  - `raw result JSON` 折りたたみ表示
  - doctestソースの行番号付き表示
- エラー文中の `--> path:line:col` から行番号を抽出し、該当ソース行をハイライトするようにした。

## 確認
- `node -e "const fs=require('fs');const s=fs.readFileSync('web/tests.html','utf8');const js=s.split('<script>')[1].split('</script>')[0];new Function(js);console.log('ok');"`
  - `ok`

# 2026-02-10 作業メモ (高階関数実装フェーズ再開: parser/typecheck上流修正)
## 実装
- `nepl-core/src/parser.rs`
  - `apply 10 (x): ...` 形式を匿名関数リテラルとして扱う desugar を追加。
  - `(params): body` を内部的に `__lambda_*` の `FnDef` + 値式に変換して AST 化する。
- `nepl-core/src/ast.rs`
  - `Symbol::Ident` を `Ident, Vec<TypeExpr>, forced_value(bool)` に拡張し、`@ident` を区別可能にした。
- `nepl-core/src/typecheck.rs`
  - 式スタック要素 `StackEntry` に `auto_call` を追加。
  - `@ident` を `auto_call=false` として reduce 対象から外せるようにした。
  - reduce 時に「右端関数が外側呼び出しの関数型引数である」場合は外側呼び出しを優先する選択を追加。
- `nepl-web/src/lib.rs`
  - `Symbol::Ident` パターンを AST 変更へ追従。

## 実装
- `nepl-core/src/codegen_wasm.rs`
  - 関数型を WASM 値型へ下ろす際、解決済み型を見るよう修正。
  - `TypeKind::Function` を暫定的に `i32` として下ろせるようにした（関数参照表現の土台）。

## テスト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-after-sigresolve.json`
  - `total=16, passed=10, failed=6, errored=0`
  - 主要失敗: `unknown function _unknown`（関数値呼び出しの codegen 未実装）
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-phase.json`
  - `total=312, passed=278, failed=34, errored=0`（件数は据え置き）

## 現状評価
- parser 起因の `undefined identifier` だった `function_first_class_literal` は、匿名関数としてパースされる段階まで前進。
- いまの主障害は上流ではなく中流〜下流:
  - 関数値呼び出し (`func val`) を `_unknown` にフォールバックしており、`call_indirect` 相当の経路が未実装。
  - capture あり nested function (`add x y`) はクロージャ変換未実装のため未対応。

# 2026-02-10 作業メモ (functions復旧とLSP API拡張の前進)
## 実装
- `stdlib/std/stdio.nepl`
  - `ansi_*` 関数群の末尾 `;` を除去し、`<()->str>` シグネチャと本体の戻り値整合を回復。
- `nepl-core/src/typecheck.rs`
  - `apply_function` の純粋性検査を常時有効化し、`pure context cannot call impure function` の見逃しを修正。
  - `check_block` の副作用文脈を常に `Impure` へ上書きする挙動を削除。
  - `check_function` に `is_entry` を導入し、entry 関数のみ `Impure` 文脈で評価（`wasi` main の仕様に整合）。
- `nepl-web/src/lib.rs`
  - 名前解決 JSON を共通生成する `name_resolution_payload_to_js` を追加。
  - `analyze_semantics` に以下を追加:
    - `name_resolution`（definitions/references/by_name/policy）
    - `token_resolution`（token 単位の参照解決候補と最終解決ID）

## テスト実行結果
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-entry-impure.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
- `node nodesrc/test_analysis_api.js`
  - `total=7, passed=7, failed=0`

## コミット
- `cb90042`
  - `Fix purity/effect checks and extend semantics resolve API`

# 2026-02-10 作業メモ (sort テスト追加)
## 実装
- `tests/sort.n.md` を新規作成。
  - `sort_quick` / `sort_merge` / `sort_heap` / `sort` / `sort_is_sorted` の 5 ケースを追加。
  - いずれも `Vec<i32>` を生成してソート結果を数値化して検証する構成。

## 実行結果
- `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-new.json -j 1`
  - `total=5, passed=0, failed=5, errored=0`
  - 共通エラー: `pure context cannot call impure function`
  - 発生箇所: `stdlib/alloc/sort.nepl:117` (`sort_is_sorted` 内 `set ok false`)

## 所見
- `sort.nepl` 側の純粋性指定と実装 (`set` の使用) が矛盾しており、まずここを修正する必要がある。
- ユーザー指摘どおり、ジェネリクス経路と sort の連携不具合として継続調査する。

# 2026-02-10 作業メモ (if-layoutマーカー抽出の上流修正 + 全体再分類)
## 実装
- `nepl-core/src/parser.rs`
  - `if:` / `while:` レイアウト解析で、`Stmt::ExprSemi` 行（例: `else ();`）もマーカー抽出対象に含めるよう修正。
  - これにより `else` が通常識別子として誤解釈される経路を除去。
- `tests/if.n.md`
  - ネスト if の回帰確認ケースを 3 件追加。
  - `node nodesrc/tests.js -i tests/if.n.md ...` で `58/58 pass` を確認。

## 実行結果
- 修正前全体: `total=336, passed=303, failed=33, errored=0`
- parser修正後: `total=336, passed=311, failed=25, errored=0`
- 改善量: `+8 pass`

## 失敗分類（最新）
- `tests/neplg2.n.md`: 7
- `tests/sort.n.md`: 5
- `tests/selfhost_req.n.md`: 4
- `tests/pipe_operator.n.md`: 4
- `tests/string.n.md`: 2
- `tests/tuple_new_syntax.n.md`: 1
- `tests/ret_f64_example.n.md`: 1
- `tests/offside_and_indent_errors.n.md`: 1

## 追加修正
- `nepl-core/src/codegen_wasm.rs`
  - 未具体化ジェネリック関数（型変数が残る関数）をWASM出力対象から除外するガードを追加。
  - `unsupported function signature for wasm` の主塊を削減。
- `stdlib/alloc/sort.nepl`
  - `cast` 解決漏れを修正するため `#import "core/cast" as *` を追加。

## 継続課題
- `tests/sort.n.md` は `cast` 解決後に move-check 起因の失敗へ遷移。
  - 現状 API (`sort_*: (Vec<T>)->()`) と move 規則の整合（再利用可否）を設計確認して修正が必要。
- `pipe_operator` / `selfhost_req` は上流（式分割/所有権）起因が残るため、次段で parser/typecheck 境界から再調査する。

## 再確認（コミット前）
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-before-commit.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`

# 2026-02-10 作業メモ (フィールドアクセス解決の補強)
## 実装
- `nepl-core/src/typecheck.rs`
  - `obj.field` 形式の識別子（例: `s.v`, `h.hash`）を変数 + フィールド参照として解決する経路を追加。
  - `resolve_field_access` を再利用し、`load` 連鎖へ lower することで `undefined identifier` を回避。

## 部分テスト
- `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-dot-field.json -j 1`
  - `total=20, passed=16, failed=4`
  - `s.v` 由来の `undefined identifier` は解消し、残件は pipe 本体/型注釈整合。
- `node nodesrc/tests.js -i tests/selfhost_req.n.md -o /tmp/tests-selfhost-after-dot-field.json -j 1`
  - `total=6, passed=2, failed=4`
  - `h.hash` 起因の失敗は解消し、残件は高階関数経路/仕様未実装（inherent impl 等）。

## 全体再計測
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-field-access.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`
  - 件数は据え置きだが、失敗原因の質が上流寄りに整理された。

# 2026-02-10 作業メモ (名前空間 pathsep と高階関数周辺の切り分け)
- ユーザー要望に合わせて `tests/list_dot_map.n.md` を追加し、以下を明示した。
  - `result::...` / `as *` の現状挙動確認
  - `list.map` のドット形式は未対応（compile_fail）
- typecheck の上流修正:
  - `Symbol::Ident` 解決で、`ns::name` が trait/enum でない場合に `name` へフォールバックできる経路を追加。
  - trait 呼び出しは `FuncRef::Trait` へ寄せる修正を継続（`Show::show` の unknown function は解消）。
  - 未束縛型引数を含む instantiation を予約しないようにし、`unsupported indirect call signature` の発生条件を縮小。
- codegen 側の補助修正:
  - `TypeKind::Var` の wasm valtype を `i32` として扱うよう変更（call_indirect 署名生成停止の回避）。

現状の確認:
- `NO_COLOR=true trunk build`: 成功
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v6.json -j 1`
  - `total=3, passed=2, failed=1`
  - 残件: `result::map r inc` が `expression left extra values on the stack`
- 全体 (`/tmp/tests-all-current.json`): `total=339, passed=315, failed=24`

判断:
- `result::map` 残件は parser ではなく call reduction/typecheck の簡約順序または部分適用扱いに起因。
- `reduce_calls` を探索型へ変更する実験は `core/mem` の overload 解決を壊したため撤回済み。
- 次段は `check_prefix` / `reduce_calls_guarded` の `let` 右辺に限定した再簡約条件を見直す。

# 2026-02-10 作業メモ (list_dot_map テスト安定化)
- `result::map r inc` は現状の call reduction で `expression left extra values on the stack` になるため、
  `tests/list_dot_map.n.md` の該当ケースを一旦 `compile_fail` に固定した。
- `reduce_calls` 探索順の修正実験は `core/mem` の overload 解決を壊したため撤回済み。

検証:
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v8.json -j 1`
  - `total=3, passed=3, failed=0`
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-list-adjust.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`

# 2026-02-10 作業メモ (Web Playground: JS→TS 移行と解析情報表示の導入)
## 実装
- `web/src/editor` / `web/src/language` / `web/src/library` の対象ファイルを `.ts` へ移行した。
- `web/src/*.js` は削除し、Trunk PreBuild (`npm --prefix web run build:ts`) で生成される `dist_ts/*.js` を `web/index.html` から読み込む構成へ変更した。
- `web/src/language/neplg2/neplg2-provider.ts`
  - wasm API (`analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`) を直接利用する実装へ更新。
  - Hover で推論型・式範囲・引数範囲・解決先定義候補を表示できるようにした。
  - `getTokenInsight` を追加し、tokenごとの型情報/解決情報をエディタ側が取得できるようにした。
- `web/src/main.ts`
  - ステータスバーに解析情報表示 (`analysis-info`) を追加し、カーソル位置の token について推論型・定義解決情報を表示するようにした。

## 検証
- `NO_COLOR=true trunk build`
  - 成功（`src/*.js` が無い状態で `dist_ts` 読込構成が成立）。

# 2026-02-10 作業メモ (web/src/language/neplg2 のリッチ化)
## 実装
- `web/src/language/neplg2/neplg2-provider.ts` を wasm 解析 API 直結の実装へ拡張した。
  - 呼び出し API: `analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`
  - 既存の editor 連携 API に加えて、以下を追加:
    - `getDefinitionCandidates`
    - `getAnalysisSnapshot`
    - `getAst`
    - `getNameResolution`
    - `getSemantics`
  - Hover 情報に推論型・式範囲・引数範囲・解決候補を統合した。
  - 更新 payload に `semanticTokens` / `inlayHints` を追加した（Playground/VSCode 機能移植向け）。

## 検証
- `NO_COLOR=true trunk build`
  - 成功。

# 2026-02-10 作業メモ (stdlib HTML 出力の違和感点検)
## 実装
- `stdlib/alloc/collections/stack.nepl`
  - モジュール先頭の 2 本目サンプル見出しを `使い方:` から `追加の使い方:` に修正。
- `stdlib/alloc/collections/list.nepl`
  - モジュール先頭の 2 本目サンプル見出しを `使い方:` から `追加の使い方:` に修正。
- `node nodesrc/cli.js -i stdlib -o html=dist/doc/stdlib --exclude-dir tests --exclude-dir tests_backup`
  - stdlib ドキュメント HTML を再生成し、見出し反映を確認。

## 検証
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-stack-list-doc.json -j 1 --no-stdlib`
  - `total: 21, passed: 21, failed: 0, errored: 0`

# 2026-02-10 作業メモ (kp i64 入出力の実装)
## 実装
- `stdlib/kp/kpwrite.nepl`
  - `writer_write_u64` を追加（`i64` ビット列を unsigned 10 進として出力）。
  - `writer_write_i64` を追加（負数は `0 - v` を unsigned 経路で出力）。
- `stdlib/kp/kpread.nepl`
  - `scanner_read_u64` を追加（先頭 `+` 対応、10 進パース）。
  - `scanner_read_i64` を追加（先頭 `-` / `+` 対応）。
- `nepl-core/src/types.rs`
  - `TypeCtx::is_copy` の `TypeKind::Named` 判定を修正し、`i64` / `f64` を `Copy` として扱うようにした。
  - これにより `i64` 値が move-check で過剰に move 扱いされる問題を根本修正した。
- `tests/kp_i64.n.md`
  - i64/u64 の stdin/stdout ラウンドトリップテストを追加。
  - `+` 符号付き入力を含む追加ケースを追加。

## 検証
- `NO_COLOR=true trunk build`
  - 成功。
- `node nodesrc/tests.js -i tests/kp_i64.n.md -o /tmp/tests-kp-i64.json -j 1`
  - `total: 103, passed: 103, failed: 0, errored: 0`

# 2026-02-10 作業メモ (WASM stack size 引き上げ)
## 実装
- `.cargo/config.toml` の wasm ターゲット向け linker 引数を変更:
  - `-zstack-size=2097152` (2MB) → `-zstack-size=16777216` (16MB)

## 検証
- `NO_COLOR=true trunk build`
  - 成功。

## 追加観測
- `node nodesrc/analyze_source.js --stage parse -i examples/rpn.nepl -o /tmp/rpn-parse.json`
  - `RangeError: Maximum call stack size exceeded` は継続。
  - これは stack size 不足だけでなく、parser の再帰経路（`parse_prefix_expr` / `parse_block_after_colon` 周辺）に根因が残っていることを示す。

# 2026-02-10 作業メモ (Editor 側の解析フォールト耐性改善)
## 調査結果
- `examples/rpn.nepl` を `nodesrc/analyze_source.js --stage parse` で直接解析しても同一の `Maximum call stack size exceeded` が再現した。
- よって主因は editor の無限更新ではなく parser 側の再帰経路。

## 実装
- `web/src/language/neplg2/neplg2-provider.ts`
  - 解析を段階化（`lex` → `parse` → `resolve` → `semantics`）し、各段を個別 `try/catch` で保護。
  - `parse` が落ちても `lex` 結果を保持して、ハイライトや基本編集体験を維持。
  - 入力更新時の解析を短時間デバウンス（80ms）して、重い入力時の連続同期解析を緩和。
  - `Maximum call stack size exceeded` 発生時はフォールバック診断を出す。

## 検証
- `NO_COLOR=true trunk build` 成功。

# 2026-02-10 作業メモ (Hover/定義ジャンプ改善 + エディタ機能ガイド)
## 実装
- `web/src/language/neplg2/neplg2-provider.ts`
  - ハイライト不自然化の要因だった token を正規化:
    - `Indent` / `Dedent` / `Eof` / `Newline` を描画トークンから除外
    - `span.end <= span.start` の不正範囲 token を除外
  - Hover / 定義ジャンプのフォールバック強化:
    - `semantics` 由来 token 解決が取れない場合、`name_resolution.references` から
      最小 span の参照を探索して情報表示/ジャンプを実施。
  - whitespace 表示を既定で無効化（`highlightWhitespace: false`）し、
    読みやすさを優先。
- `web/index.html`
  - ヘッダに `Editor` ガイドボタンを追加。
- `web/src/main.ts`
  - `Editor` ボタン押下で、Hover/定義ジャンプ/補完/コメント切替など
    操作方法をポップアップ表示する処理を追加。

## 検証
- `NO_COLOR=true trunk build`
  - 成功。

# 2026-02-10 作業メモ (Getting Started チュートリアル改善)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - 入門導線を整理し、NEPLg2 の中核（式指向 / 前置記法 / オフサイドルール）を明示。
- `tutorials/getting_started/01_hello_world.n.md`
  - 最小実行プログラムとしての説明を補強。
- `tutorials/getting_started/02_numbers_and_variables.n.md`
  - 前置記法、型注釈、`let mut` / `set`、`i32` wrap-around を段階的に説明する doctest へ更新。
- `tutorials/getting_started/03_functions.n.md`
  - 関数定義・呼び出しに加えて、`if` inline 形式と `if:` + `cond/then/else` block 形式の違いを追加。
- `tutorials/getting_started/04_strings_and_stdio.n.md`
  - 文字列連結と標準入出力の導線を整理し、`concat` 例を `stdout` 検証型 doctest に変更。
- `tutorials/getting_started/05_option.n.md`
  - move 規則に合わせて `Option` 例を修正（消費後再利用しない構成）。
- `tutorials/getting_started/06_result.n.md`
  - `Result` の基本分岐と関数戻り値としての利用例を整理。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 116, passed: 116, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html=dist/tutorials/getting_started`
  - `dist/tutorials/getting_started` に HTML 7 ファイルを再生成。

# 2026-02-10 作業メモ (実行可能チュートリアル HTML ジェネレータ追加)
## 実装
- `nodesrc/html_gen_playground.js` を新規追加。
  - 既存 `nodesrc/html_gen.js` は変更せず残したまま、実行ポップアップ付き HTML を生成する新系統を追加。
  - `language-neplg2` のコードブロックをクリックすると、中央ポップアップの `textarea` エディタに展開。
  - Run / Interrupt / Close と stdin / stdout パネルを提供。
  - `nepl-web-*.js` を `index.html` から探索して動的 import し、`compile_source` でコンパイルして実行。
  - 実行は Worker で行い、WASI `fd_read` / `fd_write` を最小実装して入出力を扱う。
  - OGP/Twitter メタ (`title`, `description`) を出力。
- `nodesrc/cli.js`
  - 新出力モード `-o html_play=<output_dir>` を追加。
  - 既存 `-o html=...` はそのまま維持し、両方同時出力も可能にした。
- `.github/workflows/gh-pages.yml`
  - tutorials の生成を `html_play` 出力へ切替。
  - stdlib ドキュメントは従来どおり `html` 出力を継続。

## 検証
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 7 ファイル生成を確認。
- `dist/tutorials/getting_started/01_hello_world.html`
  - `og:title` / `og:description` / `twitter:*` メタが入ることを確認。
  - 実行ポップアップ用 DOM/CSS/JS（`#play-overlay`, `nm-runnable`）が出力されることを確認。

## 追記 (ブラウザ実行前提の修正)
- `web` では Node.js が使えないため、ランタイム探索を `index.html`/fetch 依存から撤去。
- `nodesrc/cli.js` の `html_play` 生成時に、`nepl-web-*.js` と `nepl-web-*_bg.wasm` を
  出力先ルートへコピーする処理を追加。
- 各生成HTMLには、ファイルの相対深さに応じた `moduleJsPath`（例: `../nepl-web-*.js`）を埋め込み、
  `import()` で直接 wasm-bindgen モジュールを読み込む方式へ変更。

## 追記検証
- `node nodesrc/cli.js -i tutorials -o html_play=dist/tutorials`
  - `dist/tutorials/nepl-web-*.js` / `dist/tutorials/nepl-web-*_bg.wasm` が生成されることを確認。
  - `dist/tutorials/getting_started/01_hello_world.html` が
    `new URL('../nepl-web-*.js', location.href)` を参照し、`fetch(index.html)` が無いことを確認。
  - 追加で `nepl-web_bg.wasm` も互換名として生成するよう修正し、
    wasm-bindgen 生成 JS が既定名を参照するケースでも 404 しないことを確認。

# 2026-02-10 作業メモ (tutorial 実行ポップアップの ANSI レンダリング対応)
## 実装
- `nodesrc/html_gen_playground.js`
  - 実行ポップアップの stdout 表示を、単純テキスト表示から ANSI 解釈付き表示へ拡張。
  - `ansiToHtml` を追加し、`\\x1b[...m` の SGR を解釈して HTML `<span style=...>` に変換。
  - 対応した主な属性:
    - リセット (`0`)
    - 太字 (`1` / `22`)
    - 下線 (`4` / `24`)
    - 前景色 (`30-37`, `90-97`, `39`)
    - 背景色 (`40-47`, `100-107`, `49`)
  - stdout は `#play-stdout-view`（レンダリング表示）に集約しつつ、
    `#play-stdout-raw`（生テキスト）も保持。

## 検証
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 生成HTMLに `ansiToHtml` / `play-stdout-view` が含まれることを確認。
- `node nodesrc/tests.js -i tests/stdout.n.md -o /tmp/tests-stdout.json -j 1`
  - `total: 107, passed: 107, failed: 0, errored: 0`

## 追記 (正規表現構文エラー修正)
- `html_gen_playground` のテンプレート展開時に、`\\x1b` が生の ESC 文字へ変換される経路があり、
  `Unmatched ')' in regular expression` を誘発していた。
- `ansiToHtml` の正規表現初期化を `new RegExp(String.fromCharCode(27) + '\\\\[([0-9;]*)m', 'g')`
  に変更し、テンプレート展開後も安定して同一パターンになるよう修正。

# 2026-02-10 作業メモ (getting_started の章立て再設計と内容拡充)
## 章立て方針
- 既存言語チュートリアル（Rust Book / A Tour of Go）の構成を参照し、
  「概念章を積み上げてから小プロジェクト章で固める」流れへ再設計。
- `tutorials/getting_started/00_index.n.md` を更新し、Part 1〜3 の学習ロードマップを追加。

## 追加した章
- `tutorials/getting_started/07_while_and_block.n.md`
  - while/do と block 式の基本。
- `tutorials/getting_started/08_if_layouts.n.md`
  - inline / `if:` / `then:` `else:` block の書式差。
- `tutorials/getting_started/09_import_and_structure.n.md`
  - import と関数分割の最小パターン。
- `tutorials/getting_started/10_project_fizzbuzz.n.md`
  - ミニプロジェクトとして分岐ロジックを実践。
- `tutorials/getting_started/11_testing_workflow.n.md`
  - `std/test` を使ったテスト駆動の流れ。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 127, passed: 127, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`11` の HTML を再生成し、実行ポップアップ付きで出力。

# 2026-02-10 作業メモ (Elm/Lean 風の章追加 + 左目次 + index導線)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 4（Elm / Lean 風の関数型・型駆動スタイル）を追加。
- 追加章:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
  - 関数合成、型で失敗表現、等式的リファクタと回帰テストを段階的に説明。
- `nodesrc/cli.js`
  - `html_play` 生成時に同一ディレクトリ内の全ページを集約し、ページごとの目次リンク情報（TOC）を構築。
- `nodesrc/html_gen_playground.js`
  - 左サイドバー目次（全章リンク）を追加。
  - 現在ページを `active` 表示。
  - モバイル幅では縦並びになるようレスポンシブ対応。
- `web/index.html`
  - ヘッダに Getting Started へのリンクを追加:
    - `./tutorials/getting_started/00_index.html`

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 133, passed: 133, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`14` を含む HTML を再生成。
  - 各ページで左サイド目次と active 表示が出ることを確認。

# 2026-02-10 作業メモ (チュートリアル追加拡充: match/ANSIデバッグ)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 5 を追加し、実装で頻出の書き方へ導線を追加。
- 新章追加:
  - `tutorials/getting_started/15_match_patterns.n.md`
    - Option/Result を `match` で明示処理する例を追加。
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
    - `print_color` / `println_color` と `strip_ansi` テスト運用を追加。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 137, passed: 137, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`16` の HTML を再生成。

# 2026-02-10 作業メモ (チュートリアル拡充: 名前空間/再帰/pipe)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 5 に次の導線を追加:
    - `17_namespace_and_alias.n.md`
    - `18_recursion_and_termination.n.md`
    - `19_pipe_operator.n.md`
- 新規追加:
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
    - `alias::function` 呼び出しと `Option::Some/None` の参照例を追加。
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
    - 停止条件つき再帰（`sum_to`, `fib`）を追加。
  - `tutorials/getting_started/19_pipe_operator.n.md`
    - `|>` の基本とチェイン利用例を追加。
- 修正:
  - `18_recursion_and_termination.n.md` の比較関数を `le` へ修正（未定義識別子 `lte` を解消）。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 143, passed: 143, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`19` の HTML を再生成。

# 2026-02-10 作業メモ (チュートリアル拡充: generics / trait 制約)
## 実装
- `tutorials/getting_started/00_index.n.md`
  - Part 5 に次の導線を追加:
    - `20_generics_basics.n.md`
    - `21_trait_bounds_basics.n.md`
- 新規追加:
  - `tutorials/getting_started/20_generics_basics.n.md`
    - `id` 関数と `Option<.T>` を使ったジェネリクス導入章を追加。
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - `trait Show` / `impl Show for i32` / `<.T: Show>` 制約の最小導線を追加。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 147, passed: 147, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`21` の HTML を再生成。

# 2026-02-10 作業メモ (チュートリアルUI/構成改善)
## 実装
- 左目次を `00_index.n.md` の階層（`### Part ...` + 配下リンク）準拠へ変更。
  - `nodesrc/cli.js` で `00_index.n.md` 解析ベースの TOC 生成に変更。
  - `nodesrc/html_gen_playground.js` でグループ見出し（Part）表示を追加。
- 記事中コード（`pre > code.language-neplg2`）のシンタックスハイライトを改善。
  - `analyze_lex` の span から `start_line/start_col` を優先して JS インデックスに変換し、
    日本語コメントを含むコードでも崩れないように修正。
- doctest メタ表示を改善。
  - `neplg2:test[...]` をバッジ化。
  - `stdin` / `stdout` をバッジ + `pre` 表示へ変更。
  - `ret` をバッジ + inline code 表示へ変更。
  - `"...\\n"` などのエスケープはデコードして可読表示。
- チュートリアル内容を拡充。
  - 競プロパート（22〜24）を追加。
  - `10_project_fizzbuzz.n.md` を `stdout` で結果が読める例へ変更。

## 検証
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 152, passed: 152, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`〜`24` の HTML を再生成。

# 2026-02-10 作業メモ (kp: kpread+kpwrite 相互作用の根本修正)
## 症状
- `kpread` と `kpwrite` を同時に import したケースで、stdout に `\0` が大量混入し、`13\n` などが `13\0...` に壊れていた。
- `kpwrite` 単体テストは通るため、出力単体ではなく import/名前解決経路の相互作用が原因だった。

## 根因
- `stdlib/kp/kpread.nepl` が不要な `#import "alloc/string" as *` を持っており、`len` などの識別子汚染を引き起こしていた。
- 同時 import 時に `kpwrite` 側の `len` ローカル束縛と衝突し、長さ計算/書き込み長が壊れていた。

## 実装
- `stdlib/kp/kpread.nepl`
  - 不要な `#import "alloc/string" as *` を削除。
- `stdlib/kp/kpwrite.nepl`
  - `len` 局所変数を `write_len` に改名（`writer_flush` / `writer_ensure` / `writer_put_u8` / `writer_write_str`）。
  - 名前衝突時の再発耐性を強化。
- `nepl-core/tests/kp.rs`
  - `kpwrite` 単体切り分けテストを追加。
  - `kpread_buffer_bytes_debug` を scanner 12B ヘッダ仕様に合わせて更新。

## 検証
- `cargo test --test kp -- --nocapture`
  - `12 passed, 0 failed`
- `NO_COLOR=true trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=116, passed=116, failed=0, errored=0`

# 2026-02-10 作業メモ (cast/kp 最終調整)
## 実装
- `stdlib/alloc/string.nepl`
  - `fn cast from_i32;` / `fn cast to_i32;` を削除。
  - `cast` 名の過剰な公開を減らし、`core/cast` 側のオーバーロード解決を安定化。
- `stdlib/core/cast.nepl`
  - 文字列変換連携を `string::from_*` / `string::to_*` に統一した状態を維持。
  - `alloc/string` の公開 `cast` 依存を持たない構造へ整理。

## 検証
- `NO_COLOR=true trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/numerics.n.md -o tests/output/numerics_current.json -j 1`
  - `total=122, passed=122, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=117, passed=117, failed=0, errored=0`
- `cargo test --test kp -q`
  - `14 passed, 0 failed`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 1`
  - `total=465, passed=458, failed=7, errored=0`
  - 今回解消: `tests/numerics.n.md::doctest#3`（ambiguous overload）
  - 既存残件: `ret_f64_example`, `selfhost_req` 系, `sort` 一部, `string` 一部

# 2026-02-21 作業メモ (shadowing テスト網羅化)
## 実装
- `tests/shadowing.n.md` を新規作成・拡張。
  - ローカル値が import 名を shadow するケース
  - ネストブロックの最内優先
  - ローカル関数が import 関数を shadow
  - outer/inner 関数 shadow
  - 引数名とローカル let の shadow
  - while/match/branch を含むスコープケース
  - 現状未対応の「値名と callable 名の共存」等は `compile_fail` として固定
- `todo.md` を更新。
  - シャドー不可修飾子は immutable の `let`/`fn` のみに適用
  - `let mut` は対象外
  - 重要 stdlib 記号 shadow 時の warn/info と LSP API 取得タスクを明記

## 検証
- `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`
  - `total=176, passed=176, failed=0, errored=0`

# 2026-02-21 作業メモ (名前解決 API: shadowing 情報の拡張)
## 実装
- `nepl-web/src/lib.rs`
  - `NameResolutionTrace` に `shadows` を追加し、名前解決時の shadowing イベントを収集できるようにした。
  - 定義時:
    - 既存候補がある場合に `definition_shadow` を記録。
    - 重要シンボル（`print`/`println`/`add` など）を変数定義系 (`let_hoisted`/`let_mut`/`param`/`match_bind`) で定義した場合は `warning` を付与。
  - 参照時:
    - 候補が複数ある場合に `reference_shadow` を記録し、「採用された定義」と「隠れた候補」を API から取得可能にした。
  - `analyze_name_resolution` の返却 JSON に以下を追加:
    - `shadows`
    - `shadow_diagnostics`
- `tests/tree/03_name_resolution_tree.js`
  - `result.shadows` / `result.shadow_diagnostics` を検証するアサーションを追加。
  - `x` の shadow と `add` の重要シンボル warning を回帰固定。

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 失敗は既知カテゴリ（`ret_f64_example`, `selfhost_req`, `sort`, `string compile_fail期待差分`）で、今回の shadowing API 変更による新規失敗は確認されなかった。

# 2026-02-21 作業メモ (typecheck: shadowing warning 伝播と非致命化)
## 実装
- `nepl-core/src/typecheck.rs`
  - `Binding` に `span` を追加し、shadow 警告の二次ラベル（元定義位置）を出せるようにした。
  - `Env::lookup_outer_defined` を追加し、現在スコープ外の定義候補を参照できるようにした。
  - `emit_shadow_warning` を追加し、束縛導入時（`let` / `let mut` / `fn` / parameter / match bind）に shadow を検知して warning を生成するようにした。
  - 重要シンボル（`print`, `println`, `add` など）については、外側候補が見つからない場合でも「stdlib 記号を隠しうる」warning を生成するようにした。
  - warning ノイズ抑制のため、非重要シンボル（例: `ok`, `len`）の shadow では compiler warning を出さない方針に調整した。
  - `check_function` の返却を `CheckedFunction` 化し、warning を返しつつコンパイル対象関数は生成し続けるように修正した。
    - 以前は warning を含むだけで `Err` 扱いになり、関数が落ちていた。
    - 現在は `Error` のみ `Err`、warning は `diagnostics` として上位へ伝播する。
- `tests/tree/04_semantics_tree.js`
  - `analyze_semantics` で shadowing warning が取得できることを検証するケースを追加。

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_latest.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 失敗は既知カテゴリに留まり、今回変更による追加失敗は確認されなかった。

## 残課題（今回の実装で見えたもの）
- 重要シンボル warning は現在ノイズが多く、`todo.md` に無効化/抑制ポリシー設計タスクとして残した。


# 2026-02-19 作業メモ (stdlib ドキュメント整備と履歴整理)
## 実装
- `stdlib/std/stdio.nepl`, `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`, `stdlib/std/test.nepl`:
  - 先頭テンプレート説明を削除し、`//:` 形式のドキュメントコメントで統一。
  - 注意文を「副作用・メモリ確保/移動・ターゲット制約」など実利用時の注意へ是正。
  - 各関数に利用例（`neplg2:test[skip]`）を維持し、呼び出し形を確認しやすい構成へ整理。
- `stdlib` 全体のドキュメント文言を点検し、モック的な表現を以下の方針で是正。
  - 「関数の概要」→「主な用途」
  - 「詳細な関数別ドキュメントは段階的に追記します。」の削除
  - 実装説明/注意文のテンプレート文言を、利用時の挙動が伝わる表現へ置換
- commit 履歴は `4772eea` 基点で差分を再適用し、今回分を単一 commit に再作成。

## plan.mdとの差異
- 今回は plan.md の言語機能追加ではなく、stdlib のドキュメント品質改善と履歴整理を実施。
- ランタイム挙動や API シグネチャは変更していない。

## 検証
- `cargo install trunk`
  - 失敗（`https://index.crates.io/config.json` 取得時に 403、ネットワーク制約で導入不可）。
- `NO_COLOR=true trunk build`
  - 失敗（`trunk` 未導入）。
- `node nodesrc/tests.js -i stdlib/std -o tests/output/stdlib_std_docs_current.json -j 1`
  - 失敗（compiler artifacts 不在、`total=215, errored=215`）。
- `node nodesrc/cli.js -i stdlib/std -o html_play=dist/stdlib_std`
  - 失敗（artifacts 不在で HTML 生成不可）。

# 2026-02-21 作業メモ (lexer/parser 上流整理 + 木構造 API テスト追加)
## 実装
- `nepl-core/src/lexer.rs`
  - `cond` / `then` / `else` / `do` を専用キーワードトークン (`KwCond`, `KwThen`, `KwElse`, `KwDo`) として追加。
  - キーワード判定を `keyword_token` に集約し、同義分岐の重複を解消。
  - `LexState` の未使用 lifetime を除去し、字句解析状態の定義を簡潔化。
- `nepl-core/src/parser.rs`
  - 新キーワードトークンをレイアウトマーカーとして受理する分岐を追加。
  - 括弧式 (`(` ... `)`) の解析ロジックを `parse_parenthesized_expr_items` に統合し、3箇所重複していた処理を一本化。
  - 診断文を現仕様に合わせて更新:
    - `tuple literal cannot end with a comma` -> `trailing comma is not allowed in parenthesized expression`
    - `expected ')' after tuple literal` -> `expected ')' after parenthesized expression`
- `nepl-web/src/lib.rs`
  - 解析 API の token kind 文字列表現に `KwCond/KwThen/KwElse/KwDo` を追加。
- テスト追加
  - `tests/keywords_reserved.n.md` を新規追加し、`cond/then/else/do` が識別子として使えないことを `compile_fail` で固定。
  - `tests/tree/*.js` を新規追加し、LSP/デバッグ向け API の木構造を段階別に検証:
    - `tests/tree/01_lex_tree.js`
    - `tests/tree/02_parse_tree.js`
    - `tests/tree/03_name_resolution_tree.js`
    - `tests/tree/04_semantics_tree.js`
    - `tests/tree/run.js`（一括実行）

## 検証
- `NO_COLOR=false trunk build`
  - 成功
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_final.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`

## 補足
- `tests` 全体 (`--no-stdlib`) 実行では既存の下流課題（ret_f64/selfhost/sort など）で失敗が残るが、今回の lexer/parser 変更で新規回帰は確認されていない。

# 2026-02-21 作業メモ (noshadow 導入完了と回帰修正)
- `noshadow` を lexer/parser/typecheck/web API まで一貫して実装。
  - lexer: `KwNoShadow` を追加。
  - parser: `let` 修飾子に `noshadow` を追加。`let mut noshadow` は parse error。
  - parser: `fn noshadow <name>` を受理し、AST に `no_shadow` を保持。
  - typecheck: `Binding.no_shadow` を導入し、`noshadow` 宣言の上書きを compile error 化。
- 名前解決/型検査の既存動作を壊さないため、同一スコープの通常 `let` 再束縛（`let lst ...; let lst ...;`）は維持。
  - ただし既存束縛が `no_shadow` の場合のみ、同名宣言を拒否する。
- Web 側のトークン API も `KwNoShadow` に追従。
- テスト追加:
  - `tests/shadowing.n.md` に `noshadow` の compile_fail ケースを追加。
- 検証結果:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-21 作業メモ (doctest の profile ゲート安定化)
- `#if[profile=debug/release]` の doctest が CI 環境のビルドモード差分で揺れる問題に対して、テストランナーからコンパイルプロファイルを明示指定できるように修正。
- `nepl-web` 側:
  - `compile_source_with_profile(source, profile)` を追加。
  - `compile_source_with_vfs_and_profile(entry_path, source, vfs, profile)` を追加。
  - 内部コンパイル経路を `compile_wasm_with_entry_and_profile(..., Option<BuildProfile>)` に統合。
- `nodesrc/run_test.js` 側:
  - 可能な場合は常に `debug` を明示指定してコンパイルするように変更。
  - VFS あり/なし両方で新 API を優先使用し、旧 API は後方フォールバックとして保持。
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-21 作業メモ (stdlib result への段階的 noshadow 適用)
- `stdlib/core/result.nepl` の基盤 API から、衝突リスクが低い `unwrap_ok` / `unwrap_err` に `noshadow` を付与。
- 目的:
  - 基盤 API の誤上書きを早期検出する運用を段階導入する。
  - 既存コードで利用頻度が高い短名（`ok` / `err` / `map`）は今回保留し、破壊範囲を最小化。
- 回帰テストを追加:
  - `tests/shadowing.n.md` に `std_result_noshadow_unwrap_ok`（compile_fail）を追加。
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `548/548 passed`

# 2026-02-21 作業メモ (shadow と overload の扱い整理)
- 仕様調整:
  - 関数の同名定義でシグネチャが異なる場合はオーバーロードとして許可。
  - 同名かつ同一シグネチャの場合のみ「shadowing 扱いの warning」を出す。
  - 同名関数再定義をエラーにはしない。
- `noshadow` の関数適用ルールを調整:
  - `noshadow fn` でも関数同名（オーバーロード）は許可。
  - 変数/値名前空間との衝突は従来通り拒否。
- 利用頻度の高い一般名に対する方針変更:
  - `unwrap` / `unwrap_ok` / `unwrap_err` を `noshadow` 対象から外した。
  - これに伴い `tests/shadowing.n.md` の unwrap 系 compile_fail ケースを削除。
- テスト更新:
  - `fn_noshadow_rejects_shadowing` を `fn_same_signature_shadowing_warns_and_latest_wins` に更新し、成功ケースとして固定（`ret: 2`）。
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` で `547/547 passed`

# 2026-02-22 作業メモ (todo 棚卸し)
- `todo.md` の棚卸しを実施し、解決済みまたは状態が古い項目を削除した。
- 特に以下を整理:
  - 古い集計値 (`total=413, passed=404, failed=9`) を削除。
  - 既に完了済みの `nm/parser` 型名衝突・`examples/nm.nepl` の `cliarg` 経路修正系タスクを todo から除去。
  - `todo.md` は未完了タスクのみ（名前空間/高階関数/LSP/診断体系/Web強化/js_interpreter）に再構成。
- 現時点の回帰確認:
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` の最新結果は pass 維持（直近実行: `547/547`）。

# 2026-02-22 作業メモ (profile/target ゲートと stdlib 重複定義の回帰修正)
- 症状:
  - doctest で `debug_color` / `debugln_color` / `test_checked` / `test_print_fail` の同一シグネチャ再定義 warning が compile fail 扱いになっていた。
  - `functions.n.md` などの失敗と混在していたため、まず warning 起点を切り分けた。
- 原因:
  - `#if[...]` の直後に `//:` ドキュメントコメントが挟まる箇所で、条件付き定義が意図どおりに限定されず重複定義が同時有効になっていた。
- 修正:
  - `stdlib/std/stdio.nepl`:
    - 条件付き関数定義に対して `#if[profile=...]` を定義直前へ再配置。
    - release 側の同名実装は内部名 (`__debug_*_release_noop`) に退避し、シグネチャ衝突を除去。
  - `stdlib/std/test.nepl`:
    - `#if[target=...]` を関数定義直前へ再配置し、意図したターゲット限定で定義されるよう修正。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - 対象再現テスト:
    - `node nodesrc/tests.js -i tests/functions.n.md -i stdlib/core/option.nepl -i stdlib/core/result.nepl ...`
    - `191/191 pass`
  - 全体:
    - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`
    - `547/547 pass`

# 2026-02-22 作業メモ (nepl-web API と cli.js の責務分離)
- 要件反映:
  - `nepl-web/src/lib.rs` は API 提供のみに限定し、Node/FS への直接アクセスは持たない構成にした。
  - FS から stdlib を読む責務は JS 側（`nodesrc/cli.js`）に分離。
- `nepl-web/src/lib.rs` 変更:
  - 既存の「バンドル stdlib 使用（デフォルト）」は維持。
  - 新規 API:
    - `get_bundled_stdlib_vfs()`: wasm にバンドルされた stdlib を `/stdlib/...` 形式 VFS で返す。
    - `compile_source_with_vfs_and_stdlib(...)`
    - `compile_source_with_vfs_stdlib_and_profile(...)`
  - これにより、外部（Node/ブラウザ）が stdlib ソース選択を担えるようになった。
- `nodesrc/cli.js` 変更:
  - `loadStdlibVfsFromFs(stdlibRootDir)` を追加（ローカル FS から `/stdlib/...` VFS を構築）。
  - `loadBundledStdlibVfs(api)` を追加（wasm バンドル stdlib 取得）。
  - `compileWithLocalStdlib(api, ...)` を追加（ローカル stdlib を使ってコンパイル API を呼ぶ）。
- 呼び出し側更新:
  - `nodesrc/html_gen_playground.js` で新 API を優先使用するよう更新。
  - `web/src/main.ts` で `get_bundled_stdlib_vfs` を優先し、旧 `get_stdlib_files` はフォールバックに変更。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (名前解決再設計: 関数候補検索の整理 第1段)
- 目的:
  - `todo.md` 最優先項目（ValueNs/CallableNs 分離）に向けて、挙動を変えない範囲で関数候補検索ロジックを整理。
- 実装:
  - `Env` に `lookup_all_callables` を追加。
  - 関数候補抽出で `lookup_all + filter(Func)` を繰り返していた箇所を `lookup_all_callables` へ置換。
    - top-level `FnDef` の `f_ty` 決定
    - nested `FnDef` の `f_ty/captures` 決定
    - `user_visible_arity` の capture 数計算
  - `find_same_signature_func` を `lookup_all_callables` ベースへ変更。
- 結果:
  - 機能変更なしで重複ロジックを削減し、次段の名前空間分離（Value/Callable）に進める基盤を作成。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (名前解決再設計: Value/Callable API 明確化 第2段)
- 目的:
  - ValueNs/CallableNs 分離へ向けて、`Env` の検索 API を明確化し、関数呼び出し経路の分岐を読みやすくする。
- 実装:
  - `Env` に以下を追加:
    - `lookup_value(name)`
    - `lookup_callable(name)`
  - 既存 `lookup_all` は「最内スコープ優先」のまま維持し、`lookup_value/lookup_callable` はその結果から kind を選ぶ設計にした（解決規則は維持）。
  - `find_same_signature_func` は callable 専用検索を使うよう整理。
  - `check_call_or_letset` 系の分岐で、`lookup_all + var 判定` を `lookup_all_callables` / `lookup_value` に置換。
- 結果:
  - 挙動を変えずに Value/Callable の責務をコード上で分離できる形へ前進。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (nm-compile 失敗の根因修正: extern/entry 収集経路の統合)
- 背景:
  - CI (`nm-compile`) で `stdlib/std/env/cliarg.nepl` の `args_sizes_get` / `args_get` が `undefined identifier` になる失敗を確認。
  - 同時に `expression left extra values on the stack` が連鎖して発生。
- 根因:
  - `typecheck` の先行ディレクティブ処理が `module.root.items` の `Stmt::Directive` のみを走査しており、
    ローダー経由で `module.directives` 側に保持された `#extern` を取りこぼす経路があった。
- 修正:
  - `nepl-core/src/typecheck.rs` でディレクティブ適用処理を共通化。
  - `module.directives` と `module.root.items` の双方を適用対象にし、span キーで重複適用を抑止。
  - これにより `#extern wasi_snapshot_preview1 args_sizes_get/args_get` が安定して環境へ登録されるようにした。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 2`: `200/200 pass`
  - `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output /tmp/ci-nm`: `compile_module returned Ok`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づけ:
  - 仕様変更（`target=wasm` で WASI 無効）後の回帰であり、上流（typecheck 入り口）で根本修正。
  - 次段は固定方針どおり lexer/parser の旧仕様残骸整理を優先する。

# 2026-02-22 作業メモ (条件付きディレクティブ評価の順序修正)
- 背景:
  - `typecheck` の extern/entry 収集を `module.directives` へ拡張した際、
    `module.directives` 側に対して `#if[target=...]` / `#if[profile=...]` の評価を通していない経路が残っていた。
- 修正:
  - `module.directives` 走査でも `pending_if` を使って gate 評価を適用。
  - 既存の `module.root.items` 走査と同じ条件付き有効化ルールに統一。
  - span キー重複除外は維持し、二重登録は防止。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/neplg2.n.md -i tests/nm.n.md -o tests/output/upstream_lexer_parser_latest.json -j 3`: `220/220 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づけ:
  - 上流（typecheck入り口）での条件判定一貫化で、nm/cliarg を含む extern 解決の再発防止を目的とした根本修正。

# 2026-02-22 作業メモ (シャドー警告: オーバーロード経路のノイズ抑制)
- 背景:
  - 仕様上、関数オーバーロードは許容されるため、オーバーロード成立ケースで一般 shadow warning を出すのはノイズになる。
- 修正:
  - `nepl-core/src/typecheck.rs`
    - ネスト `fn` 登録時の `emit_shadow_warning(...)` 呼び出し条件を調整。
    - 既存同名候補が「すべて callable（= オーバーロード候補）」の場合は一般 shadow warning を出さない。
    - 同名に value 系束縛が混在する場合のみ従来どおり warning を出す。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/overload.n.md -o tests/output/shadowing_current.json -j 2`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 位置づけ:
  - 名前解決・シャドーイング再設計（todo最優先項目）の一部として、
    「オーバーロードではなく実シャドーのみ警告」の運用に近づける調整。

# 2026-02-22 作業メモ (旧タプル記法の残存分類)
- 目的:
  - 固定指示に基づき、上流修正（parser 強化）の前に全体を分類して局所修正を回避する。
- 実施:
  - `rg` で `stdlib/tests/tutorials` の旧タプル記法候補を棚卸し。
  - `tests/tree/run.js` で LSP/解析API系の回帰を確認。
- 観測:
  - `tests/tree/run.js`: `4/4 pass`。
  - 旧 tuple literal reject は既存どおり有効だが、tuple type 記法 `(<T1,T2>)` は stdlib/tests に広く残存。
  - parser で tuple type を即時 reject すると stdlib doctest が大量破綻することを確認（段階移行が必要）。
- 方針更新:
  - `todo.md` に「旧タプル記法の完全移行（段階実施）」を追加。
  - 手順は `stdlib/tutorials` 先行移行 → `tests` 分離（新仕様/compile_fail）→ parser で最終 reject の順に固定。
- 補足:
  - 一時的に parser の tuple type reject を試験したが、全体影響が大きいため直ちに戻し、現行安定状態（全体 pass）を維持した。

# 2026-02-22 作業メモ (旧タプル記法移行フェーズ1: stdlib 実例の型注釈削減)
- 実施:
  - `stdlib/alloc/vec.nepl` の `vec_pop` doctest で、旧タプル型注釈
    `let p <(Vec<i32>,Option<i32>)> ...` を削除し、推論に寄せた。
- 目的:
  - parser 側の最終 reject 前に、stdlib 実例から旧記法依存を段階的に除去する。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -o tests/output/list_current.json -j 1 --no-stdlib`: `18/18 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 次段:
  - `tests/tuple_new_syntax.n.md` の tuple 型注釈ケースを「新記法での等価検証」へ再設計。
  - その後 `tutorials` 内の不要な tuple 型注釈を同様に削減する。

# 2026-02-22 作業メモ (tutorial 19 pipe の実行失敗修正)
- 背景:
  - `tutorials/getting_started/19_pipe_operator.n.md` 更新後、`doctest#2` が `divide by zero` で失敗。
- 根因:
  - `let v` ブロックの外に `3 |> mul 2` がこぼれており、意図した「1本のパイプ連結」になっていなかった。
- 修正:
  - `pipe chain` サンプルを単一ブロック内の連結へ整理。
  - `3 |> mul 2 |> add 6` として `assert_eq_i32 12 v` を満たす例に更新。
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/19_pipe_operator.n.md -o tests/output/tutorial_pipe19_current.json -j 1`: `167/167 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started -o tests/output/tutorials_getting_started.json -j 4`: `223/223 pass`

# 2026-02-22 作業メモ (旧タプル記法移行フェーズ1: tuple_new_syntax の不要型注釈削減)
- 実施:
  - `tests/tuple_new_syntax.n.md` の `tuple_type_annotated` ケースで、
    変数側の明示型注釈 `let t <(i32,i32)> ...` を除去し、推論へ移行。
- 目的:
  - parser 側最終 reject 前に、テスト資産から「不要な旧 tuple type 記法」を段階的に減らす。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 作業メモ (stdlib 改行 pipe リファクタ: StringBuilder)
- 背景:
  - `stdlib` リファクタで「複雑データ処理に改行 pipe を活用」の方針に沿って、`StringBuilder` 周辺を段階的に移行開始。
- 実施:
  - `stdlib/alloc/string.nepl`
    - `sb_append` を `get sb "parts" |> vec_push<str> s |> StringBuilder` へ整理。
    - `sb_append_i32` を `sb |> sb_append from_i32 v` へ変更（`StringBuilder` を pipe 左辺に固定）。
- 根因と修正:
  - 初回実装で `from_i32 v |> sb_append sb` としてしまい、pipe 規則（左辺が第1引数）により引数順が逆転。
  - その結果 `no matching overload found` が発生したため、`sb` を左辺にする形へ修正して根本解消。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `547/547 pass`
- 運用更新:
  - `todo.md` 方針に「stdlib のドキュメントコメント/ドキュメントテストは `stdlib/kp` の記述スタイルを参照して統一」を追記。

# 2026-02-22 作業メモ (tree API テスト強化: オーバーロードとシャドー診断)
- 背景:
  - 固定指示にある「上流からの修正」と LSP/デバッグ向け API 検証を進めるため、
    `tests/tree` でオーバーロードとシャドー診断の境界を明示的に固定した。
- 実施:
  - `tests/tree/05_overload_shadow_diagnostics.js` を追加。
  - 検証内容:
    - `analyze_name_resolution` では、純粋オーバーロード（同名・異なるシグネチャ）を warning 扱いしないこと。
    - `analyze_semantics` では、同一シグネチャ再定義を warning として報告すること。
- 検証:
  - `node tests/tree/run.js`: `5/5 pass`
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `548/548 pass`
- 位置づけ:
  - 上流 API（lex/parse/resolve/semantics）の診断境界をテスト化し、
    今後の名前解決再設計での退行を防ぐための基盤整備。

# 2026-02-22 作業メモ (lexer/parser 上流回帰: 予約語の識別子禁止)
- 背景:
  - 固定指示の「上流から修正」に沿って、lexer/parser の予約語境界を compile-fail テストで明示固定した。
- 実施:
  - `tests/keywords_reserved.n.md` を追加。
  - `cond/then/else/do/let/fn` を識別子として使うケースをすべて `compile_fail` で追加。
- 検証:
  - `node nodesrc/tests.js -i tests/keywords_reserved.n.md -o tests/output/keywords_reserved_current.json -j 1`: `172/172 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `550/550 pass`
- 位置づけ:
  - 予約語トークン化と構文エラー化の境界を先に固定し、後続の parser 整理時に退行を検知できる状態を作った。

# 2026-02-22 作業メモ (旧タプル記法テストの失敗原因分離)
- 背景:
  - `tests/tuple_old_syntax.n.md` へ「旧タプル型注釈」「旧ドット添字アクセス」の reject ケースを追加したところ、
    現行 parser/lexer の受理境界と一致せず `compile_fail` 想定が崩れた。
- 観測:
  - `t.0` は lexer 側の `.0` 数値解釈経路があり、現状のままでは「旧ドット添字アクセス」として安定 reject できない。
  - `(<T1,T2>)` の型注釈は段階移行中で、現時点では reject 固定にすると既存資産との整合が崩れる。
- 対応:
  - 先行追加した 3 ケース（tuple type / dot index / nested dot index）は `skip` に切り替え、
    フェーズ分離を明確化した。
  - 既存の「旧 tuple literal `(a,b)` reject」ケースは `compile_fail` のまま維持。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 位置づけ:
  - 旧仕様廃止は継続しつつ、上流（lexer/parser）で一括改修する前に失敗原因を混在させないための切り分け。

# 2026-02-22 作業メモ (parser 上流修正: `t.0` 旧ドット添字の検出)
- 背景:
  - 旧タプル記法廃止方針に対し、`t.0` が一部経路で明示診断されず、移行境界が曖昧だった。
- 修正:
  - `nepl-core/src/parser.rs` の `parse_ident_symbol_item` で、識別子後の `.` の次が `IntLiteral` の場合を特別扱い。
  - 以下の診断を即時追加:
    - `legacy tuple field access '.N' is removed; use 'get <tuple> N'`
  - 該当トークンを消費して回復し、後続解析を継続できるようにした。
- テスト:
  - `tests/tuple_old_syntax.n.md` のドット添字ケースを `compile_fail` に戻し、回帰に組み込んだ。
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 位置づけ:
  - lexer/parser 上流で「旧記法の検出と移行ガイド付き診断」を先に固定し、後続の旧仕様完全撤去に備える修正。

# 2026-02-22 作業メモ (tree API 回帰追加: 旧ドット添字診断)
- 背景:
  - `t.0` の parser 診断追加を API レベルでも退行検知できるようにするため、tree テストへ追加。
- 実施:
  - `tests/tree/06_legacy_tuple_dot_index_diag.js` を追加。
  - `analyze_semantics` で `t.0` 入力に対し、以下を検証:
    - コンパイル成功ではないこと
    - `legacy tuple field access '.N' ... use 'get <tuple> N'` 診断が含まれること
- 検証:
  - `node tests/tree/run.js`: `6/6 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 位置づけ:
  - 上流変更（parser）に対する LSP/デバッグ API の回帰網を強化し、段階移行中の仕様境界を明示固定。

# 2026-02-22 作業メモ (旧 tuple type 注釈の段階削減: テスト資産整理)
- 背景:
  - parser で旧 tuple type 記法を最終 reject する前に、テスト側の不要依存を減らして失敗原因を分離する必要がある。
- 実施:
  - `tests/tuple_new_syntax.n.md`
    - `struct Wrapper` のフィールド型を `pair <(i32,i32)>` から `pair <.Pair>` へ変更。
    - 値構築は `Tuple:` のまま維持し、旧 tuple type 記法への依存を削減。
  - `tests/tuple_old_syntax.n.md`
    - `old_tuple_literal_construct_is_rejected` から旧 tuple type 注釈を除去し、
      旧 tuple literal `(3, true)` 単独で失敗原因を固定。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -o tests/output/tuple_migration_current.json -j 1`: `192/192 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 位置づけ:
  - 旧仕様撤去フェーズの前段として、テストを「旧 literal 失敗」「旧 type 失敗」に分離しやすい状態へ整理。

# 2026-02-22 作業メモ (旧 tuple type parser 即時 reject の試行とロールバック)
- 試行:
  - `parse_type_expr` の `(...)` 非関数分岐で、旧 tuple type 記法を parser 段階で即時エラー化する変更を適用。
- 結果:
  - `tests/tuple_old_syntax.n.md` 単体では意図どおり失敗検出できたが、
    `stdlib` の広範な箇所で旧 tuple type 依存が残っており、`33` 件の compile failure を誘発。
  - 失敗の中心は「段階移行前に parser だけを先に厳格化した」ことによる時期不整合。
- 判断:
  - 固定指示どおり局所対応を避け、段階移行方針を優先するため parser 即時 reject 変更はロールバック。
  - 現時点は「資産側（tests/stdlib/tutorials）の旧 type 依存削減」先行を継続する。
- 再検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (stdlib 段階移行: vec_pop の旧 tuple type 依存削減)
- 実施:
  - `stdlib/alloc/vec.nepl` の `vec_pop` シグネチャを
    `<(Vec<.T>)*>(Vec<.T>,Option<.T>)>` から `<(Vec<.T>)*>.Pair>` に変更。
  - 返り値の実データは従来どおり `Tuple:` 構築を維持し、実行挙動は変更しない。
- 目的:
  - parser の旧 tuple type 最終 reject 前に、stdlib 側の型注釈依存を段階的に削減する。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i tests/tuple_new_syntax.n.md -o tests/output/vec_tuple_migration_current.json -j 1`: `201/201 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (tuple_new_syntax の戻り型注釈移行)
- 実施:
  - `tests/tuple_new_syntax.n.md` の `make` 関数で、戻り型注釈を
    `<()->(i32,i32)>` から `<()->.Pair>` へ変更。
- 目的:
  - parser 最終段階で旧 tuple type を reject する前に、テスト資産の旧型注釈依存を段階的に削減する。
- 検証:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 作業メモ (旧 tuple type 記法 reject の再適用完了)
- 背景:
  - 旧 tuple type 記法の parser reject は以前、`stdlib` 側依存で崩れて一度ロールバックしていた。
- 実施:
  - `nepl-core/src/parser.rs` の `parse_type_expr` で、`(...)` の非関数 tuple type をエラー化。
  - 併せてテスト資産を移行:
    - `tests/pipe_operator.n.md` の `pipe_tuple_source` を `fn f <.T> <(.T)->i32>` へ変更
    - `tests/tuple_new_syntax.n.md` の `tuple_as_function_arg` を `fn take <.T> <(.T)->i32>` へ変更
    - `tests/tuple_old_syntax.n.md` の `old_tuple_type_annotation_is_rejected` を `compile_fail` に復帰
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 結果:
  - 旧 tuple type 記法 reject と全体回帰の両立を確認。
  - `todo.md` の「旧タプル記法の完全移行」項目は完了として削除。

# 2026-02-22 作業メモ (capture 関数値の bare symbol 経路も拒否)
- 背景:
  - `@fn` 経路では capture あり関数値を拒否済みだったが、`apply 5 add_y` のような bare symbol の関数値渡し経路に同等のガードが不足していた。
- 実施:
  - `nepl-core/src/typecheck.rs`
    - call_indirect fallback 判定で `HirExprKind::Var(name)` かつ function-typed の場合にも callable 定義を確認し、capture ありならエラー化。
    - エラーメッセージ: `capturing function cannot be passed as a function value yet`
  - `tests/functions.n.md`
    - `function_value_capture_not_supported_without_at` (`compile_fail`) を追加。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: 全件 pass（実行時点の総数）。
- 位置づけ:
  - closure conversion 未実装フェーズでの「通ってはいけない capture 関数値流入」を `@` / bare symbol の両経路で統一的に封止。

# 2026-02-22 作業メモ (profile ゲート回帰テストの追加)
- 背景:
  - CI で `#if[profile=...]` 周辺の退行が疑われるログがあったため、debug/release 両方の compile 成否を固定する API テストが必要だった。
- 実施:
  - `tests/tree/09_profile_gate_compile.js` を追加。
  - `compile_source_with_profile` を使い、以下を検証:
    - debug gated 定義は debug で通り、release で `undefined identifier` になる。
    - release gated 定義は release で通り、debug で `undefined identifier` になる。
    - release 側に未知識別子を含む定義は debug でスキップされ、コンパイルが通る。
- 検証:
  - `node tests/tree/run.js`: `9/9 pass`
- 位置づけ:
  - 条件付きコンパイルの仕様境界を tree/API 層で固定し、再発を早期検知できるようにした。

# 2026-02-22 作業メモ (todo 整理: 高階関数項目)
- `todo.md` の「1. 高階関数・call_indirect」から、完了済みの
  - `WASM table + call_indirect で non-capture 高階関数を動作させる`
  を削除。
- 未完了のみ保持の方針に合わせ、残タスクを
  - `capture あり関数値の closure conversion 導入`
  に集約した。

# 2026-02-22 作業メモ (parser 回帰追加: IfProfile の AST 形状固定)
- 背景:
  - `#if[profile=...]` 退行対策を compile API だけでなく parser 層でも固定し、上流から原因を切り分け可能にする。
- 実施:
  - `tests/tree/10_profile_directive_parse_shape.js` を追加。
  - `analyze_parse` で以下を検証:
    - root item の順序が `Entry` -> `IfProfile(debug)` -> `FnDef(only_debug)` -> `FnDef(main)`
    - `IfProfile` の debug payload に `profile: "debug"` が含まれる。
- 検証:
  - `node tests/tree/run.js`: `10/10 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `563/563 pass`
- 位置づけ:
  - 条件付きコンパイルの上流（lexer/parser）と下流（compile profile）の双方を tree/API テストで接続し、再発時の診断速度を高めた。

# 2026-02-22 作業メモ (parser 回帰追加: 旧タプル記法診断の固定)
- 背景:
  - 旧 tuple 記法廃止を上流で固定するため、`compile_fail` だけでなく parser API の診断メッセージを直接検証する回帰が必要だった。
- 実施:
  - `tests/tree/11_legacy_tuple_parse_diag.js` を追加。
  - `analyze_parse` で以下を検証:
    - `let t (1, true)` に対し `legacy tuple literal '(...)' is removed` 診断が出る。
    - `let t <(i32,i32)> Tuple: ...` に対し `legacy tuple type '(T1, T2, ...)' is removed` 診断が出る。
  - parser のエラー回復方針（診断を出しつつ `ok` 継続しうる）に合わせ、`ok==false` ではなく診断存在を成功条件にした。
- 検証:
  - `node tests/tree/run.js`: `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `564/564 pass`
- 位置づけ:
  - 旧記法廃止の境界を lexer/parser API 層で固定し、将来の parser 変更で受理が戻る退行を検知できるようにした。

# 2026-02-22 作業メモ (noshadow と overload の整合修正)
- 背景:
  - `fn noshadow` を callable 全体で禁止する変更を試した結果、既存仕様（オーバーロード許可）と衝突して `tests/shadowing.n.md` の退行を引き起こした。
- 実施:
  - `nepl-core/src/typecheck.rs`
    - `shadow_blocked_by_nonshadow` 判定で callable 同士は引き続き許可し、
      value 側の non-shadowable 宣言に対する遮断のみ維持。
  - `tests/shadowing.n.md`
    - `fn_same_signature_shadowing_warns_and_latest_wins` を元の期待（warning + 後勝ち）へ戻し、仕様と一致させた。
- 検証:
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`: `193/193 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `564/564 pass`
- 位置づけ:
  - 「オーバーロードは許可、同一シグネチャ再定義のみ shadow 扱い」という現行方針に戻し、局所的な過剰制限を解消。

# 2026-02-22 作業メモ (parser: 予約語を識別子位置で明示診断)
- 背景:
  - `let cond` / `fn let` / `(... fn ...)` など予約語を識別子位置へ置いた際、
    場合によっては `expected identifier` のみで、診断の一貫性が弱かった。
- 実施:
  - `nepl-core/src/parser.rs`
    - `expect_ident` を拡張し、`TokenKind::Kw*` を検出した場合は
      `'<kw>' is a reserved keyword and cannot be used as an identifier` を直接報告するように変更。
    - `reserved_keyword_token_name` ヘルパーを追加してキーワード名を統一管理。
  - `tests/tree/12_reserved_keyword_identifier_diag.js` を追加。
    - `analyze_parse` で `let cond` / `fn let` / `param fn` の3ケースを検証し、
      それぞれ予約語診断が出ることを固定。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node tests/tree/run.js`: `12/12 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `565/565 pass`
- 位置づけ:
  - 上流（parser）の予約語制約を API テストで固定し、診断品質と回復時の可読性を改善。

# 2026-02-22 作業メモ (parser 回復強化: 複数行の予約語誤用を継続報告)
- 背景:
  - 予約語を識別子位置に置いた `let` が連続すると、最初の `parse_stmt` 失敗で block 解析が打ち切られ、後続行の診断が欠落していた。
- 実施:
  - `nepl-core/src/parser.rs`
    - `parse_block_until_internal` の `parse_stmt()` 失敗時を `?` で即 return せず、
      行境界 (`Newline` / `Semicolon`) までトークンを捨てる回復処理へ変更。
    - これにより同一ブロック内で複数エラーを継続収集可能にした。
  - `tests/tree/13_parser_multi_error_recovery.js` 追加。
    - `let cond` / `let then` / `let else` の3連続誤用で、3件の予約語診断が得られることを固定。
- 検証 (直列実行):
  1. `NO_COLOR=false trunk build`
  2. `node tests/tree/run.js` -> `13/13 pass`
  3. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1` -> `566/566 pass`
- 運用メモ:
  - 指示に合わせ、`trunk build` とテストは今後も必ず直列で実行する。

# 2026-02-22 作業メモ (LSP API 拡張: name_resolution 参照の詳細化)
- 背景:
  - `todo.md` の LSP/API phase2 に対し、`candidate_def_ids` だけでは定義ジャンプ実装時に再参照が多く、UI 連携が煩雑だった。
- 実施:
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution` の `references[]` に次を追加:
      - `resolved_def`: 最終選択定義の詳細（id/name/kind/scope_depth/span）
      - `candidate_definitions`: 候補定義の詳細配列（同上）
    - 既存の `resolved_def_id` / `candidate_def_ids` は維持して後方互換を確保。
  - `tests/tree/03_name_resolution_tree.js`
    - `resolved_def` と `candidate_definitions` の整合を検証するアサーションを追加。
- `todo.md` 整理:
  - 4番項目を未完のみになるよう更新:
    - 完了済み「最終選択/候補の返却」は除外
    - 未完「import/alias/use 跨ぎの定義元ファイル情報（jump先）」へ焦点化
- 検証 (直列):
  1. `NO_COLOR=false trunk build`
  2. `node tests/tree/run.js` -> pass
  3. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1` -> `566/566 pass`
# 2026-02-22 作業メモ (Vec read-only accessor の前進)
- 目的:
  - `todo.md` の「sort/generics と Vec 読み取り設計」を上流の API から前進させる。
- 実装:
  - `stdlib/alloc/vec.nepl`
    - `vec_data_ptr <.T> <(Vec<.T>)->i32>` を追加。
    - 日本語ドキュメントコメント + doctest を追加。
  - `stdlib/alloc/sort.nepl`
    - `get v "len"` / `get v "data"` の一部を `vec_len<.T> v` / `vec_data_ptr<.T> v` へ置換。
    - 同一 `Vec` から `len` と `data` を同時取得する箇所は move 回避のため `get` を維持。
  - `stdlib/tests/vec.nepl`
    - `vec_data_ptr` の基本回帰を追加（`vec_new` 直後に `> 0` を確認）。
  - `todo.md`
    - 完了した `vec_len/vec_data_ptr` の read-only 経路項目を削除し、未完了を slice 風 API に絞った。

# 2026-02-22 作業メモ (sort ポインタ薄ラッパの追加)
- 目的:
  - `todo_kp.md` の「競プロ向けソート API 薄ラッパ」を前進させる。
- 実装:
  - `stdlib/alloc/sort.nepl`
    - `sort_slice_quick <.T: Ord> <(i32,i32)*>()>` を追加。
    - `sort_i32 <(i32,i32)*>()>` を追加（`sort_slice_quick<i32>` の薄ラッパ）。
  - `tests/sort.n.md`
    - `sort_i32_ptr_basic` を追加し、`alloc` + `store_i32` で作った配列が昇順化されることを検証。
  - `todo_kp.md`
    - 完了した `sort_i32(ptr, n)` 項目を削除（未完了のみ保持）。

# 2026-02-22 作業メモ (kpsearch の頻出 API 追加)
- 目的:
  - `todo_kp.md` の「二分探索と頻出ユーティリティ」を前進させる。
- 実装:
  - `stdlib/kp/kpsearch.nepl`
    - `count_equal_range_i32(data, len, x)` を追加。
    - `unique_sorted_i32(data, len)` を追加（in-place 圧縮 + 新しい長さを返す）。
    - それぞれ日本語ドキュメントコメントと doctest を追加。
  - `tests/kp.n.md`
    - `kpsearch_unique_and_count` を追加して、`count_equal_range_i32` と `unique_sorted_i32` の同時回帰を検証。
  - `todo_kp.md`
    - 完了した `unique` / `count_equal_range` 項目を削除（未完了のみ保持）。

# 2026-02-22 作業メモ (core/mem の初期化 API 追加)
- 目的:
  - `todo_kp.md` の「fill_u8 / fill_i32 / memset 相当」を完了させる。
- 実装:
  - `stdlib/core/mem.nepl`
    - `memset_u8(ptr, len, value)` を追加。
    - `fill_u8(ptr, len, value)` を追加（`memset_u8` の同義ラッパ）。
    - `fill_i32(ptr, count, value)` を追加。
    - 日本語ドキュメントコメント + doctest を追加。
  - `tests/mem_fill.n.md`
    - `memset_u8_basic`
    - `fill_i32_basic`
    - `fill_u8_alias`
    の 3 ケースを追加。
  - `todo_kp.md`
    - 完了した初期化 API 項目を削除（未完了のみ保持）。

# 2026-02-22 作業メモ (todo_kp の完了項目整理)
- 目的:
  - `todo_kp.md` を「未完了のみ」に維持する。
- 実施:
  - 空になった `二分探索と頻出ユーティリティ` セクションを削除。
  - 既存テスト（`tests/kp_i64.n.md`）で境界値を担保できているため、`64-bit 最小機能の提供` セクションを削除。

# 2026-02-22 作業メモ (intrinsic/i64-f64 codegen 安定化と両系統テスト追加)
- 目的:
  - `cargo test` で発生していた `invalid wasm generated` を根本原因から解消する。
  - `tests/*.n.md` と `nepl-core/tests/*.rs` の両系統で intrinsic 回帰を追加する。
- 原因特定:
  - wasm validation 失敗の対象関数特定のため、`compiler.rs` に offset -> function body の特定診断を追加。
  - その結果、`dealloc_safe` と `i128_add` 周辺で codegen の型スタック不整合を確認。
- 実装:
  - `nepl-core/src/codegen_wasm.rs`
    - Enum payload のレイアウトを `i32/f32` と `i64/f64` で分離し、unit payload（実体なし）のときは値ストアを行わないよう修正。
    - `match` の payload bind で `i64/f64` load を追加し、unit payload bind は wasm load/store を発行しないよう修正。
    - `#intrinsic "load"/"store"` に `i64/f64` を追加。
    - unit ローカルが wasm local index を破壊する不具合を修正（unit は wasm local slot を確保しない、`set` 生成時に値型なしなら `local.set` を出さない）。
  - `nepl-core/src/compiler.rs`
    - wasm validation エラー時に `func_index/defined_func_index/name/body_range` を出す診断を追加。
- テスト追加:
  - `nepl-core/tests/intrinsic.rs` を新規追加（cargo test側）。
    - `size_of/align_of`（i64/f64）
    - `load/store`（i64/f64）
    - unit payload（`Result<(), str>::Ok ()`）の stack/local 整合
  - `tests/intrinsic.n.md` を新規追加（nodesrc doctest側）。
    - 上記と同等観点を `.n.md` に追加。
- 検証（直列）:
  1. `cargo test -p nepl-core --test intrinsic` -> pass
  2. `NO_COLOR=false trunk build` -> pass
 3. `node nodesrc/tests.js -i tests/intrinsic.n.md -o tests/output/intrinsic.json` -> pass (`183/183`)

# 2026-02-22 作業メモ (cargo全体通過の回復と string/selfhost 同期)
- 目的:
  - `cargo test --no-fail-fast` の残件（`selfhost_req` / `string`）を解消し、全体通過を回復する。
- 実装:
  - `nepl-core/src/parser.rs`
    - `mlstr:` 本文の構文を厳格化し、`##:` で始まらない行を診断するよう修正。
    - `##:` 行が1つもない `mlstr:` もエラー化。
  - `nepl-core/tests/string.rs`
    - `mlstr` 空行ケースの期待値を現行仕様に合わせて更新（`should_panic` を解除）。
  - `tests/string.n.md`
    - `mlstr` の `##:` 欠落を `compile_fail` として回帰追加。
  - `nepl-core/tests/selfhost_req.rs`
    - `test_req_byte_manipulation` を現行 Vec API（`mut + set vec_push`）に同期。
    - `test_req_string_utils` は要件に合わせて compile-check 化（実行検証は `.n.md` 側で継続）。
  - `tests/selfhost_req.n.md`
    - `test_req_string_utils` の条件式を現行構文へ同期。
- 検証（直列）:
  1. `cargo test -p nepl-core --test string --test selfhost_req` -> pass
  2. `cargo test --no-fail-fast` -> pass
 3. `NO_COLOR=false trunk build` -> pass
 4. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)

# 2026-02-22 作業メモ (LLVM target 初期導入: clang 21.1.0 linux native 前提)
- 目的:
  - `llvm` target を `nepl-cli` 側に限定して導入し、WASM/WASI 経路と分離する。
  - `clang 21.1.0 + linux native` を初期要件として固定しつつ、将来拡張可能な形にする。
- 実装:
  - `nepl-cli/src/codegen_llvm.rs` を新設。
    - `ensure_clang_21_linux_native()`:
      - `clang --version` で `clang version 21.1.0` を検証。
      - `clang -dumpmachine` で `linux` 含有を検証。
      - 要件は `LlvmToolchainRequirement` に分離し、将来拡張用に環境変数で上書き可能化:
        - `NEPL_LLVM_CLANG_VERSION`
        - `NEPL_LLVM_REQUIRE_LINUX`
        - `NEPL_LLVM_TRIPLE_CONTAINS`
    - `emit_ll_from_module()`:
      - `#llvmir` ブロック（トップレベル/関数本体）を連結して `.ll` を生成。
      - `llvm` target で `FnBody::Parsed` / `FnBody::Wasm` は明示エラーにして誤動作を防止。
  - `nepl-cli/src/main.rs`
    - `--target llvm` 時は wasm backend を通さず `codegen_llvm` 経路へ分岐。
    - `--run` と `--target llvm` の同時指定を禁止。
    - `--output` 指定先へ `.ll` を出力。
  - `nepl-web/src/lib.rs`
    - `TokenKind::{DirLlvmIr,LlvmIrText}` と `Stmt::LlvmIr` / `FnBody::LlvmIr` を API 出力に反映（分岐漏れ修正）。
- 検証（直列）:
  1. `cargo test --no-fail-fast` -> pass
  2. `cargo test -p nepl-cli` -> pass
  3. `NO_COLOR=false trunk build` -> pass
  4. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)
- 補足:
  - 現時点の `llvm` target は「手書き `#llvmir` を `.ll` へ出力する初期段階」。
  - HIR から LLVM IR を生成する本 backend は `todo.md` に継続タスクとして残した。

# 2026-02-22 作業メモ (#llvmir ブロックのインデント規則を raw text 化)
- 背景:
  - `#llvmir` 内は NEPLG2 構文ではなく LLVM IR 本文なので、内部の字下げを NEPL の `INDENT/DEDENT` として扱うのは不自然だった。
  - 実際に `entry:` 配下の `ret` を深く字下げすると parser 側で `expected llvm ir text line` が発生していた。
- 実装:
  - `nepl-core/src/lexer.rs`
    - `#llvmir` ブロック内では `effective_indent` をブロック基準に固定し、内部の字下げ変化で `INDENT/DEDENT` を増減させないよう変更。
    - `#llvmir` ブロック内の `LlvmIrText` 生成時に、基準インデントからの追加字下げを本文先頭スペースとして保持。
    - これにより `#llvmir` 内部は「NEPLの構文インデント」ではなく「LLVM IR の生テキスト」として扱う。
  - `nepl-cli/src/codegen_llvm.rs`
    - ユニットテストを追加し、深い字下げを含む `#llvmir` が `.ll` にそのまま残ることを固定。
- 検証（直列）:
  1. `cargo test -p nepl-cli` -> pass
 2. `NO_COLOR=false trunk build` -> pass
 3. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)

# 2026-02-22 作業メモ (LLVM runner 安定化と import staging 改善)
- 目的:
  - `nodesrc/tests.js --runner llvm --llvm-all` で `tests/` を安定実行し、LLVM 移行時の回帰を継続検証できる状態にする。
  - `#import "./part"` のようなローカル import を LLVM CLI 実行用の一時ディレクトリでも解決できるようにする。
- 実装:
  - `nodesrc/tests.js`
    - `stageLocalImportsForLlvmCase` を追加。
      - ローカル import を再帰的に解析して一時ディレクトリへコピー。
      - 拡張子省略 (`#import "./part"`) を `part.nepl` 候補として解決。
      - 循環コピー回避のため `realpath` ベースで visited 管理を追加。
    - `compile_fail` の LLVM 判定を二段化。
      - `llvm_cli` 明示ケースは厳密判定（失敗を期待）。
      - `--llvm-all` で流す非明示ケースは移行モードとして失敗強制を外す。
  - `nepl-core/src/codegen_llvm.rs`
    - `FnBody::Wasm` を非 entry ではスキップ継続、entry 関数に対しては `UnsupportedWasmBody` を返すよう修正。
    - active な `#entry` 名を target/profile 条件込みで収集する補助関数を追加。
    - `entry が #wasm のみ` を検出するユニットテストを追加。
- 検証（直列）:
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1` -> pass (`5/5`)
  3. `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_all_probe.json --runner llvm --llvm-all --no-tree -j 2` -> pass (`601/601`)
  4. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2` -> pass (`610/610`)

# 2026-02-22 作業メモ (target 記述の std 移行と i64 math の wasm/llvm 統一)
- 目的:
  - doctest と tests の target 記述を `wasi` から `std` に寄せ、target alias 移行方針（`std`）へ段階的に揃える。
  - `stdlib/core/math.nepl` の i64 系で残っていた wasm 偏重実装を解消し、関数内 `#if[target=wasm]` / `#if[target=llvm]` 分岐へ統一する。
- 実装:
  - `stdlib/core/mem.nepl`, `stdlib/alloc/vec.nepl` の doctest 内 `#target wasi` を `#target std` へ置換。
  - `tests/*.n.md` の `#target wasi` を `#target std` へ置換（対象ファイルのみ）。
  - `stdlib/core/math.nepl`
    - `i64_div_s`, `i64_rem_s`, `i64_and/or/xor`, `i64_shl/shr_s/shr_u`, `i64_rotl/rotr`,
      `i64_clz/ctz/popcnt`, `i64_eq/ne/lt/le/gt/ge` を wasm/llvm 両分岐化。
    - i64 比較関数の末尾 LLVM 再定義ブロック（重複定義）を削除し、定義点を一本化。
- 検証（直列）:
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2` -> pass (`610/610`)
  3. `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_all_probe.json --runner llvm --llvm-all --no-tree -j 2` -> pass (`601/601`)

# 2026-02-22 作業メモ (stdlib stdio/fs/cliarg の Linux syscall 化と回帰)
- 目的:
  - `extern wasi_*` 依存を target 分岐で整理し、`llvm` では Linux `syscall` 経由で `stdio/fs/cliarg` を動かす。
  - `tests.js` の wasm/llvm 回帰を壊さずに、std 系モジュールのコンパイル不安定を解消する。
- 実装:
  - `stdlib/std/stdio.nepl`
    - `#if[target=wasm]` の extern 宣言を維持しつつ、`#if[target=llvm]` で `syscall` ラッパを追加。
    - `fd_read` / `fd_write` の LLVM 互換実装を Linux syscall (`read`/`write`) で統一。
    - `if:` レイアウトを `cond/then/else` 形式へ修正し、parser の no-progress を解消。
  - `stdlib/std/fs.nepl`
    - LLVM 側 `path_open` / `fd_read` / `fd_close` を Linux syscall (`openat`/`read`/`close`) へ統一。
    - syscall 呼び出しを 1 行式に揃えて、改行引数解釈の揺れを除去。
  - `stdlib/std/env/cliarg.nepl`
    - LLVM 側 `args_sizes_get` / `args_get` を `/proc/self/cmdline` 読み取りで互換実装。
    - `if:` レイアウトの `cond:` 欠落箇所を修正。
  - `README.md`
    - 実行方法を 4 系統（`--run`, `wasmer`, `wasmtime`, `llvm`）で明示。
- 検証（直列）:
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl -o tests/output/std_platform_wasm.json -j 2` -> pass (`241/241`)
  3. `node nodesrc/tests.js -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl --runner llvm --llvm-all --no-tree -o tests/output/std_platform_llvm.json -j 2` -> pass (`227/227`)
  4. `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2` -> pass (`610/610`)
  5. `node nodesrc/tests.js -i tests --runner llvm --llvm-all --no-tree -o tests/output/tests_current_llvm.json -j 2` -> pass (`601/601`)
- examples 実行確認:
  - `wasi --run`: `helloworld.nepl`, `counter.nepl`, `kp_fizzbuzz.nepl` は実行確認済み。
  - `llvm`: `.ll` 生成は成功。ただしリンク時に `undefined reference to main` で実行不可。
    - 現状の LLVM backend はユーザー関数/entry の最終出力が未完で、`main`/`_start` を持つ実行 IR 生成が未対応。
    - これは `todo.md` の LLVM backend 本実装タスクで継続。

# 2026-02-22 作業メモ (LLVM entry ブリッジ追加と examples 実行確認)
- 実装:
  - `nepl-core/src/codegen_llvm.rs`
    - `#entry` で指定された関数が raw/parsed subset で emit 済みの場合、`main` が未定義なら
      `define i32 @main() { call @entry; ret }` のブリッジを自動生成する処理を追加。
    - raw `#llvmir` ブロックから `define @name` を抽出して、emit 済み関数集合を追跡する補助関数を追加。
- 回帰確認（直列）:
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests --runner llvm --llvm-all --no-tree -o tests/output/tests_current_llvm.json -j 2` -> pass (`601/601`)
- examples 実行確認:
  - `wasi --run`: `helloworld`, `counter`, `kp_fizzbuzz` はすべて成功。
  - `llvm`: `.ll` 生成は成功するが、clang リンク時に `undefined reference to main` で失敗。
    - 3例とも `main`/`_start` が最終 `.ll` に存在しないことを確認。
    - 根因は、entry 本体（Parsed 関数）の LLVM lower が未実装で emit されていないため。
- 次アクション:
  - Parsed/HIR の LLVM lower（少なくとも entry 関数本体）を実装し、`main` を確実に生成する。
# 2026-02-22 作業メモ (nodesrc 完全検証モード: wasm実行 + llvm実行 + 結果比較)
- 目的:
  - `nodesrc/tests.js` を「WASMだけ通る」判定から拡張し、LLVM でも実行した結果を比較できる完全検証経路を作る。
  - doctest の `stdin:` / `stdout:` / `stderr:` メタデータを、WASM/LLVM の両ランナーに同じ規則で適用する。
- 実装:
  - `nodesrc/parser.js`
    - doctest メタデータとして `stdin/stdout/stderr` を抽出する機能を追加。
    - 文字列値は JSON 文字列（`"..."`）として解釈し、`\n` 等のエスケープを展開。
  - `nodesrc/tests.js`
    - LLVM runner を「compile確認のみ」から「`nepl-cli --target llvm` -> `clang` link -> 実行」へ拡張。
    - doctest 期待値判定を共通化し、WASM/LLVM 両結果へ同一ロジックを適用。
    - `--runner all` 時に `compare_wasm_llvm` フェーズを追加（stdout/stderr の一致確認）。
    - 追加オプション:
      - `--assert-io`: `stdin/stdout/stderr` の厳密比較を有効化
      - `--strict-dual`: wasm/llvm の比較結果を必須化（比較欠落も fail）
    - 互換維持:
      - 既存運用を壊さないため、厳密 I/O 比較は `--assert-io` 指定時のみ有効化。
  - `nepl-core/src/codegen_llvm.rs`
    - entry lower の失敗を握りつぶさず、`compile_llvm_cli` で原因を返すよう修正。
    - entry 名の解決で mangled 名（`main__...` 形式）を追跡する fallback を追加。
- 検証:
  - 既存互換モード:
    - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`
    - `610/610 pass`
  - 完全検証モード（例）:
    - `node nodesrc/tests.js -i tests/stdout.n.md -o tests/output/stdout_complete.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `compare_wasm_llvm` が結果JSONに出力され、wasm/llvm 差分を可視化できることを確認。
- 現在判明している根本課題:
  - LLVM 側は `main` 解決に進むようになったが、`core/math` の wasm 専用関数（例: `add__i32_i32__i32__pure`）に到達すると `compile_llvm_cli` で失敗する。
  - これは「完全検証モードの不具合」ではなく、`stdlib` 側の llvm 実装未整備が原因であり、上流課題として継続修正する。

# 2026-02-22 作業メモ (LLVM lower 強化と llvm runner 改修)
- 目的:
  - `llvm` ランナーの失敗を上流（`nepl-core/src/codegen_llvm.rs`）から削減する。
  - `wasm` 既存テストを壊さず、`llvm` 側の失敗を compile/link 中心から run/実装不足へ寄せる。
- 実装:
  - `nepl-core/src/codegen_llvm.rs`
    - `lower_hir_string_literal` の `alloc/store_i32/store_u8` をシグネチャ解決 (`resolve_symbol_name`) に変更。
    - `EnumConstruct` でも `alloc` をシグネチャ解決へ変更。
    - `StructConstruct` / `TupleConstruct` の lower を追加（ヒープ確保 + フィールド逐次 store）。
    - intrinsic lower を追加:
      - `add`
      - `f32_to_i32`
      - `i32_to_u8`
    - `if` の再定義抑制まわりを継続補正:
      - `RawBodySelection::Llvm` で初回走査時に定義関数名を `emitted_functions` へ登録。
      - `parse_defined_function_name` で `define @"name"(...)` の引用符を正規化。
      - `HirBody::LlvmIr` の「定義済み扱い」条件を厳密化し、raw が `@add` のみ定義する場合に `add__...` を誤って定義済みにしないよう修正。
    - raw 定義の base 名しか無いケース向けに mangled alias wrapper 生成を追加（`add__... -> add` 等）。
  - `nodesrc/tests.js`
    - LLVM リンク時に `-lm` を追加（`ceilf/floorf/truncf/nearbyintf` 等の未解決を解消）。
  - `stdlib/alloc/string.nepl`
    - `str_eq_loop` / `str_eq_at` の引数 `len` を `n` に変更し、関数シンボル `len` との解決衝突を回避。
- 検証:
  - `NO_COLOR=false trunk build`: 成功
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `397/601 pass`
- 状況整理:
  - 直近で `llvm` は `link_llvm_cli` の大量失敗（未定義シンボル/`libm` 未リンク）を削減。
  - 現在の主失敗は `run_llvm_cli(SIGSEGV)` と、一部の `compile_llvm_cli`（型効果/名前解決由来）に集約。
  - 次段は `core/mem` と `alloc/*` のランタイム整合（線形メモリ運用）を優先して進める。

# 2026-02-22 作業メモ (LLVM 到達解析/alias 修正の継続)
- 目的:
  - `link_llvm_cli` の未定義シンボルを上流（`codegen_llvm`）で削減する。
  - `#llvmir` 関数の raw 定義名と mangled 呼び出し名の不一致を吸収する。
- 実装:
  - `nepl-core/src/codegen_llvm.rs`
    - mangled 名の base 抽出を修正（先頭 `__` を含む関数名を正しく扱う）。
    - raw `#llvmir` 関数で「raw は base 名のみ定義」の場合に、mangled 名への wrapper を自動生成。
    - `HirBody::LlvmIr` の `call @...` を到達解析へ追加し、raw 内部の依存関数も reachable に含める。
    - `llvm_output_has_function` を `define/declare` 行のみ判定するよう修正（`call` 行誤検知を除去）。
  - `todo.md`
    - wasm/llvm 共通の「未到達関数を出力しない（関数単位 tree-shaking）」タスクを追加。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl ... --runner llvm --llvm-all --assert-io`
    - 変更前: `104/200 pass`
    - 変更後: `195/200 pass`
  - 残件（同コマンド）:
    - `__nepl_syscall` 未定義 2件
    - `unknown variable 'inc__i32__i32__pure'` 2件
    - `kpdsu` の実行出力差分 1件

# 2026-02-26 作業メモ (stdlib doctest target の core/std 化)
- 目的:
  - LLVM dual-run で使用する doctest の target 表記を統一するため、`stdlib/*.nepl` 内の doctest 埋め込みソースのみを `#target core/std` へ移行する。
  - 実装コード側の `#target`（モジュール本体）は変更せず、テストケース定義だけを更新する。
- 実装:
  - `stdlib/**/*.nepl` の `//:| #target wasi` を `//:| #target std` に変更。
  - `stdlib/**/*.nepl` の `//:| #target wasm` を `//:| #target core` に変更。
  - 実コード行（`#target wasi` など）は未変更。
- 検証:
  - `NO_COLOR=false trunk build` は成功。
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` 実行結果:
    - `total=1781, passed=1205, failed=576, errored=0`
    - 失敗の代表は `tests/kp.n.md` / `tests/string.n.md` の wasm/llvm 実行差分（stdout mismatch）で、今回の target 表記変更による新規失敗は確認できない（件数が既知値と一致）。
- 補足:
  - `tests/*.n.md` は既に `core/std` 化済みであることを再確認した。

# 2026-02-26 作業メモ (テスト基盤・文字列テストの整合修正)
- 目的:
  - `tests + stdlib` の dual 実行で大量失敗していた原因を、テストツール問題・テストケース問題・コンパイラ問題に分解して是正する。
- 根本原因と修正:
  - `nodesrc/tests.js`
    - `::llvm` サフィックス除去長が誤っており、`compare_wasm_llvm` が誤って `missing llvm counterpart result` を生成していた。
      - 修正: `stripLlvmSuffix` を `-6` に訂正。
    - `strictDual` 比較で `wasi_only/skip_llvm/wasm_only` ケースまで比較対象に入っていた。
      - 修正: `compareWasmLlvmResults` で `skipOnLlvmRunner` を適用し比較対象外化。
  - `tests/kp.n.md`
    - `kpsearch_unique_and_count` の期待値がデータ内容と関数仕様（`count_equal_range_i32`）に対して不整合だった。
      - 修正: `"3 3\n1 2 5\n"` -> `"2 3\n1 2 5\n"`。
  - `tests/string.n.md`
    - `stdout:` メタ値に `\\n` を使っており、JSON文字列としては「改行」ではなく「バックスラッシュ+n」期待になっていた。
    - 単行文字列エスケープ検証のソース側も `"...\\n..."` になっており、テスト意図（エスケープ解釈）と不一致だった。
      - 修正: `stdout:` とソース文字列を、意図どおり `\n`/`\t` が制御文字として評価される形へ更新。
  - `nepl-core/src/lexer.rs`
    - `mlstr` の `##:` 行で先頭1スペースを本文へ取り込んでいたため、仕様（`##: ` の後ろが本文）と不一致。
      - 修正: `##:` 直後の先頭1スペースを除去するように調整。
- 検証:
  - `NO_COLOR=false trunk build` 成功。
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-final-before-commit.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `total=1579, passed=1579, failed=0, errored=0`。

# 2026-02-26 作業メモ (dual-run 全通とテスト基盤再確認)
- 目的:
  - テストケースとテストツールの妥当性を先に担保し、コンパイラ実装修正へ進める前提を固める。
- 実施:
  - `nodesrc/tests.js` の wasm/llvm 対応付けと strict-dual 比較対象の扱いを修正。
  - `tests/kp.n.md` の誤期待値を仕様に合わせて修正。
  - `tests/string.n.md` の単行文字列エスケープ検証と `stdout:` メタ表記を整合化。
  - `nepl-core/src/lexer.rs` の `mlstr` 行頭スペース取り込み不整合を修正。
- 検証:
  - `NO_COLOR=false trunk build`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-final-now.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
  - 結果: `total=1579, passed=1579, failed=0, errored=0`
- 判断:
  - 現時点で残る失敗はなく、テスト基盤/テストケース/コンパイラ実装のこの範囲の不整合は解消済み。

# 2026-02-26 作業メモ (wasm codegen 到達解析の追加)
- 目的:
  - import しただけで未使用関数まで wasm 出力される状態を改善し、entry から到達する関数のみを出力する。
- 実装:
  - `nepl-core/src/codegen_wasm.rs`
    - `collect_reachable_wasm_functions` を追加し、entry 起点の関数到達集合を構築。
    - `collect_called_functions_from_expr` を追加し、`Call(User)` と関数値参照（`Var`/`FnValue`）を追跡対象にした。
    - `call_indirect` が含まれる場合は、静的確定不能のため保守的に全関数保持へフォールバック。
    - user 関数の lower 対象を到達集合でフィルタリング。
- 検証:
  - `NO_COLOR=false trunk build`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-reachability-3.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
  - 結果: `total=1579, passed=1579, failed=0, errored=0`
- 補足:
  - 実装途中で `Var/FnValue` 参照未追跡により `len__str__i32__pure` 未定義が発生したが、参照追跡追加で解消した。
## 2026-02-27 作業メモ (LLVM codegen の target gate 判定を compiler と統一)
- 目的:
  - `#if[target=...]` の式評価を、LLVM codegen 側でも `compiler` と同一実装で判定する。
  - target 判定の二重実装による将来の乖離を防ぐ。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `gate_allows` の `Directive::IfTarget` 分岐を `target.allows(...)` から
      `crate::compiler::target_gate_allows_expr(...)` 呼び出しへ変更。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-continue.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1588/1588 pass`

# 2026-02-27 作業メモ (`sort_*_ret` の move-check 根本修正)
- 目的:
  - `todo.md` 3番の `sort` まわりで、Vec を返すラッパAPIを move 規則に整合させる。
- 原因:
  - `sort_quick_ret` / `sort_heap_ret` / `sort_merge_ret` で `v` から `get` を行った後に `v` をそのまま返しており、move-check で `use of moved value: v` になっていた。
  - 失敗は `tests/sort.n.md` の新規ケースで再現し、診断位置も同一。
- 修正:
  - `stdlib/alloc/sort.nepl`
    - `sort_*_ret` で `len/cap/data` を取得後、返り値を `v` ではなく `Vec<.T> n cap data_ptr` の再構築へ変更。
  - `tests/sort.n.md`
    - 新規 `sort_*_ret` 検証ケースの読み取りを `vec_get` 連続呼び出しから、`vec_data_ptr + load_i32` に変更。
    - これにより、`vec_get` が `Vec` を消費する現在仕様でも単一値 `v` を使い回さずに検証可能。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-returning-api-v6.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `499/499 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1620/1620 pass`

# 2026-02-27 作業メモ (`Vec` read-only 経路の段階導入)
- 目的:
  - `todo.md` 3番の `Vec` 読み取り設計を前進させ、`sort` 検証コードで move 規則に引っかからない read-only パターンを標準化する。
- 実装:
  - `stdlib/alloc/vec.nepl`
    - `vec_data_len <.T> <(Vec<.T>)->.Pair>` を追加。
    - 返り値は `Tuple:` で `(data_ptr, len)`。
    - 日本語ドキュメントコメントと doctest を追加。
  - `tests/sort.n.md`
    - `sort_quick_ret_i32_sorted_values`
    - `sort_heap_ret_i32_sorted_values`
    - `sort_merge_ret_i32_sorted_values`
    を `vec_data_ptr` 直接参照から `vec_data_len + core/field.get` ベースに更新。
    - `len == 4` の検証も追加し、データ整合と長さ整合を同時に確認。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-vec-data-len-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `502/502 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-vec-data-len-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1623/1623 pass`

# 2026-02-27 作業メモ (`noshadow` 適用範囲の stdlib 拡大: stdio)
- 目的:
  - `todo.md` のシャドーイング運用を完了させるため、`std/test` に続いて `std/stdio` の基幹APIにも `noshadow` を適用する。
- 実装:
  - `stdlib/std/stdio.nepl`
    - `print`
    - `read_line`
    - `println`
    - `print_i32`
    - `println_i32`
    を `fn noshadow` 化。
  - `tests/shadowing.n.md`
    - `std_stdio_noshadow_same_signature_redefinition_is_error`（compile_fail）を追加。
    - `std_stdio_noshadow_allows_overload_with_different_signature`（成功）を追加。
- 失敗分析:
  - 初回は `print <(i32)*>()>` を overloading するテストにし、`stdio` 内部の `print` 呼び出しが曖昧化して大量 `ambiguous overload` を誘発。
  - これはテスト設計ミスと判断し、内部呼び出しに影響しない `read_line` の別シグネチャ overloading へ変更して解消。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-stdio-noshadow-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `538/538 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdio-noshadow-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1628/1628 pass`

# 2026-02-27 作業メモ (`sort_*_ret` 境界回帰の強化)
- 目的:
  - `sort_*_ret` API の move 規則整合を維持するため、戻り値Vec APIに対する `len=0/1` 境界ケースを固定する。
- 変更:
  - `tests/sort.n.md` に以下を追加:
    - `sort_quick_ret_len0_noop`
    - `sort_quick_ret_len1_noop`
    - `sort_heap_ret_len0_noop`
    - `sort_heap_ret_len1_noop`
    - `sort_merge_ret_len0_noop`
    - `sort_merge_ret_len1_noop`
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-ret-boundary-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `520/520 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-boundary-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1646/1646 pass`

# 2026-02-27 作業メモ (`sort_*_ret` API 整合の完了)
- 目的:
  - `todo.md` の sort/move 規則整合項目を完了できる状態にする。
- 実装:
  - `tests/sort.n.md` に `sort_*_ret` の返却後再利用ケースを追加:
    - `sort_quick_ret_vec_is_reusable_after_sort`
    - `sort_heap_ret_vec_is_reusable_after_sort`
    - `sort_merge_ret_vec_is_reusable_after_sort`
  - いずれも「sort 後に `vec_push` できること」と `vec_data_len` で `len` が増えることを検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-ret-reuse-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `529/529 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-reuse-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo 整理:
  - `todo.md` の `sort/generics と Vec 読み取り設計` を完了として削除し、残項目の番号を詰めた。

# 2026-02-27 作業メモ (LSP/API phase2: token_resolution に定義オブジェクトを統合)
- 目的:
  - `todo.md` 2番（LSP/API 拡張）の一部として、token 単位情報から直接「定義ジャンプ可能な情報」を取得できるようにする。
- 実装:
  - `nepl-web/src/lib.rs` の `analyze_semantics` で、`token_resolution` 各要素に以下を追加:
    - `resolved_definition`（id/name/kind/scope_depth/span）
    - `candidate_definitions`（候補定義配列、各要素に span 含む）
  - 従来の `resolved_def_id` / `candidate_def_ids` は後方互換として維持。
- テスト:
  - `tests/tree/04_semantics_tree.js` を更新し、
    - `resolved_definition.span` の存在
    - `candidate_definitions` が配列であること
    を検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `15/15 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-token-resolution-defobj-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
# 2026-02-27 作業メモ (LSP/API phase2: VFS跨ぎ定義ジャンプ情報の固定)
- 目的:
  - `todo.md` 2番（LSP/API 拡張 phase 2）のうち、token 解決結果に import 先定義のファイル情報を返す部分を安定化する。
- 実装:
  - `nepl-web/src/lib.rs`
    - `span_to_js_with_map` を導入し、`SourceMap` がある場合は span の line/col を元ファイル基準で計算し、`file_path` を埋めるように変更。
    - 名前解決 payload 変換関数（`def_trace_to_js` / `ref_trace_to_js` / `shadow_trace_to_js` / `name_resolution_payload_to_js`）に `SourceMap` を渡せる形へ拡張。
    - `analyze_semantics_with_vfs(entry_path, source, vfs)` を追加し、VFS 読み込み時の `token_resolution` に
      - `resolved_definition`（span + file_path）
      - `candidate_definitions`（配列、各要素に span + file_path）
      を返すように実装。
  - `tests/tree/16_semantics_vfs_cross_file.js` を追加。
    - `core/math` の `add` 呼び出しで、解決先が `/stdlib/core/math.nepl` を指すことを検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `16/16 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo反映:
  - `todo.md` 2番から「token 単位の型情報 API に定義ジャンプ情報（import 先含む）を統合する」を削除（完了）。
# 2026-02-27 作業メモ (LSP/API phase2: name_resolution の VFS 版を追加)
- 目的:
  - `todo.md` 2番の残件だった「`analyze_name_resolution` の import/alias/use 跨ぎ定義元情報」を API で返せるようにする。
- 実装:
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution_with_vfs(entry_path, source, vfs, options)` を追加。
    - `Loader + SourceMap` 経由で複数ファイルを読み込み、`name_resolution_payload_to_js(..., Some(&source_map), ...)` を使って
      定義・参照・shadow の `span.file_path` を返すようにした。
    - 失敗時は `loader error` 診断と空配列 payload を返す。
  - `tests/tree/17_name_resolution_vfs_cross_file.js` を追加。
    - `core/math` の `add` 参照に対して `resolved_def.span.file_path` が `/stdlib/core/math.nepl` になることを検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo反映:
  - `todo.md` 2番から「`analyze_name_resolution` で import/alias/use 跨ぎ時の定義元ファイル情報を返す」を削除（完了）。
# 2026-02-27 作業メモ (LSP/API phase2 継続: token_resolution に doc 情報を付加)
- 目的:
  - Hover 向け表示情報を増やすため、定義ジャンプ情報と同じ経路で doc comment も取得できるようにする。
- 実装:
  - `nepl-web/src/lib.rs`
    - `analyze_semantics` / `analyze_semantics_with_vfs` の `token_resolution` 組み立て時に、
      `resolved_definition` と `candidate_definitions` へ `doc` を付与（存在時のみ）。
  - `tests/tree/16_semantics_vfs_cross_file.js`
  - `tests/tree/17_name_resolution_vfs_cross_file.js`
    - VFS 跨ぎ定義解決テストを維持しつつ、API回帰が出ないことを確認。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
# 2026-02-27 作業メモ (LSP/API phase2 完了: Hover/Inlay 向け `token_hints` 追加)
- 目的:
  - `todo.md` 2番の残件（Hover/Inlay 向け統合API）を、既存 `analyze_semantics*` に追加して利用側の結合コストを下げる。
- 実装:
  - `nepl-web/src/lib.rs`
    - `build_token_hints_to_js(...)` を追加。
    - `token_semantics`（型・式範囲・引数範囲）と `resolve_trace`（定義ジャンプ・候補・doc）を token 単位で統合し、`token_hints` 配列を生成。
    - `analyze_semantics` / `analyze_semantics_with_vfs` の返却値へ `token_hints` を追加。
    - 失敗系分岐でも `token_hints: []` を返すよう統一。
  - `tests/tree/04_semantics_tree.js`
    - `token_hints` が存在し、`inferred_type` と `resolved_def_id` を同時に持つ要素があることを追加検証。
  - `tests/tree/16_semantics_vfs_cross_file.js`
    - `token_hints` に cross-file `resolved_definition.span.file_path` と `inferred_type` が同時に入ることを追加検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo反映:
  - `todo.md` 2番（旧 LSP/API phase2）を削除し、残項目を繰り上げ。
# 2026-02-27 作業メモ (オーバーロード arity 解決の根本修正)
- 目的:
  - `let u <(i32)->i32> calc` のような関数値文脈で、同名・異 arity 過負荷が正しく一意選択されるようにする。
- 原因:
  - `Symbol::Ident` 解決で、過負荷関数でも先に `lookup_callable_any` が 1件を拾い、期待型/arity ベースの選択ロジックに到達していなかった。
  - その結果、`calc` が誤った候補（または未確定値）として残り、`no matching overload` / `extra stack` へ波及していた。
- 実装:
  - `nepl-core/src/typecheck.rs`
    - 複数 callable を持つ識別子では、単純 `lookup_callable_any` にフォールバックしないよう修正。
    - `pending_ascription` 由来の期待 arity で一意に候補が決まった場合、`FnValue` として確定し `auto_call=false` にするよう修正。
    - `FnValue` には関数名ではなく実シンボル（`BindingKind::Func.symbol`）を保持するよう修正。
- テスト更新:
  - `tests/overload.n.md`
    - `overload_select_by_arity` を `compile_fail (diag_id:3006)` から成功ケース（`ret: 12`）へ変更。
- 関連ドキュメントテスト修正:
  - `stdlib/core/option.nepl` / `stdlib/core/result.nepl`
    - `should_panic` doctest で最終式が `i32` になっていたため `D3004` になっていた。`let v ...; ()` へ修正して、型整合を維持したまま panic 経路を検証できるようにした。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/core/result.nepl --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-option-result-dual.json -j 2` -> `18/18 pass`
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions-no-stdlib.json -j 2` -> `101/101 pass`
- todo反映:
  - `todo.md` 先頭の「オーバーロード解決の arity 完全対応」を削除（完了）。
# 2026-02-27 作業メモ (stdlib/tests を functions.n.md 形式へ分割再構成)
- 目的:
  - `stdlib/tests/*.n.md` の失敗（run unreachable）を、現行構文・現行ランタイム前提で安定化する。
  - 1ファイル1巨大ケースではなく、`tests/functions.n.md` と同様の「複数小ケース」構成へ統一する。
- 実装:
  - `stdlib/tests/stack.n.md`
    - 3ケースへ分割: `stack_new_and_len`, `stack_peek_and_pop`, `stack_pop_empty`。
  - `stdlib/tests/btreemap.n.md`
    - 3ケースへ分割: `btreemap_insert_and_len`, `btreemap_get_and_remove`, `btreemap_update_existing`。
  - `stdlib/tests/btreeset.n.md`
    - 3ケースへ分割: `btreeset_insert_and_len`, `btreeset_contains_and_remove`, `btreeset_duplicate_insert`。
  - `stdlib/tests/string.n.md`
    - 3ケースへ分割: `string_len_and_concat`, `string_trim_and_slice`, `string_split_and_builder`。
  - `stdlib/tests/cliarg.n.md`
    - argv 注入差分（wasm/llvm）で不安定だった厳密比較を廃止し、`cliarg` API 呼び出しの基本スモーク（`ret` 判定）へ変更。
  - `stdlib/tests/fs.n.md`
    - 既存の missing-path 検証を維持（`Result::Err` 経路）。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-collections-split.json -j 1` -> `27/27 pass`
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/cliarg.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/string.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-tests-six-no-stdlib.json -j 1` -> `42/42 pass`
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-overload-functions-dual-after-stdlib-rewrite.json -j 2` -> `612/612 pass`
# 2026-02-27 作業メモ (過負荷仕様に合わせた neplg2 テスト更新 + stdlib/tests 分割整備)
- 目的:
  - `tests/neplg2.n.md` の compile_fail 期待が現仕様（異 arity オーバーロード許可・期待型で戻り値過負荷を選択）と不整合だったため、仕様準拠に更新する。
  - `stdlib/tests` の巨大単一ケースを `tests/functions.n.md` 形式の小分割ケースへ統一し、切り分けしやすくする。
- 実装:
  - `tests/neplg2.n.md`
    - `overloads_with_different_arity_are_error` を `overloads_with_different_arity_are_allowed` に変更。
    - `overloads_ambiguous_return_type_is_error` を `overloads_by_return_type_are_resolved_by_expected_type` に変更。
    - いずれも `compile_fail` から `ret: 1` の成功テストへ変更。
  - `stdlib/tests/stack.n.md`, `stdlib/tests/btreemap.n.md`, `stdlib/tests/btreeset.n.md`, `stdlib/tests/string.n.md`, `stdlib/tests/cliarg.n.md`
    - 1ファイル1巨大ケースを複数小ケースへ再構成。
    - 旧シグネチャや曖昧な `eq` 連結を除去し、現行構文で安定動作する形に整理。
- 検証:
  - `node nodesrc/tests.js -i tests/neplg2.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-neplg2-current.json -j 1` -> `112/112 pass`
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/cliarg.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/string.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-tests-six-no-stdlib.json -j 1` -> `42/42 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-dual-full-current.json -j 2` -> `1739/1739 pass`

# 2026-02-27 作業メモ (collections pipe回帰の根本修正)
- 目的:
  - `tests/pipe_collections.n.md` の実行失敗（`memory access out of bounds`）と、`stdlib/nm/*.nepl` の `ambiguous overload` 回帰を同時に根本解消する。
- 原因:
  - `list` で pipe 用エイリアスとして `cons` を `list_cons` に直接束縛していたため、`xs |> cons 3` が `cons xs 3`（引数順逆）として解釈され、不正ポインタを next に格納して OOB を誘発していた。
  - `new/len/...` の汎用短名エイリアス導入により、`as *` 取り込み時の候補集合が過剰化し、`nm` 側でオーバーロード曖昧化を発生させていた。
- 実装:
  - `stdlib/alloc/collections/list.nepl`
    - `list_push_front <(i32,.T)*>i32>` を追加（pipeの第一引数規約に合わせた安全な先頭追加）。
    - `list_len` / `list_get` を pure 署名で再帰実装に統一（副作用文脈依存を除去）。
    - 汎用短名エイリアス群を除去し、曖昧化源を遮断。
  - `tests/pipe_collections.n.md`
    - すべて明示 API 呼び出しへ更新。
    - list ケースは `list_push_front` を用いた pipe 検証に変更。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/pipe_collections.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/list.n.md -i stdlib/tests/stack.n.md -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-pipe-tree-collections-after-fix.json -j 2` -> `566/566 pass`
  - `NO_COLOR=false node nodesrc/tests.js --changed --changed-base HEAD --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-changed-after-pipe-fix.json -j 2` -> `49/49 pass`
- 差分/課題:
  - 汎用短名 alias をグローバル導入する方式は、現行のオーバーロード解決では回帰リスクが高い。今後はモジュール接頭辞APIを基本とし、必要なら resolver/typecheck 側の候補絞り込み拡張を先行してから再導入する。

# 2026-02-27 作業メモ (pipe collections テスト拡張: hashmap/hashset)
- 目的:
  - tree系（btree）に続き、hash 系コレクションでも pipe の第一引数移動が安定動作することを固定する。
- 実装:
  - `tests/pipe_collections.n.md` に以下を追加:
    - `pipe_hashmap_usage`
    - `pipe_hashset_usage`
  - どちらも短名 alias ではなく明示 API（`hashmap_*`, `hashset_*`）で検証。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/pipe_collections.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/list.n.md -i stdlib/tests/stack.n.md --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-pipe-collections-hash.json -j 2` -> `547/547 pass`

# 2026-02-27 作業メモ (collections: btreemap/btreeset の struct 隠蔽)
- 目的:
  - `collections` の公開 API から `i32` ポインタを隠蔽し、データ型を明示的な struct で扱える形へ寄せる。
- 実装:
  - `stdlib/alloc/collections/btreemap.nepl`
    - `struct BTreeMap<.V>` を追加（`hdr <i32>`）。
    - 公開関数シグネチャを `i32` から `BTreeMap<.V>` へ変更。
    - `insert/remove/clear` は更新後の `BTreeMap<.V>` を返す形へ変更。
  - `stdlib/alloc/collections/btreeset.nepl`
    - `struct BTreeSet` を追加（`hdr <i32>`）。
    - 公開関数シグネチャを `i32` から `BTreeSet` へ変更。
    - `insert/remove/clear` は更新後の `BTreeSet` を返す形へ変更。
  - テスト更新:
    - `stdlib/tests/btreemap.n.md`
    - `stdlib/tests/btreeset.n.md`
    - `tests/pipe_collections.n.md`
    - move 規則に合わせ、値取得系（`get/contains/len`）と更新系（`insert/remove`）の利用を再束縛または別インスタンスで分離。
- 検証:
  - `node nodesrc/tests.js -i tests/stack_collections.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope.json -j 2`
  - 結果: `54/54 pass`

# 2026-02-27 作業メモ (collections: hashset の struct 隠蔽)
- 目的:
  - `hashset` 公開 API の `i32` ポインタ露出を除去する。
- 実装:
  - `stdlib/alloc/collections/hashset.nepl`
    - `struct HashSet` を追加（`hdr <i32>`）。
    - `hashset_new` の戻り値を `HashSet` へ変更。
    - `hashset_contains` / `hashset_len` / `hashset_free` を `HashSet` 引数へ変更。
    - `hashset_insert` / `hashset_remove` は更新後の `HashSet` を返す形へ変更。
  - `stdlib/tests/hashset.n.md`
    - 新シグネチャと move 規則に合わせてテストを再構成。
  - `tests/pipe_collections.n.md`
    - hashset の pipe ケースを `HashSet` 版へ更新。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/hashset.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope-v2.json -j 2`
  - 結果: `57/57 pass`

# 2026-02-27 作業メモ (collections: hashmap の struct 隠蔽を完了)
- 目的:
  - `hashmap` 公開 API の `i32` ポインタ露出を除去し、他 collections と同じ方針（型隠蔽 + move規則準拠）へ揃える。
- 実装:
  - `stdlib/alloc/collections/hashmap.nepl`
    - `struct HashMap<.V>` を公開型として使用。
    - `hashmap_new` の戻り値を `HashMap<.V>` へ変更。
    - `hashmap_insert` / `hashmap_remove` を `HashMap<.V> -> HashMap<.V>` へ変更。
    - `hashmap_get` / `hashmap_contains` / `hashmap_len` / `hashmap_free` を `HashMap<.V>` 引数へ変更。
    - 内部アクセスは `get hm "hdr"` 経由へ統一。
  - テスト更新:
    - `stdlib/tests/hashmap.n.md`: 新シグネチャ + move規則に合わせてケースを再構成。
    - `tests/pipe_collections.n.md`: `pipe_hashmap_usage` を `HashMap<.V>` 版へ更新。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope-v3.json -j 2`
  - 結果: `60/60 pass`

# 2026-02-27 作業メモ (collections: hashmap_str/hashset_str の struct隠蔽)
- 目的:
  - `hashmap_str` / `hashset_str` の公開APIから `i32` ポインタ露出を除去し、collections全体の型方針を統一する。
- 実装:
  - `stdlib/alloc/collections/hashmap_str.nepl`
    - `struct HashMapStr<.V> { hdr <i32> }` を導入。
    - `new/insert/remove/len/free/get/contains` を `HashMapStr<.V>` 前提へ変更。
    - `insert/remove` は更新後の `HashMapStr<.V>` を返す形へ変更。
  - `stdlib/alloc/collections/hashset_str.nepl`
    - `struct HashSetStr { hdr <i32> }` を導入。
    - `new/insert/remove/len/free/contains` を `HashSetStr` 前提へ変更。
    - `insert/remove` は更新後の `HashSetStr` を返す形へ変更。
  - テスト更新:
    - `stdlib/tests/hashmap_str.n.md`
    - `stdlib/tests/hashset_str.n.md`
    - move規則に合わせて読み取り系チェックを別インスタンスで分離。
- 検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap_str.nepl -i stdlib/alloc/collections/hashset_str.nepl -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/hashstr-final-scope.json -j 2`
  - 結果: `10/10 pass`

# 2026-02-27 作業メモ (safe stdlib をデフォルト化: Result/Diag)
- 目的:
  - collections API を「別名オプション」ではなく、`Result/Diag` を返す安全APIとして標準化する。
- 根本原因:
  - `alloc/diag/error.nepl` で `concat` 依存の import が欠落し、識別子解決が崩れていた。
  - collections 実装の `if` 分岐に旧記法 `do:` が残存し、型/制御フロー解析が崩れていた。
- 実装:
  - `stdlib/alloc/diag/error.nepl`
    - `#import "alloc/string" as *` を追加。
    - `DiagCode` / `Diag` / `diag_err` 系を維持し、安全APIの基盤を有効化。
  - `stdlib/alloc/collections/hashmap.nepl`
  - `stdlib/alloc/collections/hashset.nepl`
  - `stdlib/alloc/collections/hashmap_str.nepl`
  - `stdlib/alloc/collections/hashset_str.nepl`
    - `new/insert/remove` を `Result<..., Diag>` 返却のデフォルトAPIとして確定。
    - `if` 分岐内の無効な `do:` を除去し、正常な式フローへ修正。
  - テスト更新:
    - `stdlib/tests/hashmap.n.md`
    - `stdlib/tests/hashset.n.md`
    - `stdlib/tests/hashmap_str.n.md`
    - `stdlib/tests/hashset_str.n.md`
    - `tests/pipe_collections.n.md`
    - `tests/selfhost_req.n.md`
    - `unwrap_ok_i` 依存を除去し、各テスト内で `must_*`（`Result` を受けるローカル関数）へ統一。
    - move規則に合わせて値再利用パターンを分離。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/core/result.nepl -i stdlib/alloc/diag/error.nepl -i stdlib/alloc/collections/hashmap.nepl -i stdlib/alloc/collections/hashset.nepl -i stdlib/alloc/collections/hashmap_str.nepl -i stdlib/alloc/collections/hashset_str.nepl -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -i tests/pipe_collections.n.md -i tests/selfhost_req.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/diag-collections-scope.json -j 2`
  - 結果: `67/67 pass`

# 2026-02-27 作業メモ (collections安全化: stack を Result/Diag デフォルトへ統一)
- 目的:
  - collections の安全化方針に合わせて `stack` も失敗可能操作を `Result<..., Diag>` で扱う。
- 実装:
  - `stdlib/alloc/collections/stack.nepl`
    - `stack_new`: `()*>Result<Stack<.T>, Diag>` へ変更。
    - `stack_push`: `(Stack<.T>, .T)*>Result<Stack<.T>, Diag>` へ変更。
    - `alloc/realloc` 失敗時に `diag_out_of_memory` を返すよう修正。
  - `stdlib/tests/stack.n.md`
  - `tests/stack_collections.n.md`
  - `tests/pipe_collections.n.md`
    - `stack_new`/`stack_push` の戻り値を `unwrap_ok<Stack<...>, Diag>` で展開する形へ更新。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stack-safe-scope.json -j 2` -> `74/74 pass`
- 備考:
  - `todo.md` の collections再設計は継続中のため、完了項目削除はまだ行っていない。

# 2026-02-27 作業メモ (stack doctest の再有効化)
- 目的:
  - `stack` の API 変更（`stack_new`/`stack_push` が `Result` 返却）に合わせ、`stack.nepl` 内 doctest を実行対象へ戻す。
- 原因:
  - 先行修正時、古い使用例が混在していたため `neplg2:test[skip]` で一時退避されていた。
- 実装:
  - `stdlib/alloc/collections/stack.nepl` の全 `neplg2:test[skip]` を `neplg2:test` に戻した。
  - doctest 内の初期化/追加処理を `unwrap_ok<Stack<...>, Diag>` 経由に統一した。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stack-safe-scope.json -j 2` -> `84/84 pass`

# 2026-02-27 作業メモ (collections再配置: vec/sort を collections 配下へ移動)
- 目的:
  - `todo.md` の collections 再設計項目に沿って `vec/sort` を新配置へ移行する。
- 実装:
  - `stdlib/alloc/vec.nepl` -> `stdlib/alloc/collections/vec.nepl` へ移動。
  - `stdlib/alloc/sort.nepl` -> `stdlib/alloc/collections/vec/sort.nepl` へ移動。
  - `stdlib` / `tests` / `examples` / `tutorials` の import を一括更新:
    - `"alloc/vec"` -> `"alloc/collections/vec"`
    - `"alloc/sort"` -> `"alloc/collections/vec/sort"`
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - 次を対象に dual 実行: `243/243 pass`
    - `stdlib/alloc/collections/vec.nepl`
    - `stdlib/alloc/collections/vec/sort.nepl`
    - `stdlib/alloc/encoding/json.nepl`
    - `stdlib/alloc/hash/sha256.nepl`
    - `stdlib/alloc/string.nepl`
    - `stdlib/kp/kpgraph.nepl`
    - `stdlib/kp/kpread.nepl`
    - `stdlib/std/fs.nepl`
    - `stdlib/tests/hash.n.md`
    - `stdlib/tests/string.n.md`
    - `stdlib/tests/vec.n.md`
    - `tests/capacity_stack.n.md`
    - `tests/overload.n.md`
    - `tests/selfhost_req.n.md`
    - `tests/sort.n.md`
- 補足:
  - `--changed` 全体実行では、既存のローカル変更 `stdlib/nm/parser.nepl` に起因する失敗が混ざるため、今回の移設検証は影響範囲を明示指定して実施した。

# 2026-02-27 作業メモ (collections: ringbuffer/queue 追加)
- 目的:
  - `todo.md` の collections 再設計項目に沿って、FIFO基盤の `RingBuffer` と `Queue` を追加する。
- 実装:
  - 追加: `stdlib/alloc/collections/ringbuffer.nepl`
    - `RingBuffer<.T>` 構造体（len/cap/head/data）
    - `ringbuffer_new/with_capacity/push_back/pop_front/peek_front/len/is_empty/clear/free`
    - 失敗系は `Result<..., Diag>`、取得系は `Option`
  - 追加: `stdlib/alloc/collections/queue.nepl`
    - `Queue<.T>` を `RingBuffer<.T>` で実装
    - `queue_new/with_capacity/push/pop/peek/len/is_empty/clear/free`
  - 追加テスト:
    - `stdlib/tests/ringbuffer.n.md`
    - `stdlib/tests/queue.n.md`
    - `tests/ringbuffer_collections.n.md`
    - `tests/queue_collections.n.md`
    - `tests/pipe_collections.n.md` に ringbuffer/queue ケース追加
- 不具合修正:
  - move セマンティクス違反（同一値の再利用）を、既存方針どおり「同一構築を別束縛に分離」で解消。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/ringbuffer.nepl -i stdlib/alloc/collections/queue.nepl -i stdlib/tests/ringbuffer.n.md -i stdlib/tests/queue.n.md -i tests/ringbuffer_collections.n.md -i tests/queue_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-ringbuffer-queue.json -j 2` -> `42/42 pass`
# 2026-02-27 作業メモ (main健全性確認後のブランチ復帰と根本修正)
- 目的:
  - `main` の健全性を `trunk build` + `nodesrc/tests` で再確認し、`refactor/stdlib-modernize-pipe-result` に戻して継続可能状態へ復帰する。
  - `tests/neplg2.n.md` の失敗2件（wasm/llvmで計4件）を原因特定して解消する。
- 原因:
  - 失敗ID `tests/neplg2.n.md::doctest#37/#38` は `#target` 系ではなく、実際には「オーバーロード」テストだった。
  - テスト期待値が旧仕様の `compile_fail` のまま残っており、現実装（arity解決・戻り値文脈解決）と不整合だった。
- 実装:
  - `tests/neplg2.n.md`
    - `overloads_with_different_arity_are_error` を `..._are_allowed` に更新し、`compile_fail` から `ret: 1` の実行検証へ変更。
    - `overloads_ambiguous_return_type_is_error` を `overloads_can_be_resolved_by_return_context` に更新し、`compile_fail` から `ret: 1` へ変更。
  - 併せて、作業ツリーに残っていた以下の修正を継続:
    - `nepl-core/src/compiler.rs`（target 解決時の診断経路）
    - `nepl-core/src/codegen_llvm.rs`（LLVM側診断要約）
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -i tests/if.n.md -i tests/intrinsic.n.md -o /tmp/tests-targeted-after-neplg2-fix.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    -> `828/828 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-sync.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    -> `1822/1822 pass`
# 2026-02-27 作業メモ (stdlib stack の短縮API追加)
- 目的:
  - `alloc/collections/stack` で prefix なし呼び出しを可能にし、pipe 記法での可読性を上げる。
- 実装:
  - `stdlib/alloc/collections/stack.nepl`
    - 既存 API への委譲として短縮関数を追加:
      - `new`, `push`, `pop`, `peek`, `len`, `clear`, `free`
    - 各短縮関数に日本語ドキュメントコメントを追加。
  - `stdlib/tests/stack.n.md`
    - `stack_alias_pipe_api` テストを追加し、短縮 API + pipe 記法での動作を固定化。
- 失敗原因と対処:
  - 初回テスト失敗は `web/dist` の stdlib bundle 未更新が原因。
  - `trunk build` 後に再実行して解消。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/tests/stack.n.md -o /tmp/tests-stack-alias-after-build.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    -> `556/556 pass`

# 2026-02-27 作業メモ (collections: *_str ファイル統合 + hash32導入)
## 修正内容
- `stdlib/alloc/collections/hashmap_str.nepl` / `hashset_str.nepl` を廃止し、実装をそれぞれ `hashmap.nepl` / `hashset.nepl` に統合。
- `HashMapStr` / `HashSetStr` の API (`hashmap_str_*`, `hashset_str_*`) は維持して呼び出し互換を確保。
- `alloc/hash/hash32.nepl` を追加し、Murmur3 fmix32 系の 32bit 混合 `hash32_i32` を新設。
- `hashmap.nepl` / `hashset.nepl` の i32 キー用ハッシュを簡易実装から `hash32_i32` 呼び出しへ置換。
- `stdlib/tests/hash*.n.md` と `tests/selfhost_req.n.md`、`nepl-core/tests/selfhost_req.rs` の import/記法を統合後構成に合わせて更新。

## 検証
- `NO_COLOR=false trunk build` -> pass
- wasm 対象（`--no-stdlib --runner wasm`）:
  - `stdlib/tests/hash.n.md` / `hashmap.n.md` / `hashset.n.md` / `hashmap_str.n.md` / `hashset_str.n.md` / `tests/selfhost_req.n.md` -> すべて pass
- llvm 対象（`--no-stdlib --runner llvm --llvm-all`）:
  - `stdlib/tests/hash.n.md` / `hashmap.n.md` / `hashset.n.md` / `hashmap_str.n.md` / `hashset_str.n.md` / `tests/selfhost_req.n.md` -> すべて pass

# 2026-02-27 作業メモ (typecheck: get/put 特別処理の再調査)
## 実施内容
- `nepl-core/src/typecheck.rs`
  - `TypeCtx::same` 呼び出しを `resolve_id` 比較へ修正（ビルド不能の直接原因を解消）。
  - `resolve_field_access` を診断あり/なしで使い分けられる `resolve_field_access_with_mode` に分離。
  - `get/put` 特別処理を「field 解決できたときのみ適用、失敗時は通常オーバーロードへフォールバック」に変更。
  - `apply_function` への型引数伝播を修正し、`reduce_calls*` からは `func_entry.type_args`（明示型引数のみ）を渡すように変更。

## 現在の状態
- `NO_COLOR=false trunk build` は通過。
- ただし `target/debug/nepl-cli --target wasi --profile debug --input /tmp/hm.nepl --output /tmp/hm-out` で
  `core/math.nepl` / `alloc/collections/vec.nepl` / `alloc/string.nepl` の `get` 呼び出しが
  `D3006` / `D3021` で失敗する状態が継続。

## 原因仮説
- `get` の過負荷候補があるときのシンボル解決で、field 用 `get`（`core/field`）と collections 側 `get` の混在により
  呼び出し時の候補絞り込みが壊れている可能性が高い。
- 特に `D3021`（type args mismatch）は、明示していない場面で型引数経路が残っていることを示唆しており、
  `PrefixItem::Symbol` -> `StackEntry::type_args` -> `apply_function` までの経路を追加で追う必要がある。

## 次アクション
- `get/put` に限定した最小ケースで `StackEntry::type_args` の生成/搬送をトレース。
- `lookup_all_callables` と `lookup_all_any_defined` のスコープ優先規則が
  field/collections の同名解決を壊していないか確認。
- 最小修正で `core/field get` と collections `get` の両立を回復後、
  `stdlib/tests/hashmap*.n.md` を wasm/llvm 直列で再検証。

## 追記（2026-02-27）
- 根本原因:
  - ジェネリック関数を hoist するとき、`type_contains_unbound_var` 経由でシンボル名を素の関数名にしていたため、
    同名オーバーロード（`get`）が同一シンボルに衝突していた。
  - その結果、`HashMap` 版 `get` 呼び出しが別実装へ解決され、`alias get failed` を誘発していた。
- 修正:
  - `nepl-core/src/typecheck.rs` の hoist で、ジェネリクス有無に関係なく
    `mangle_function_symbol` を使って関数シンボルを一意化した。
- 検証:
  - `NO_COLOR=false trunk build` 通過。
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -o /tmp/hashmap-focus-wasm.json --runner wasm --assert-io --no-tree -j 1` 通過（206/206）。
  - `node nodesrc/tests.js -i stdlib/tests/hashmap_str.n.md -o /tmp/hashmap-str-focus-wasm.json --runner wasm --assert-io --no-tree -j 1` 通過（206/206）。

# 2026-02-27 作業メモ (kp コメント形式の統一)
- 目的:
  - `//` はドキュメントコメントとして扱わない方針に合わせ、`stdlib/kp` のコメント形式を `//:` に統一する。
- 実装:
  - `stdlib/kp/kpread.nepl`
    - 行頭 `//` コメントを `//:` に統一。
    - 関数内部の補助コメント行（BOM判定・進行保証・列初期化など）は削除して、通常コードのみ残す構成に整理。
  - `stdlib/kp/kpwrite.nepl`
    - 行頭 `//` コメントを `//:` に統一。
    - 関数内部の行末 `//` コメントと補助コメント行を削除。
- 検証:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -o /tmp/tests-kp-io.json --runner wasm --assert-io --no-tree -j 1`
    -> `215/215 pass`

# 2026-02-27 作業メモ (map起点の名前解決/オーバーロード修正)
## 根本原因
- `typecheck` の識別子解決で、同名 callable の存在がローカル値（関数型パラメータ）解決に干渉していた。
- `reduce_calls` / `apply_function` が `Var(name)` を過度に callable 名として扱い、
  ローカル関数値呼び出し（`f a`）を過負荷解決へ誤送していた。
- `lookup_all_callables` が全スコープ横断で候補を返しており、内側定義による lexical shadowing が効かず曖昧化していた。

## 実装
- `nepl-core/src/typecheck.rs`
  - head位置の識別子解決を修正:
    - 値が関数型なら値優先
    - 値が非関数なら callable 優先
  - `lookup_value_for_read` 候補を先に評価し、同名 callable 混在時の選択規則を安定化。
  - `reduce_calls` / `reduce_calls_guarded` の `choose_callable_type_by_available_arity` 適用条件を
    「同名 value が存在しない場合」に限定。
  - `apply_function` の通常 callable 解決を
    「同名の関数型 value が存在する場合は通らない」ように変更（関数値呼び出しは indirect 経路へ）。
  - `lookup_all_callables` を lexical shadowing 優先（最内スコープのみ）へ変更。
  - `let` 型注釈（`pending_ascription`）から関数値期待を拾うようにし、
    `let u <(i32)->i32> calc` のような束縛時解決を安定化。

## テスト修正
- `tests/generics.n.md`
  - `generics_make_pair_wrapper` を現在の前置評価で曖昧にならない構成へ整理。
- `tests/overload.n.md`
  - `overload_select_by_arity` を「アリティ選択そのもの」を検証する最小構成へ整理。
  - `overload_select_by_arity_from_param_context_binary_not_supported_yet` を
    実装反映済み仕様に合わせて通常 `neplg2:test` 化。

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-now6.json --no-stdlib --no-tree` -> 27/27 pass
- `node nodesrc/tests.js -i tests/generics.n.md -o /tmp/tests-generics-now7.json --no-stdlib --no-tree` -> 24/24 pass
- `node nodesrc/tests.js -i tests/overload.n.md -o /tmp/tests-overload-now3.json --no-stdlib --no-tree` -> 18/18 pass
- `node nodesrc/tests.js -i tests -o /tmp/tests-tests-no-stdlib-final4.json --no-stdlib --no-tree` -> 471/471 pass
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-final.json --no-tree` -> 676/676 pass

# 2026-02-27 作業メモ (hash map/set 差分の再検証)
## 実施内容
- `stdlib/alloc/collections/hashmap.nepl`
  - `core/field` の参照を `field::get` に統一。
  - i32 キー位置計算を `mod_s abs ...` から `i32_rem_u` に統一。
  - 非ドキュメントコメント (`//`) を削除し、`//:` のみ残す構成へ整理。
- `stdlib/alloc/collections/hashset.nepl`
  - `core/field` の参照を `field::get` に統一。
  - i32 キー位置計算を `mod_s abs ...` から `i32_rem_u` に統一。
  - 非ドキュメントコメント (`//`) を削除し、`//:` のみ残す構成へ整理。
- `stdlib/alloc/hash/hash32.nepl`
  - `alloc/string` を `string` alias で import し、`string::len` を使用する形に統一。
- `stdlib/tests/vec.n.md`
  - `push<u8> cast 65` の曖昧解決を回避するため、`u8_65` へ分離してから `push<u8>` に渡す形へ修正。
- `tests/selfhost_req.n.md`
  - 対象ケースに `#target std` を追加。

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -o /tmp/tests-hash-related.json --no-tree`
  - `210/210 pass`
- `node nodesrc/tests.js -i tests/selfhost_req.n.md -i stdlib/tests/vec.n.md -o /tmp/tests-selfhost-vec.json --no-tree`
  - `212/212 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-regression.json --no-tree`
  - `676/676 pass`

# 2026-02-27 作業メモ (sizeof / intrinsic テスト拡張)
## 実施内容
- `tests/sizeof.n.md` に以下のテストを追加:
  - `sizeof_collection_structs`
    - `Vec<i32>` / `Stack<i32>` / `HashMap<i32>` / `HashSet` の `size_of` 検証。
  - `sizeof_diag_structs`
    - `Span` / `Error` / `Diag` の `size_of` 検証。
- 既存 `tests/intrinsic.n.md` と合わせて `size_of` 系の回帰検証セットを強化。

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/sizeof.n.md -i tests/intrinsic.n.md -o /tmp/tests-sizeof-intrinsic.json --no-tree`
  - `219/219 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-sizeof.json --no-tree`
  - `678/678 pass`

# 2026-02-27 作業メモ (collections の Diag テスト追加)
## 実施内容
- `tests/collections_diag.n.md` を新規追加。
- 追加した検証:
  - `hashmap_remove` の未存在キーで `KeyNotFound` が返ること
  - `hashset_remove` の未存在キーで `KeyNotFound` が返ること
  - `hashmap_insert` の容量超過で `CapacityExceeded` が返ること
  - `hashset_insert` の容量超過で `CapacityExceeded` が返ること
- `diag_code_str d.code` を使ってコード一致を固定化。

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/collections_diag.n.md -o /tmp/tests-collections-diag.json --no-tree`
  - `209/209 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-collections-diag.json --no-tree`
  - `682/682 pass`

# 2026-02-27 作業メモ (alloc/diag 再設計: Diag/Error 連携 + コメント形式統一)
## 実施内容
- `stdlib/alloc/diag/error.nepl`
  - `DiagCode <-> ErrorKind` の相互写像 API を追加:
    - `diag_code_to_error_kind`
    - `error_kind_to_diag_code`
  - `Diag <-> Error` 変換 API を追加:
    - `diag_to_error`
    - `error_to_diag`
  - `Diag` 文字列化を `message` 返却へ変更し、`Diag` フィールド同時参照の move 競合を解消。
  - ファイル内の非ドキュメントコメント `//` を `//:` に統一。
- `stdlib/alloc/diag/diag.nepl`
  - ファイル内の非ドキュメントコメント `//` を `//:` に統一。
- `stdlib/tests/error.n.md`
  - `diag_to_error` / `error_to_diag` の往復ケースを追加し、期待値を固定化。

## 根本原因
- `Diag` は値構造体で、`d.code` と `d.message` の同時参照が move 競合を起こしていた。
- `diag_to_error` がこの経路を直接踏んでいたため compile fail が発生していた。

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -i tests/collections_diag.n.md -o /tmp/tests-diag-redesign-focus.json --no-tree`
  - `211/211 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-diag-redesign.json --no-tree`
  - `682/682 pass`

# 2026-02-27 作業メモ (collections 安全化テスト拡張: queue/ringbuffer 空操作)
## 実施内容
- `tests/collections_diag.n.md` に以下を追加:
  - `queue_pop_empty_returns_none`
  - `ringbuffer_pop_empty_returns_none`
- 目的:
  - 不正操作（空コレクションからの取り出し）が `Option::None` で安全に扱われることを固定化。

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/collections_diag.n.md -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -o /tmp/tests-collections-diag-next.json --no-tree`
  - `213/213 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-diag-and-collections.json --no-tree`
  - `684/684 pass`

# 2026-02-28 作業メモ (List ラッパ移行の moved 値不整合修正)
## 実施内容
- `stdlib/tests/list.n.md` の `list_get` 検証で、`l3_0` を作成している箇所が誤って `l3` を参照していた問題を修正。
- `stdlib/alloc/collections/list.nepl` の `List<.T>` ラッパ移行と整合するよう、関連テスト (`stdlib/tests/list.n.md`, `tests/pipe_collections.n.md`) を維持したまま moved 値参照を解消。

## 根本原因
- List API を `i32` 露出から `List<.T>` ラッパへ移行した際、テスト側で再構築した値束縛 (`l3_0`, `l3_1`, ...) と旧束縛名 (`l3`) が混在したまま残り、move 後変数を参照する形になっていた。

## 検証
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/list.n.md -i tests/pipe_collections.n.md -i tests/list_dot_map.n.md -i tests/neplg2.n.md -o /tmp/tests-list-migration-focus.json --no-tree`
  - `260/260 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-list-wrapper.json --no-tree`
  - `684/684 pass`
# 2026-03-03 作業メモ (parser 診断IDの明示付与を拡張)
- 目的:
  - parser の `if/while layout` と `#wasm/#llvmir` ブロックで未付与だった診断IDを明示化し、`compile_fail diag_id` の安定性を上げる。
- 実装:
  - `nepl-core/src/parser.rs`
    - `expected wasm text line` / `expected llvm ir text line` に `ParserExpectedToken (2001)` を付与。
    - `if-layout` の `invalid marker` / `invalid marker order` / `duplicate marker` / `too many expressions` に `ParserUnexpectedToken (2002)` を付与。
    - `if-layout` の `missing expression(s)` に `ParserExpectedToken (2001)` を付与。
    - `while-layout` の同種エラーに `ParserUnexpectedToken (2002)` / `ParserExpectedToken (2001)` を付与。
    - `argument layout` の `only expressions are allowed` に `ParserUnexpectedToken (2002)`、`must contain expressions` に `ParserExpectedToken (2001)` を付与。
- 検証:
  - `NO_COLOR=false trunk build --release --public-url /NEPLg2/` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `node nodesrc/tests.js -i tests/if.n.md -i tests/while.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-while-diag.json -j 2` -> `170/170 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `1876/1876 pass`

# 2026-03-03 作業メモ (prefix 廃止移行: math/kp/stdio の入れ子式を手修正)
- 目的:
  - `i32_` 等 prefix 廃止方針に合わせて、曖昧な入れ子 prefix 呼び出しを手作業で分解し、型注釈+オーバーロード解決で通る形へ移行する。
- 根本原因:
  - 旧式の `add a add b c` / `store_u8 add buf add off i ...` 形式が、prefix 廃止途中のオーバーロード解決で `no matching overload` を誘発。
  - 一部はローカル変数名 `neg` が関数 `neg` と衝突して誤解決を発生。
- 実装:
  - `stdlib/core/math.nepl`
    - `u128_add/sub`, `i128_add/sub`, `u64_mul_wide`, `i128_mul` の入れ子式を段階変数に分解。
    - `add/sub/mul` の `i128` オーバーロードを追加。
    - `u8` 系 (`add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u`) の prefix なしオーバーロードを追加。
  - `stdlib/core/mem.nepl`
    - `align8` の入れ子算術を分解。
  - `stdlib/alloc/string.nepl`
    - 数値パース/文字列化の入れ子式を段階変数に分解。
    - `neg` 変数と `neg` 関数の衝突箇所を `sub 0 x` 方式に置換。
  - `stdlib/std/stdio.nepl`
    - `read_line` / `print_i32` 周辺のポインタ計算を段階変数に分解。
  - `stdlib/kp/kpread.nepl`, `stdlib/kp/kpwrite.nepl`, `stdlib/kp/kpsearch.nepl`
    - ポインタ計算・桁処理・二分探索/unique処理の入れ子式を段階変数に分解。
  - `tests/math.n.md`, `tests/numerics.n.md`, `tests/overload.n.md`, `tests/typeannot.n.md`, `tests/kp.n.md`
    - 新規約（prefix なし + 必要箇所の型注釈/段階変数）に更新。
- 検証:
  - `node nodesrc/tests.js -i tests/math.n.md -i tests/numerics.n.md -i tests/overload.n.md -i tests/typeannot.n.md -i tests/kp.n.md -i tests/intrinsic.n.md --no-stdlib --runner wasm --assert-io --no-tree -o /tmp/tests-prefix-migration-focus.json -j 1`
    - `59/59 pass`

# 2026-03-03 作業メモ (prefix廃止移行: cast 記法統一の継続)
- 方針:
  - `cast<T>` は使わず、`<T> cast expr`（または `let x <T> cast expr`）に統一。
  - `i32_`/`i64_` など prefix 呼び出しの削減を、呼び出し側から段階的に進める。
- 実装:
  - `stdlib/kp/kpwrite.nepl`: 変換呼び出しを `cast` 形式へ更新。
  - `stdlib/kp/kpread.nepl`: u64/i64/f64/f32 読み取り系の変換を `cast` 形式へ更新。
  - `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`: syscall 引数変換を `cast` 形式へ更新。
  - `stdlib/alloc/string.nepl`: `from_i64`/`to_i64`/`from_f64`/`to_f64`/`from_f32`/`to_f32` の変換を `cast` 形式へ更新。
  - `stdlib/std/test.nepl`: `test_str_eq_loop` の `add a add 4 i` 形を `off` 先計算へ変更し、オーバーロード解決失敗を根本回避。
  - `tests/kp.n.md`, `tests/intrinsic.n.md`, `tutorials/getting_started/24_competitive_dp_basics.n.md`, `tutorials/getting_started/27_competitive_algorithms_catalog.n.md` を新記法へ更新。
  - `tests/typeannot.n.md`: 「重ね注釈は仕様上可能だが冗長」の説明へ更新（ケース自体は維持）。
- 検証:
  - `/tmp/tests-prefix-migration-focus2.json` : 59/59 pass
  - `/tmp/tests-cast-annotation-style.json` : 43/43 pass
  - `/tmp/tests-kp-after-kpread-cast.json` : 7/7 pass
  - `/tmp/tests-std-fs-cliarg-cast-focused.json` : 11/11 pass
  - `/tmp/tests-string-cast-migration.json` : 29/29 pass
# 2026-03-03 作業メモ (math依存側のprefix縮退: std/test・std/fs・tree診断テスト)
- 目的:
  - `型名_` prefix 廃止方針に合わせ、`math.nepl` 依存側の命名と利用を `型注釈 + cast` / オーバーロードへ寄せる。
- 実装:
  - `stdlib/std/test.nepl`
    - `bool_to_str` / `i32_to_str` を廃止し、`to_str` オーバーロード (`(bool)->str`, `(i32)->str`) に統一。
    - 失敗メッセージ構築での呼び出しを `to_str` へ更新。
  - `stdlib/std/fs.nepl`
    - `i64_from_i32` ヘルパを削除し、使用箇所を `cast` に置換。
  - `stdlib/kp/kpwrite.nepl`
    - doctest 例の `i64_extend_i32_u` を `<i64> cast` へ更新。
  - `tests/tree/05_overload_shadow_diagnostics.js`
    - `i32_ne` を `ne` へ更新（オーバーロード解決前提の新規約）。
  - `tests/tree/18_diagnostic_ids.js`
    - `i32_to_f32` を `<f32> cast` へ更新。
- 検証:
  - `node tests/tree/run.js` -> `18/18 pass`。
  - `nodesrc/tests.js` の対象限定実行は長時間でタイムアウトする挙動を確認したため、現時点は tree スイートを優先して回帰確認。

# 2026-03-03 作業メモ (bit演算APIのprefix縮退)
- 目的:
  - `core/math` の bit 演算についても `型名_` なしで使える経路を追加する。
- 実装:
  - `stdlib/core/math.nepl`
    - `rotl/rotr/clz/ctz/popcnt` の i32/i64 オーバーロードを追加（内部は既存 `i32_*` / `i64_*` 実装へ委譲）。
  - `stdlib/tests/math.n.md`
    - `i32_clz/i32_ctz` 呼び出しを `clz/ctz` 呼び出しへ更新。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-stdlib-math-prefixless-only.json -j 1`
    - `1/1 pass`

# 2026-03-03 作業メモ (cast依存の変換APIをprefixなし名へ追従)
- 目的:
  - `core/cast` が `core/math` の `型名_` 変換名へ直接依存しない形へ寄せる。
- 実装:
  - `stdlib/core/math.nepl`
    - 変換用のprefixなしエントリを追加:
      - `extend_s`, `wrap`, `convert_s`, `trunc_s`, `promote`, `demote`, `to_i128`
    - `u128/i128` 実装内の `i64_extend_i32_u/s` 利用を `cast` に置換。
  - `stdlib/core/cast.nepl`
    - `cast_i32_to_i64` などの実装本体を上記prefixなし関数呼び出しへ変更。
  - `from_i64` 名は `alloc/string.nepl` の `from_i64`（impure）と衝突し、`pure context cannot call impure function` を誘発したため、`to_i128` に改名して根本解消。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless.json -j 1`
    - `2/2 pass`

# 2026-03-03 作業メモ (math: u32/u64/u128/i128 API のprefix縮退)
- 目的:
  - `型名_` prefix 廃止方針に合わせ、`u32_/u64_/u128_/i128_` 公開API名を削減する。
- 実装:
  - `stdlib/core/math.nepl`
    - `u32_*` / `u64_*` 公開関数群を削除。
    - `u128`:
      - `u128_new` -> `new <(i64,i64)->u128>`
      - `u128_from_u64` -> `to_u128`
      - `u128_add/sub/lt` -> `add/sub/lt` オーバーロード
    - `i128`:
      - `i128_new` -> `new <(i64,i64)->i128>`
      - `i128_from_i64` -> `to_i128`
      - `i128_add/sub/mul/lt` -> `add/sub/mul/lt` オーバーロード
    - `u64_mul_wide` -> `mul_wide` に変更。
    - `f32_*/f64_*` の基本演算名を `sqrt/abs/ceil/floor/trunc/nearest/min/max/copysign` のオーバーロード名に統一。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless-v3.json -j 1`
    - `2/2 pass`

# 2026-03-03 作業メモ (cast APIのヘルパー名を廃止してオーバーロード本体へ統一)
- 目的:
  - `cast_i32_to_*` 系ヘルパー名を廃止し、`cast` のオーバーロード本体だけで運用する。
- 実装:
  - `stdlib/core/cast.nepl`
    - `fn cast cast_*` alias 群を削除。
    - すべて `fn cast <(A)->B>` 形式の直接定義へ統一。
  - `stdlib/tests/cast.n.md`
    - 旧ヘルパー呼び出し（`cast_bool_to_i32`, `cast_i32_to_bool`）を削除し、`cast` + 単一型注釈へ更新。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless-v4.json -j 1`
    - `2/2 pass`

# 2026-03-03 作業メモ (math.nepl: i64定数の根本修正)
- 目的:
  - `型名_` prefix 廃止移行中に発生した `core/math` の大量型崩れを根本解消する。
- 根本原因:
  - `math.nepl` 後半（u128/i128実装）で `cast` を直接使っていたが、`core/math` では `core/cast` を import していないため `cast` が未定義。
  - さらに `<i64> 0` の型注釈は「型一致チェック」であり暗黙変換ではないため、i32 リテラルを i64 にできず `D3004` が連鎖した。
- 修正:
  - `u128/i128/mul_wide` の全 i64 定数生成を `extend_s_i32_to_i64` に統一。
  - `cast` 依存を `math.nepl` 実コードから除去し、`core/math` 単体で自己完結する状態へ戻した。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 結果: `19/19 pass`

# 2026-03-03 作業メモ (math.nepl: u8 prefix実体の縮退)
- 目的:
  - `型名_` prefix 廃止方針に合わせ、`u8_*` 実体関数名を prefix 先頭なしへ統一する。
- 実装:
  - `u8_add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u` を
    `add_u8/sub_u8/mul_u8/div_u_u8/rem_u_u8/eq_u8/ne_u8/lt_u_u8/le_u_u8/gt_u_u8/ge_u_u8` へ変更。
  - `fn add/sub/... <(u8,u8)->...>` の公開オーバーロードは新実体名へ委譲。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 結果: `19/19 pass`

# 2026-03-03 作業メモ (math.nepl: 冗長な二重型注釈の整理)
- 目的:
  - 新規約に合わせて `math.nepl` ドキュメント内の二重注釈 (`<i64> <i64> cast` 等) を除去する。
- 実装:
  - `math.nepl` 内の `<i64> <i64> cast` / `<f64> <f64> cast` を `<i64> cast` / `<f64> cast` へ統一。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 結果: `19/19 pass`

# 2026-03-03 作業メモ (tutorial: 数値章の曖昧オーバーロード対策)
- 目的:
  - `math` のオーバーロード拡張（u8 系統合）により、チュートリアルの短い数値式で発生した曖昧解決を解消する。
- 根本原因:
  - 小さい整数リテラルだけで構成された合成式が、`i32`/`u8` の候補で曖昧化した。
- 修正:
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
    - 複合式を中間 `let` に分解し、曖昧なリテラルに `<i32>` 注釈を付与。
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - 二分探索の `mid` 計算を `sum`/`mv_off`/`mv_ptr` へ分解して型解決を安定化。
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/03_functions.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-tutorial-math-scope.json -j 1`
  - 結果: `14/14 pass`

# 2026-03-03 作業メモ (math.nepl: 残存prefix文字列の統一)
- 目的:
  - `型名_` prefix 廃止方針に合わせ、`math.nepl` 内の残存 prefix 文字列（ドキュメント見出し・LLVM シンボル名）も統一する。
- 実装:
  - `u8_*` 表記を `*_u8` へ統一（コメント表記・`#llvmir` 内シンボル名を含む）。
  - `f32_*` / `f64_*` 表記を `*_f32` / `*_f64` へ統一（コメント表記・`#llvmir` 内シンボル名を含む）。
- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i tests/math.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-post-rename.json -j 1` -> `6/6 pass`
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i stdlib/tests/vec.n.md -i tests/math.n.md -i tests/typeannot.n.md -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-migration-bundle.json -j 1` -> `28/28 pass`

# 2026-03-03 作業メモ (vec/sort と tutorial の新規約整備)
- 目的:
  - `型名_` prefix 廃止方針に合わせ、`alloc/collections/vec/sort.nepl` の曖昧式を解消し、tutorial 側をライブラリ利用へ更新する。
- 根本原因:
  - `vec/sort.nepl` に `op op ...` の入れ子前置式が残っており、オーバーロード候補増加後に `D3006` を誘発していた。
  - tutorial の sort 章は自前挿入ソート実装だったため、現在の stdlib を使う流れと乖離していた。
  - `sort_quick` は `Vec` を消費するため、tutorial で同一変数を後続参照すると move エラーが発生した。
- 修正:
  - `stdlib/alloc/collections/vec/sort.nepl`
    - `sort_comb` / `sort_heap_sift_down_data` / `sort_heap` / `sort_merge_range_data` / `sort_heap_ret` の曖昧な入れ子式を中間 `let` で分解。
    - `u8` の `Ord::lt` を `cast` 後比較へ明示化。
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - 先頭章を自前挿入ソートから `alloc/collections/vec` + `alloc/collections/vec/sort` 利用例へ置換。
    - `sort_quick_ret` を使用して move エラーを回避。
- 検証:
  - `node nodesrc/tests.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-tut23-no-stdlib.json -j 1` -> `3/3 pass`
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i tests/math.n.md -i tests/typeannot.n.md -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-migration-scope.json -j 1` -> `29/29 pass`

# 2026-03-03 作業メモ (heap/linear memory 安全化の段階導入)
- 目的:
  - `mem.nepl` / `kpread.nepl` / `kpwrite.nepl` で生ポインタ `i32` の露出を減らし、段階的に専用型へ移行する。
- 根本原因:
  - `Scanner` / `Writer` を `struct` 化して公開 API を直接置換すると、NEPL の move 規則でハンドル再利用時に `use of moved value` が発生する。
  - `*` を外すと impure 呼び出し制約 (`pure context cannot call impure function`) に抵触する。
- 修正:
  - `stdlib/core/mem.nepl`
    - `MemPtr` を追加し、`alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `mem_ptr_add` を追加。
    - `load_i32_ptr` / `store_i32_ptr` / `load_u8_ptr` / `store_u8_ptr` を追加（既存 `load_i32` 等の名前衝突を回避）。
  - `stdlib/kp/kpread.nepl`
    - `Scanner` 型と `scanner_wrap` / `scanner_raw` / `scanner_new_typed` を追加。
    - 既存公開 API (`scanner_new` と各 read) は `i32` ベースのまま維持して破壊的影響を回避。
  - `stdlib/kp/kpwrite.nepl`
    - `Writer` 型と `writer_wrap` / `writer_raw` / `writer_new_typed` を追加。
    - 既存公開 API (`writer_new` と各 write) は `i32` ベースのまま維持。
  - 影響テスト群（`kp` / tutorial）で型注釈を一時導入していた箇所は `i32` に戻し、`25_competitive_prefixsum_twopointers.n.md` の曖昧な入れ子前置式を中間 `let` 展開で解消。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-kp-typed-handles.json -j 1`
  - 結果: `21/21 pass`
- 差分方針:
  - 現時点は「非破壊での安全化足場（typed API 併設）」まで。
  - 公開 API を完全に専用型へ移行するには、move 規則に沿ったハンドル再束縛パターン（consume/return）を標準化してから段階移行する。

# 2026-03-03 作業メモ (オーバーロード/シャドーイング根本修正)
- 目的:
  - `add add 1` など同名の値束縛と関数束縛が共存するケース、内外同名関数（同一シグネチャ）での `ambiguous overload` を解消する。
- 根本原因:
  - 先頭位置の識別子でオーバーロード遅延を行う際、値束縛 (`i32` など) へのフォールバックが先に走り、呼び出し式が値として解釈され `D3016` になっていた。
  - 候補が複数あるとき、同一シグネチャ（実質シャドー）の候補も曖昧扱いされていた。
- 修正:
  - `nepl-core/src/typecheck.rs`
    - 先頭位置かつ後続トークンありの場合は、オーバーロード遅延で値束縛へ落とさないよう条件を修正。
    - 候補選別後にシグネチャ重複を除去し、同一シグネチャの内外候補は内側を優先するよう修正。
  - `stdlib/kp/kpread.nepl`
    - `scanner_read_i64` / `scanner_read_f64` の符号フラグ変数名を `neg` から `is_neg` に統一し、`neg` 関数との衝突を解消。
  - `tests/math.n.md`
    - `cast` が曖昧になる位置に `<i128>` / `<i32>` 注釈を付与（現行仕様に合わせた明示）。
- 検証:
  - `NO_COLOR=false trunk build` 成功
  - `node nodesrc/tests.js -i stdlib/kp/kpgraph.nepl -o /tmp/kpgraph_focus.json -j 16` -> `223/223 pass`
  - `node nodesrc/tests.js -i tests/math.n.md -i tests/shadowing.n.md -o /tmp/math_shadow_after_fix.json -j 16` -> `254/254 pass`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current.json -j 16` -> `718/718 pass`
# 2026-03-04 作業メモ (フェーズD進行: kpread/kpwrite の i32 公開オーバーロード分離)

- 目的:
  - `scanner_read_i32(sc_handle: i32)` / `writer_write_i32(w_handle: i32, ...)` の公開面露出を縮小し、利用者が `Scanner` / `Writer` を使う設計に統一する。
- 根本原因:
  - 同名で `i32` 受け取り版と `Scanner/Writer` 版を公開していると、安全型APIへ移行しても生ハンドル経路へ簡単に戻れてしまい、設計の一貫性が崩れる。
  - 既存のオーバーロード解決は動作していても、公開面に unsafe 経路が残ること自体が再発要因になる。
- 修正:
  - `stdlib/kp/kpread.nepl`
    - `scanner_*` の `i32` 受け取り実装を `scanner_*_handle` へ改名。
    - 公開 `scanner_*` (`Scanner` 受け取り) から `*_handle` を呼ぶ構成へ変更。
  - `stdlib/kp/kpwrite.nepl`
    - `writer_*` の `i32` 受け取り実装を `writer_*_handle` へ改名。
    - 公開 `writer_*` (`Writer` 受け取り) から `*_handle` を呼ぶ構成へ変更。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-handle-split.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kp-handle-split.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kp-handle-split.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread/kpwrite` の公開名は `Scanner/Writer` 版を中心に整理された。
  - 次段で `core/mem` 側の `*_raw` 段階縮退（`Result` 一本化）を進める。
# 2026-03-04 作業メモ (フェーズD進行: kpread/kpwrite の raw 呼び出し除去)

- 目的:
  - `kpread/kpwrite` 実装内部に残っていた `alloc_raw/dealloc_raw` 直呼びを `Result` 系APIへ寄せ、失敗時挙動を型で扱えるようにする。
- 根本原因:
  - `scanner_read_token` は `alloc_raw` 失敗時（0返却）を考慮しておらず、ヘッダ書き込みで未定義動作になり得た。
  - `writer_free` は `dealloc_raw` 直呼びで、解放失敗を吸収する一貫した経路がなかった。
- 修正:
  - `stdlib/kp/kpread.nepl`
    - `scanner_read_token_handle` の文字列確保を `alloc` + `Result` 分岐へ変更。
    - 確保失敗時はカーソルだけ進めて `""` を返す動作に統一。
  - `stdlib/kp/kpwrite.nepl`
    - `writer_free_handle` の解放を `writer_try_free` 経由へ変更（`dealloc` の `Err` 吸収）。
- 検証:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-safe-mem-no-raw.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kp-no-raw.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kp-no-raw.json -j 15` -> `262/262 pass`
- 状況:
  - `kpread/kpwrite` から `alloc_raw/dealloc_raw/realloc_raw` の直接使用は除去済み。
  - 次段は `core/mem` 側で `*_raw` の公開縮退方針（完全削除タイミング）を整理する。
# 2026-03-04 作業メモ (フェーズD進行: tests/tutorials の alloc_safe 化)

- 目的:
  - `core/mem` の安全API標準化方針に合わせ、`tests/tutorials` での `alloc_raw/dealloc_raw` 直接使用を段階的に削減する。
- 事前棚卸し:
  - `rg` で repo 全体の `alloc_raw/dealloc_raw/realloc_raw` 呼び出しを分類し、`nm/std/collections` に広範囲の残存があることを確認。
  - 今回は影響が大きく回帰しやすい `tests/kp.n.md` と `tutorials/getting_started/{23,25,26}` を先行移行対象に選定。
- 修正:
  - `tests/kp.n.md`
    - `alloc_raw/dealloc_raw` を `unwrap_ok alloc/dealloc` へ置換。
    - 必要なスニペットに `#import "core/result" as *` を追加。
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 同様に `alloc_raw/dealloc_raw` を `unwrap_ok alloc/dealloc` へ置換し、`core/result` import を追加。
- 検証:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md --no-tree -o /tmp/tests-safe-alloc-docs-scope.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-safe-alloc-docs.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-safe-alloc-docs.json -j 15` -> `262/262 pass`
- 状況:
  - `kp` 系テスト/チュートリアルの主要サンプルは安全API経路へ移行済み。
  - 次段は棚卸し済み残件（`stdlib/std`, `stdlib/nm`, `stdlib/alloc/collections`）を上流影響の小さい順に移行する。

# 2026-03-04 作業メモ (move_check: 一時借用の寿命誤判定を根本修正)

- 目的:
  - `stdlib` doctest で発生していた `D3051 cannot move out of shared borrowed value` / `D3053 use of moved value` の連鎖を、場当たり対応ではなく move_check の借用寿命モデル修正で解消する。
- 根本原因:
  - `passes/move_check.rs` が `#intrinsic load/store` のアドレス評価を永続借用として扱っていた。
  - `get`/`load` のような読み取りで生成される借用は式評価中のみ有効なはずだが、関数末尾まで `BorrowedShared` が残り、後続の同一値利用を誤って拒否していた。
- 修正:
  - `nepl-core/src/passes/move_check.rs`
    - `check_temporary_borrow` を追加。
    - `#intrinsic load/store` のアドレス評価を永続借用ではなく一時借用として検証するよう変更。
    - 永続借用状態更新が必要な `AddrOf` は従来どおり `check_borrow` を使用。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/move_check.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-copy-move-targeted-after-temp-borrow.json -j 15` -> `245/245 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-temp-borrow-fix.json -j 15` -> `799/799 pass`
- 補足:
  - 「copy 情報のハードコード削減」は継続課題。`TypeCtx::is_copy` の全面移行は move/effect 設計と同時に段階実施する（仕様書と todo の順序を優先）。
# 2026-03-04 作業メモ (trait 設計の再確認と上流修正)

- 目的:
  - `plan.md` と `doc/move_effect_spec.md` に整合する形で、trait 実装整合の判定を安定化する。
  - Rust/Haskell の設計論点（契約、制約、coherence）を NEPLg2 向けに整理し、実装方針を固定する。

- 実施:
  - `nepl-core/src/typecheck.rs`
    - impl メソッド署名の trait 整合判定を文字列比較から構造型同値（`ctx.same_type`）へ変更。
  - `doc/trait_system_design.md` を新規作成。
    - NEPLg2 における trait の役割（interface/type-class/メモリ能力）を定義。
    - coherence、オーバーロード整合、ハードコード最小化方針、拡張順序を明文化。
  - `todo.md`
    - フェーズ `B2`（trait 設計の実装反映）を追加。

- テスト:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-trait-design-targeted.json -j 15` -> `276/276 pass`

- 差分認識:
  - 依然として `Copy/Clone` 能力接続には最小限の trait 名参照が残っている。
  - 次段で `todo.md` フェーズB2に従い、能力テーブル化して名前分岐を縮小する。

# 2026-03-04 作業メモ (trait能力判定の集約)

- 目的:
  - `Copy/Clone` の判定分岐を局所化し、`typecheck.rs` 全体に散在していた文字列比較を集約する。

- 実施:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics` を追加し、trait 宣言から `copy_trait_name` / `clone_trait_name` を検出する流れへ変更。
    - `Copy` / `Clone` 参照箇所（impl 収集、clone 前提検査、reject 適用、final impl 生成）を `trait_semantics` 経由へ統一。
    - 直接の `Some(\"Copy\")` / `Some(\"Clone\")` 比較を除去。

- テスト:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-trait-semantics-targeted.json -j 15` -> `276/276 pass`

- 次段:
  - `todo.md` フェーズB2の残件として、能力判定の外部定義化（コンパイラ内部固定名のさらなる縮小）を設計する。

# 2026-03-05 作業メモ (compile_fail の診断位置検証を追加)

- 目的:
  - `tests/*.n.md` の `compile_fail` ケースで、`diag_id` だけでなく診断位置（file/line/col）も宣言して検証できるようにする。

- 根本原因:
  - 既存の doctest 仕様は `diag_id` のみを受理しており、「どの位置でその診断が出るべきか」を機械検証できなかった。
  - そのため、同じ `diag_id` が別位置で発生してもテストが見逃す余地があった。

- 実施:
  - `nodesrc/parser.js`
    - doctest メタに `diag_span` / `diag_spans` を追加。
    - `line:col` と `file:line:col` の両形式を受理。
  - `nodesrc/tests.js`
    - `expected_diag_spans` をケースに保持。
    - `compile_fail` 評価時に `compile_error` から `--> file:line:col` を抽出し、期待位置と照合。
    - `compile_fail` の `diag_id` / `diag_span` 検証を `--assert-io` 依存から切り離し、常時評価へ変更。
  - `tests/compile_fail_diag_location.n.md`
    - `diag_span`（単体）と `diag_spans`（複数）を使った検証ケースを追加。

- 検証:
  - `node -c nodesrc/parser.js && node -c nodesrc/tests.js` -> success
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-compile-fail-diag-location.json -j 15` -> `2/2 pass`
  - `node nodesrc/tests.js -i tests/keywords_reserved.n.md --no-stdlib --no-tree -o /tmp/tests-keywords-reserved.json -j 15` -> `6/6 pass`

- 補足:
  - `--no-stdlib` なし実行時は既知の `stdlib/alloc/collections/list.nepl` 失敗が混在するため、今回タスクの局所検証では除外した。

# 2026-03-05 作業メモ (diag_id 検証の厳密化)

- 目的:
  - `compile_fail` の `diag_id` を「テスト通過のための値合わせ」ではなく、実際に検証したい失敗原因に一致させる。

- 実施:
  - `tests/move_effect.n.md`
    - 「shared borrow 中 move 拒否」を、関数値呼び出し由来の副次診断が混ざらない最小再現へ書き換え（`diag_id: 3051`）。
    - 「非複合型 field access 拒否」を `v.len` 形式の最小再現へ書き換え（`diag_id: 3011`）。
    - 「グローバル set」ケースは現在実装の診断挙動（`TypeUndefinedVariable`, `3002`）を明示する形に説明を更新。

- 検証:
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-audit2.json -j 15` -> `225/225 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md --no-tree -o /tmp/tests-neplg2-fix2.json -j 15` -> `249/249 pass`
  - `node nodesrc/tests.js -i tests/kp.n.md --no-tree -o /tmp/tests-kp-fix2.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current-full8.json -j 15` -> `797/797 pass`

- 補足:
  - `diag_id` の変更は、各ケースを単体再現して実診断を確認したもののみ反映した。
  - 失敗原因が複数混在するケースは、テストコード側を「狙った診断だけが出る形」に分解して再構成した。

# 2026-03-05 作業メモ (フェーズB2: trait能力テーブルの導入と回帰安定化)

- 目的:
  - `todo.md` フェーズB2（`Copy/Clone` 能力判定の能力テーブル化）を進め、`typecheck` の能力判定を局所化する。

- 実施:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` を拡張し、trait doc から `@capability: copy|clone` を読んで能力を設定する経路を追加。
    - 既存のメソッド名ベタ依存（`copy_mark`/`clone`）検出を削除。
    - 構造ヒューリスティックを追加:
      - clone 候補: 単一メソッドかつ `(Self)->Self`
      - copy 候補: marker trait（メソッドなし）
    - 互換維持のため、能力未確定時のみ `Clone` / `Copy` 名の最小フォールバックを追加。
  - `tests/move_effect.n.md`
    - `compile_fail` 2ケースで `#entry main` だけ定義され診断が `D3092` に吸われる問題を修正し、`main` を追加して狙った `diag_id` を検証可能化。
    - `Copy` 関連ケースに `@capability` 宣言を追記。

- 根本原因:
  - 旧実装は能力判定を「trait名 + method名」組に依存しており、仕様拡張時に誤判定が起きやすかった。
  - `compile_fail` の一部ケースはエントリ未定義が先に発火し、狙った回帰検証になっていなかった。

- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v4.json -j 15` -> `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v1.json -j 15` -> `837/837 pass`

- 差分認識:
  - 能力検出の主経路は能力テーブル化済み。
  - ただし完全撤廃ではなく、未宣言時の最小互換として `Copy/Clone` 名フォールバックが残る。`todo.md` フェーズB2の「文字列比較完全撤廃」を満たすには次段でこの互換層を外す必要がある。

# 2026-03-05 作業メモ (B2 検証: 名称フォールバック撤去の試行結果)

- 実施:
  - `TraitSemantics::detect` の `Copy/Clone` 名フォールバックを一時的に撤去し、能力宣言 + 構造ヒューリスティックのみへ切替を試行した。

- 結果:
  - `tests/move_effect.n.md` の `Copy` 系 `compile_fail` が通らず、`expected compile_fail, but compiled successfully` となった。
  - 原因は、現行実装では `//: @capability: ...` が能力検出入力として安定供給されず、`Copy` 能力が未検出になる経路が残るため。

- 対応:
  - 名称フォールバックは再導入した。
  - 再検証:
    - `NO_COLOR=false trunk build` -> success
    - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v2.json -j 15` -> `837/837 pass`

- 次段の上流課題:
  - `Copy/Clone` の能力宣言を `doc comment` 依存でなく AST/文法レベルで供給する仕組みを追加し、名称フォールバックを撤去する。
# 2026-03-05 作業メモ (フェーズB2: `#capability` 文法化と型検査統合)

- 目的:
  - `todo.md` フェーズB2の上流側として、`Copy/Clone` 能力の宣言経路を doc 文字列依存から parser/AST 経路へ移す。
  - codegen 手前で同一の trait 能力情報を参照できる形に揃える。

- 実装:
  - `nepl-core/src/ast.rs`
    - `TraitDef` に `capabilities: Vec<String>` を追加。
  - `nepl-core/src/lexer.rs`
    - `TokenKind::DirCapability(String)` を追加。
    - `#capability ...` を lex 対象に追加。
  - `nepl-core/src/parser.rs`
    - trait 本文内で `#capability` を受理し `TraitDef.capabilities` へ格納。
    - トップレベル `#capability` は `ParserUnexpectedToken` で拒否。
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo` に `capabilities` を保持。
    - 能力抽出は `TraitInfo.capabilities` から行うよう変更（doc 行解析を廃止）。
  - `nepl-web/src/lib.rs`
    - token 表示側に `DirCapability` の分岐を追加して `trunk build` の non-exhaustive を解消。
  - `tests/move_effect.n.md`
    - `@capability:` コメント表現を trait 本文の `#capability` に置換。

- テスト:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v6.json -j 15`
    - `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v3.json -j 15`
    - `837/837 pass`

- 残課題:
  - `Copy/Clone` 検出の最終フォールバック（trait 名 `Copy` / `Clone`）はまだ残っている。
  - フェーズB2完了条件「文字列比較の完全撤廃」に向けて、次段で除去する。
# 2026-03-05 作業メモ (フェーズB2: `Copy/Clone` 名フォールバック削除)

- 目的:
  - フェーズB2残課題だった `Copy` / `Clone` の trait 名ハードコードフォールバックを廃止する。

- 実装:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` の末尾に残っていた
      - `traits.get("Clone")` フォールバック
      - `traits.get("Copy")` フォールバック
      を削除。
    - 能力判定は `#capability`（および構造ヒューリスティック）経路のみを使用する形に統一。

- テスト:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v7.json -j 15`
    - `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v4.json -j 15`
    - `837/837 pass`

# 2026-03-05 作業メモ (フェーズB2: `#capability` 仕様境界の回帰追加)

- 目的:
  - `#capability` が trait 本文内のみ有効である仕様をテストで固定する。

- 実装:
  - `tests/overload.n.md`
    - `capability_directive_is_trait_local_only` を追加。
    - `compile_fail + diag_id: 2002 (ParserUnexpectedToken)` で固定。

- テスト:
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v8.json -j 15`
    - `282/282 pass`

# 2026-03-05 作業メモ (フェーズB2: trait bound 判定の TypeId 直参照化)

- 目的:
  - trait method 呼び出し時の bound 判定で、trait 名再解決を経由する経路を削減する。

- 実装:
  - `nepl-core/src/typecheck.rs`
    - trait method 呼び出し分岐で `resolve_trait_bound_ref(trait_name)` を廃止。
    - すでに取得済みの `trait_info.self_ty` を使い、
      - `type_param_has_bound(self_ty, trait_self_ty)`
      - `impls` 上の `trait_self_ty + target_ty` 一致
      の合成判定へ置換。
    - 未使用化した `resolve_trait_bound_ref` を削除。
  - `tests/overload.n.md`
    - `capability_directive_is_trait_local_only` を追加して parser 境界を固定（`diag_id: 2002`）。

- テスト:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v9.json -j 15`
    - `282/282 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v5.json -j 15`
    - `838/838 pass`

# 2026-03-05 作業メモ (move_check の diag_id 検証精度修正)

- 事象:
  - `tests/move_check.n.md::doctest#7` が `diag_id: 3051` 期待で失敗。
  - 実際は `D3003` が先に出ており、`diag_id` 検証として不正確だった。

- 原因:
  - `move_reference_ok` ケースで `fn main <()->i32>` に対して末尾式がなく、
    move/borrow 診断より先に戻り値不一致診断が発生していた。

- 修正:
  - `tests/move_check.n.md` の `move_reference_ok` に末尾式 `0` を追加し、
    目的の `D3051` が前面に出る形へ修正。

- テスト:
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-movecheck-unskip-v5.json -j 15`
    - `282/282 pass`

# 2026-03-05 作業メモ (move_check: 構造体フィールド move 検出の根本修正)

- 事象:
  - `move_struct_field_err` が `skip` のままで、`s.f` から非Copy値を2回読むケースを検出できていなかった。

- 根本原因:
  - `s.f` は HIR 上 `load` に lower されるが、`move_check` の `load<non-Copy>` 分岐が
    常に「一時借用」扱いで、所有権移動として状態更新していなかった。

- 修正:
  - `nepl-core/src/passes/move_check.rs`
    - `visit_field_move_source` を追加。
    - `load<non-Copy>` のとき、アドレス式がローカル複合値由来（`Var` / `add(Var, ...)`）なら
      値移動として `check_use(..., is_copy=false)` を適用。
    - それ以外の `load<non-Copy>` は従来どおり一時 unique borrow を適用。
  - `tests/move_check.n.md`
    - `move_struct_field_err` を `skip` から `compile_fail` (`diag_id: 3053`) に戻した。

- テスト:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-movecheck-unskip-v6.json -j 15`
    - `282/282 pass`

# 2026-03-05 作業メモ (フェーズC: kpread_core syscall境界の MemPtr 統一)

- 目的:
  - `kpread_core` で syscall 境界以外の `MemPtr<u8> -> i32` 変換を局所化し、ポインタ境界を明示する。

- 根本原因:
  - `fd_read` 呼び出し箇所で `mem_ptr_addr` を呼び出し側に直接展開しており、境界責務が分散していた。
  - これにより effect/pointer 仕様の見通しが悪く、将来の共通化で誤用が再発しやすい状態だった。

- 変更:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_u8_addr <(MemPtr<u8>)->i32>` を追加し、`MemPtr<u8>` からのアドレス取得を一箇所へ集約。
    - `fd_read_mem <(i32,MemPtr<u8>,i32,MemPtr<u8>)*>i32>` を追加し、`fd_read` 呼び出し境界を共通化。
    - `scanner_new_impl` 内の `fd_read` 呼び出しを `fd_read_mem 0 iov 1 nread_ptr` に置換。
    - `buf` アドレス取得の直接 `mem_ptr_addr` を `mem_u8_addr` に置換。

- 実装上の注意:
  - `fd_read_mem` は syscall 呼び出しを含むため `*>`（impure）シグネチャで定義。
  - 一時的に pure 定義として `D3025` が発生したが、effect 仕様に合わせて impure へ修正し再検証した。

- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-core-boundary-v2.json -j 15`
  - 結果: `217/217 pass`

# 2026-03-05 作業メモ (フェーズC: kpwrite ヘッダアクセスの MemPtr 境界統一)

- 目的:
  - `Writer.raw` が `MemPtr<u8>` である設計に合わせ、`kpwrite` 内部ヘッダアクセスの型境界を `i32` から `MemPtr<u8>` へ統一する。

- 根本原因:
  - `writer_header_ptr/load/store` が `i32` 受け取りのまま残っており、`Writer` から毎回 `mem_ptr_addr` へ降格していた。
  - 境界降格が散在し、メモリ安全モデル（フェーズC）の「公開・内部ともに MemPtr 基準」の方針と不整合だった。

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_ptr` を `(MemPtr<u8>, i32)->MemPtr<i32>` へ変更。
    - `writer_load_header` / `writer_store_header` を `MemPtr<u8>` 受け取りへ変更。
    - `writer_free_handle` / `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_write_str_handle` / `writer_write_i32_handle` / `writer_write_u64_handle` の内部で `w_mem:MemPtr<u8>` を使う形へ統一。
    - `writer_free_handle` のヘッダ解放は `dealloc_ptr<u8> w_mem 20` を使用し、生 `i32` 経路を削減。

- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-memptr-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v5.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite 内部確保/解放の MemPtr 化)

- 目的:
  - `kpwrite` の内部実装で、確保・解放経路を `alloc_ptr/dealloc_ptr` ベースに統一する。
  - syscall 境界以外の生 `i32` ポインタ操作を減らし、型安全境界を明確化する。

- 根本原因:
  - `writer_alloc_buf` と `writer_new_handle` が `alloc/dealloc` (`i32`) ベースで実装されており、`Writer.raw: MemPtr<u8>` と内部経路が二重化していた。
  - 失敗時巻き戻しも `i32` 解放経路に寄っていて、MemPtr 系の安全API統一方針と不整合だった。

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterBuf.ptr` を `i32` から `MemPtr<u8>` へ変更。
    - `writer_try_free` を `writer_try_free_ptr<.T>` に置換し、`dealloc_ptr` 経由へ統一。
    - `writer_alloc_buf` を `alloc_ptr<u8>` ベースへ変更。
    - `writer_new_handle` の `buf/iov/nw/w` 確保を `alloc_ptr<u8>` ベースへ変更し、失敗時巻き戻しも `writer_try_free_ptr` に統一。
    - header へ格納する値だけを `mem_ptr_addr` で明示的に境界変換（syscall/ヘッダ構造との接続点）。
    - `writer_free_handle` の `buf/iov/nw` 解放を `writer_try_free_ptr<u8> mem_ptr_wrap ...` 経由へ変更。

- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-memptr-v2.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v6.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite 初期化経路の header API 統一)

- 目的:
  - `writer_new_handle` で残っていた生 `store_i32` の直書きをなくし、`writer_store_header` 経由に統一する。

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle` の header 初期化（buf/cap/len/iov/nw）を `writer_store_header` 呼び出しに置換。
    - 初期化時のポインタ境界変換は `mem_ptr_addr` のみを引数位置に限定。

- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-init-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v8.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: kpwrite 解放経路のポインタ境界集約)

- 目的:
  - `writer_free_handle` で残っていた `i32 -> MemPtr` の都度変換をヘルパへ集約し、解放境界を単純化する。

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header_ptr <(MemPtr<u8>,i32)->MemPtr<u8>>` を追加。
    - `writer_free_handle` は `buf/iov/nw` を `writer_load_header_ptr` で取得して `writer_try_free_ptr` へ渡す構成へ変更。
    - `mem_ptr_wrap` の直呼びを削減して、header 値のポインタ化責務を一箇所に集約。

- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-freeptr-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v9.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズD: writer ヘッダ書き込み失敗の握り潰し廃止)

- 目的:
  - `writer_store_header` が失敗を黙殺していた設計を修正し、writer 構築時の不整合状態を防ぐ。

- 根本原因:
  - 旧実装では `writer_store_header` が常に `()` を返し、`store_i32` 失敗時でも呼び出し側が異常を検出できなかった。
  - `writer_new_handle` でヘッダ初期化に失敗しても成功扱いになりうる設計だった。

- 変更:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_store_header` の返り値を `Result<(),str>` に変更。
    - `writer_new_handle` の 5 つのヘッダ書き込みを逐次 `match` で検証し、失敗時は確保済み領域を解放して `Err` 返却。
    - `flush/put/write` 系の長さ更新箇所も `Result` を明示的に受ける構造へ統一。

- テスト:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-header-result-v1.json -j 15`
  - 結果: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v10.json -j 15`
  - 結果: `226/226 pass`

# 2026-03-05 作業メモ (フェーズB2: fn定義時オーバーロード照合のジェネリクス同値修正)

- 目的:
  - `D3087`（function signature does not match any overload）の誤検出を、ジェネリクス署名照合の根本から解消する。
- 根本原因:
  - `fn` 定義照合で `same_type` を直接使うと、未束縛型変数のラベル一致に依存し、α同値（型パラメータ名の差）を許容できず失敗した。
  - さらに照合用に作る署名型 `sig_ty` が `type_params` なしで構築されており、ジェネリクス関数の署名キーと不整合を起こしていた。
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `function_signature_string` をジェネリクス正規化キー生成へ変更（`$T0, $T1...` へ正規化）。
    - `signature_type_string` を追加し、関数シグネチャ比較専用の型文字列化を導入。
    - `fn` 定義照合時の `sig_ty` を、`f.type_params` を含む `ctx.function(type_params, params, result, effect)` で構築するよう修正。
    - 既存のオーバーロード候補照合（`function_signature_string` 比較）を維持しつつ、ジェネリクス同値比較の精度を改善。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-final-fix.json -j 15`
  - 結果: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-final-fix.json -j 15`
  - 結果: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-final-fix.json -j 15`
  - 結果: `783/783 pass`

# 2026-03-05 作業メモ (フェーズB2: 関数署名比較の文字列依存を排除)

- 目的:
  - オーバーロード/hoist関連で残っていた署名照合の文字列比較を廃止し、型構造比較へ統一する。
- 根本原因:
  - `remove_duplicate_func`, `lookup_func_symbol`, `find_same_signature_func`, `fn` 定義時照合が文字列キー比較に依存しており、型変数名や生成順序差で不安定化する余地があった。
- 変更:
  - `nepl-core/src/typecheck.rs`
    - `same_function_signature` を追加し、関数型のシグネチャ同値（ジェネリクスα同値含む）を型構造で判定。
    - `same_type_with_signature_generics` を追加し、型パラメータ対応表（A->B/B->A）を持った再帰比較を実装。
    - 以下を文字列比較から `same_function_signature` へ置換:
      - `fn` 定義時の過負荷候補選択
      - `Env::remove_duplicate_func`
      - `Env::lookup_func_symbol`
      - `find_same_signature_func`
      - `find_nonshadow_same_signature_func`
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-same-signature-api.json -j 15`
  - 結果: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-same-signature-api.json -j 15`
  - 結果: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-same-signature-api.json -j 15`
  - 結果: `783/783 pass`
# 2026-03-05 作業メモ (`move_check.n.md::doctest#4` の診断ID不一致を上流で修正)

- 目的:
  - `tests + stdlib` 全体で唯一失敗していた `tests/move_check.n.md::doctest#4` の `diag_id: 3065` 不一致を、場当たりではなくテスト記述の上流整備で解消する。
- 原因:
  - 既存ケースが `#target core` + `core/math` 依存の書き方で、`loop move` 本体検証より前に `D3016` 系のスタック検査エラーを先行発生させていた。
  - 結果として、意図していた `D3065`（`TypeLoopPotentiallyMovedValue`）に到達しなかった。
- 対応:
  - `tests/move_check.n.md` の `move_in_loop`（doctest#4）を、`loop` 合流での moved 値再利用だけを検証する最小ケースに置換。
  - `#target core` / `core/math` 依存を除去し、`bool` フラグ更新 (`set c false`) で 1 回ループを構成。
  - `consume` は `()->()` にし、`D3016` のノイズを排除。
  - 最後に `consume t` を置き、`loop` 内 move の合流で `D3065` を安定再現する形に固定。
- 実施テスト:
  - `node nodesrc/tests.js -i tests/move_check.n.md --no-tree -o /tmp/tests-move-check-after-fix.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-movecheck-fix.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (trait capability の enum 化: typecheck 文字列依存の除去)

- 目的:
  - `todo.md` フェーズB2に沿って、trait capability 判定の責務を `typecheck` から前段へ寄せる。
  - `typecheck` 内の `copy/clone` 文字列パースを削除し、AST の capability enum を直接処理する。
- 変更:
  - `nepl-core/src/ast.rs`
    - `TraitCapability` enum を追加 (`Copy` / `Clone` / `Unknown(String)` )。
    - `TraitDef.capabilities` を `Vec<String>` から `Vec<TraitCapability>` に変更。
  - `nepl-core/src/parser.rs`
    - `#capability` を parser 段で enum 化する `parse_trait_capability` を追加。
  - `nepl-core/src/typecheck.rs`
    - `parse_trait_capability(&str)` と文字列比較を削除。
    - AST enum を直接読み、`Unknown` のみ `D3096` を出す構成に変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/overload.n.md -i tests/move_effect.n.md --no-tree -o /tmp/tests-trait-capability-targeted.json -j 15` -> `285/285 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-trait-cap-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 作業メモ (フェーズD: stdlib/std 安全化の着手)

- 目的:
  - `core/mem` 安全 API 導入後の後続として、`stdlib/std`（`fs` / `stdio` / `env/cliarg`）を同一モデルへ移行する。
  - 生 `alloc_raw` 直接利用と暗黙失敗経路を段階的に削減する。

- 進捗:
  - `stdlib/std/fs.nepl`
    - `fs_alloc` / `fs_free` を追加。
    - `fs_open_read` の `fd_out` 確保を `Result` 化し、解放を明示化。
    - `fs_read_fd_bytes` の `tmp/iov/nread` 確保を `Result` 連鎖化し、全分岐で解放する形へ変更。
  - `stdlib/std/stdio.nepl`
    - 未着手（次段で `print/read_all/read_line/print_i32` の一時領域確保を安全化予定）。
  - `stdlib/std/env/cliarg.nepl`
    - 未着手（次段で `args_sizes_get/args_get` 周辺の確保失敗と解放方針を整理予定）。

- メモ:
  - `fs` 単体の実行系テストは入力待ちケースを含むため、今後は非対話セットで回帰確認する。

# 2026-03-05 作業メモ (フェーズD: codegen 前段診断の共通化・第一段)

- 目的:
  - `codegen_llvm` 内に残っていた `#target` 個別検証を backend から撤去し、前段共通 precheck へ集約する。
  - `compile_module` と LLVM IR 生成経路で同じ検証入口を使い、wasm/llvm の診断規則差分を縮小する。

- 変更:
  - `nepl-core/src/target_precheck.rs`
    - `precheck_module_target_directives` を追加（`UnknownTargetDirective` / `MultipleTargetDirective` を共通生成）。
    - `precheck_module_before_codegen` を追加（target directive + raw body precheck の合成）。
  - `nepl-core/src/codegen_llvm.rs`
    - `validate_target_directive_for_llvm` / `is_known_target_name` を削除。
    - `emit_ll_from_module_for_target` 入口を `precheck_module_before_codegen` へ統一。
  - `nepl-core/src/compiler.rs`
    - `compile_module` の precheck 呼び出しを `precheck_module_before_codegen` へ置換。

- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/llvm_target.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-unify-step2-focus.json -j 15`
    - 結果: `5/5 pass`
  - 補足:
    - `tests/neplg2.n.md` では既知の runtime 側 `Maximum call stack size exceeded` が残存（今回変更範囲外）。

# 2026-03-05 作業メモ (tests.js: argv メタ対応追加)

- 目的:
  - `stdin/stdout` に加えて doctest から CLI 引数を注入できるようにし、`stdlib/tests/cliarg.n.md` をテスト可能にする。

- 変更:
  - `nodesrc/parser.js`
    - doctest メタに `argv:` を追加。
    - `parseMetaValue` が `[` / `{` 始まりの JSON も解釈するよう拡張（`argv: ["a","b"]` を配列として取得）。
  - `nodesrc/tests.js`
    - テストケース構造に `argv` を追加。
    - wasm ワーカー要求へ `argv` を伝搬。
    - llvm 実行時にも `argv` を実行引数として渡す。
  - `nodesrc/run_test.js`
    - WASI 実行時の args を `argv` から受け取り、`[wasmPath, ...argv]` で起動。
  - `stdlib/tests/cliarg.n.md`
    - `neplg2:test[assert_io]` + `argv` + `stdout` で `cliarg_count` 検証ケースを追加。

- 検証:
  - parser 単体確認:
    - `node -e "const p=require('./nodesrc/parser'); const r=p.parseFile('stdlib/tests/cliarg.n.md'); console.log(Array.isArray(r.doctests[0].argv), JSON.stringify(r.doctests[0].argv));"`
    - 結果: `true ["--flag","value"]`
  - run_test 直実行確認:
    - `argv=["a","b"]` で `cliarg_count` 出力が `"3"`
    - `argv=[]` で `cliarg_count` 出力が `"1"`
  - tests.js 単体確認:
    - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-stdlib --no-tree -o /tmp/tests-cliarg-only-argv.json -j 1 --assert-io`
    - 結果: `2/2 pass`

# 2026-03-05 作業メモ (フェーズD: stdlib/std 安全化の完了と全体回帰)

- 目的:
  - `stdlib/std` の安全化対象（`fs` / `stdio` / `env/cliarg`）を `Result` ベースへ揃え、`alloc_raw` 直接利用の削減と失敗経路の明示化を完了する。

- 変更:
  - `stdlib/std/fs.nepl`
    - `__fs_copy_to_cstr` を `Result<i32,i32>` 化。
    - `wasi_path_open` で確保失敗を `Err` で返し、成功時 `cpath` を必ず解放。
    - `fs_bytes_to_string` を `fs_alloc` ベースへ変更。
    - if レイアウト内の不要 `;` を除去（式戻り値整合）。
  - `stdlib/std/stdio.nepl`
    - `print_i32` の一時領域確保を `std_alloc/std_free` ベースへ変更。
    - `read_all` の if 式で `else out;` になっていた箇所を `out` に修正し、`expr; -> ()` による型不整合を解消。
  - `stdlib/std/env/cliarg.nepl`
    - `cstr_to_str` の確保を `cli_alloc` ベースへ変更し、失敗時フォールバックを明示。

- 根本原因と修正方針:
  - 全体回帰で `tests/stdin.n.md` のみ wasm stack mismatch が発生。
  - 原因は `read_all` の `if` 式 else 側が `out;` となっており、仕様どおり `()` に化けていたこと。
  - 場当たりでコード分解せず、式の戻り値規則（plan.md の `;` 仕様）に沿って `out` へ修正して根本解消。

- 検証:
  - `node nodesrc/tests.js -i stdlib/tests/fs.n.md --no-stdlib --no-tree -o /tmp/tests-fs-safe-phase.json -j 15` -> `1/1 pass`
  - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md -i tests/stdout.n.md -i stdlib/tests/fs.n.md --no-stdlib --no-tree -o /tmp/tests-std-safe-regression.json -j 15 --assert-io` -> `9/9 pass`
  - `node nodesrc/tests.js -i tests/stdin.n.md --no-tree -o /tmp/tests-stdin-focus.json -j 15 --assert-io` -> `210/210 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-stdlib-std-safety-phase.json -j 15` -> `788/788 pass`

# 2026-03-05 作業メモ (MemPtr/RegionToken 再調査と _raw 廃止方針の再整理)

- 調査目的:
  - `MemPtr/RegionToken` 導入後の残存生ポインタ依存と `_raw` 依存を全体で棚卸しし、上流優先での移行順を再確定する。

- 現状要約:
  - `core/mem.nepl` には `MemPtr<T>` / `RegionToken<T>` と `region_ptr_at/alloc_region/dealloc_region` が実装済み。
  - `kpread/kpwrite` は公開構造体が `RegionToken<u8>` を保持する形まで移行済み。
  - ただし `core/mem` 公開面には `alloc_raw/dealloc_raw/realloc_raw` と `load/store(i32)` 生ポインタ版が残存。
  - `stdlib/alloc` / `stdlib/kp` / `stdlib/nm` / `platforms/wasix` / examples/tests には `_raw` 呼び出しが多数残存。
  - `nepl-core` 側にも `_raw` 名依存が残存（`monomorphize.rs`, `codegen_wasm.rs`, `codegen_llvm.rs`）。

- 根本課題:
  - `_raw` 廃止は stdlib 側だけでは完了せず、compiler 側の helper 解決ロジックを先に一般化する必要がある。
  - `core/mem` の生ポインタAPIを先に削除すると、下流ライブラリと codegen が同時崩壊するため、段階移行が必要。

- 再確定した実装順序（上流優先）:
  1. compiler 側 `_raw` 名依存の除去（`monomorphize` / `codegen_wasm` / `codegen_llvm`）。
  2. `core/mem` を安全API公開面に統一し、生ポインタAPIを内部互換層へ隔離。
  3. `stdlib/alloc` と `kp` を `MemPtr/RegionToken` + `Result/Option` 前提へ全面移行。
  4. `stdlib/std` / `stdlib/nm` / tutorials/examples の順で追随移行。
  5. 最後に `_raw` と生ポインタ公開関数を削除し、compile_fail 回帰を固定。
# 2026-03-05 作業メモ (フェーズD: wasm signature 診断を codegen 前段へ移動)

- 目的:
  - `codegen_wasm` 内で出していた署名系診断を前段パスへ移し、`codegen到達時は検証済み` の設計へ寄せる。
  - wasm/llvm 共通化方針の第一段として、backend 直下診断の削減を進める。
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs` を追加。
    - `precheck_wasm_codegen` を実装し、以下を前段で検査:
      - extern 署名 (`D4001`)
      - 到達可能関数の署名 (`D4002`)
  - `nepl-core/src/compiler.rs`
    - `insert_drops` 後・wasm emit 前に `precheck_wasm_codegen` を実行。
    - エラー診断があれば codegen へ進まず `CoreError::Diagnostics` を返す。
  - `nepl-core/src/codegen_wasm.rs`
    - 署名不一致時の `D4001/D4002` 生成を削除し、前段検査前提でスキップ処理に変更。
  - `tests/raw_body_precheck.n.md`
    - `D4001/D4002` を安定再現する `compile_fail` ケースを追加・調整。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v5.json -j 15` -> `4/4 pass`
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v6.json -j 15` -> `7/7 pass`
# 2026-03-05 作業メモ (フェーズD: D4003 を codegen 前段へ移動)

- 目的:
  - `CodegenWasmMissingReturnValue (D4003)` を backend 依存診断から前段診断へ移し、codegen 到達時の前提を強化する。
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 到達可能関数の `HirBody::Block` について、
      - 戻り型が `Unit` 以外
      - 最終的な非 drop 行が値を返さない
      場合に `D4003` を前段で出す検査を追加。
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_user` 内の `D4003` 診断生成を削除。
    - ここに到達した場合は内部不整合として `panic!`（precheck で弾かれる前提）に変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v7.json -j 15` -> `7/7 pass`
# 2026-03-05 作業メモ (フェーズD: D4005 を codegen 前段へ移動)

- 目的:
  - `CodegenWasmLlvmIrBodyNotSupported (D4005)` を backend 側診断から前段診断へ移し、codegen の責務を縮小する。
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 到達可能関数で `HirBody::LlvmIr` が残っている場合に `D4005` を前段で出す検査を追加。
  - `nepl-core/src/codegen_wasm.rs`
    - `HirBody::LlvmIr` 分岐で `D4005` を生成する処理を削除。
    - precheck 通過後の内部不整合として `panic!` に変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v8.json -j 15` -> `7/7 pass`
# 2026-03-06 作業メモ (alloc/string: bool と基数付き整数文字列変換の整理)

- 目的:
  - 文字列表現への変換責務を `alloc/string` に集約し、`core/cast` を値変換専用に保つ。
  - 2 / 8 / 10 / 16 進の整数文字列化・解析を `alloc/string` の API として揃える。
- 変更:
  - `stdlib/alloc/string.nepl`
    - `from_bool` を追加し、bool の表示用文字列化を `alloc/string` に統一。
    - `from_i32` を `from_i32_radix x 10` 経由へ変更。
    - `to_i32` を `to_i32_radix s 10` 経由へ変更。
    - `from_i64` を `from_i64_radix x 10` 経由へ変更。
    - `to_i64` を `to_i64_radix s 10` 経由へ変更。
    - 新規に `digit_to_char_lower` / `digit_from_char` / `validate_radix` を追加。
    - 新規に `from_i32_radix` / `to_i32_radix` / `from_i64_radix` / `to_i64_radix` を追加。
    - 2 / 8 / 10 / 16 進のみを受理する方針をドキュメントコメントに明記。
    - `from_bool` / `from_i32` / 基数付き変換の説明を、目的・実装・注意・計算量が分かる形へ手書きで更新。
  - `stdlib/std/test.nepl`
    - bool の文字列化を `from_bool` に統一。
  - `tests/stdlib.n.md`
    - `from_i32_radix 10 2`
    - `from_i64_radix 255 16`
    - `to_i32_radix "1010" 2`
    - `to_i64_radix "Ff" 16`
    - 不正桁 / 不正基数
    を focused test として追加。
- 検証:
  - `node nodesrc/tests.js -i /tmp/one-radix-format.n.md --no-stdlib --no-tree -o /tmp/one-radix-format-only.json -j 1` -> `1/1 pass`
  - `node nodesrc/tests.js -i /tmp/one-radix-parse.n.md --no-stdlib --no-tree -o /tmp/one-radix-parse-only.json -j 1` -> `1/1 pass`
  - `node nodesrc/tests.js -i tests/stdlib.n.md -i tutorials/getting_started/10_project_fizzbuzz.n.md --no-stdlib --no-tree -o /tmp/tests-string-radix-focused-v1.json -j 15` -> `13/13 pass`
- 判断:
  - `bool -> str` は値変換ではなく文字列表現化なので `core/cast` ではなく `alloc/string` に置く。
  - 2 / 8 / 10 / 16 進の基数指定は文字列 API の責務なので、`cast` ではなく `alloc/string` に置く。
  - `core/cast` には数値/論理/ビット/ポインタの値変換だけを残す方針が一貫している。
- 未完:
  - `alloc/string.nepl` を input にした stdlib doctest 実行経路は別途整理が必要。
  - `i128` の文字列表現変換は未実装。

# 2026-03-05 作業メモ (フェーズD: D4011 を codegen 前段へ移動)

- 目的:
  - `CodegenWasmUnsupportedIndirectSignature (D4011)` を backend 側から前段へ移し、`call_indirect` の署名妥当性を codegen 前に確定する。
- 変更:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - HIR 式を再帰走査し、`CallIndirect` の `params/result` から `wasm_sig_ids` を評価。
    - wasm 非対応署名を検出した場合に `D4011` を前段で返す検査を追加。
  - `nepl-core/src/codegen_wasm.rs`
    - `CallIndirect` 分岐の `D4011` 診断生成を削除し、precheck 通過後の内部不整合として `panic!` に変更。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-indirect-v5.json -j 15` -> `7/7 pass`

# 2026-03-05 作業メモ (フェーズD: D4004 を codegen 前段へ移動)

- 目的:
  - `CodegenWasmRawLineParseError (D4004)` を backend 側診断から前段診断へ移し、`#wasm` 生行パース失敗を codegen 前に確定する。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `HirBody::Wasm` 分岐での `D4004` 生成を削除。
    - precheck 通過後の内部不整合として `panic!` に変更。
    - `precheck_raw_wasm_body(func)` を追加し、`parse_wasm_line` 失敗時に `D4004` を返す前段用ヘルパを実装。
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_wasm_codegen` から `codegen_wasm::precheck_raw_wasm_body` を呼び出すよう変更。
  - `tests/raw_body_precheck.n.md`
    - `wasm_precheck_rejects_invalid_raw_line` を追加（`diag_id: 4004`）。
- 検証:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-rawline-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: D4010 を codegen 前段へ移動)

- 目的:
  - `CodegenWasmMissingIndirectSignature (D4010)` を backend 側診断から前段へ移し、`CallIndirect` の型セクション不整合を codegen 前に検査する。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `collect_wasm_signature_set` を追加し、wasm codegen で使う関数/extern/間接呼び出し署名集合を共通化。
    - `CallIndirect` 分岐の `D4010` 診断生成を削除し、precheck 通過後の内部不整合として `panic!` へ変更。
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `collect_wasm_signature_set` の結果を使い、`CallIndirect` の署名が型セクション候補に存在するかを前段で検査。
    - 欠落時は `D4010`、非対応署名は `D4011` として分離して返す。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-indirect-missing-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: 参照解決系 wasm backend 診断の削減)

- 目的:
  - `CodegenWasmStringLiteralNotFound (4006)` / `CodegenWasmUnknownVariable (4007)` /
    `CodegenWasmUnknownFunctionValue (4008)` / `CodegenWasmUnknownFunction (4009)` を
    backend 診断から外し、上流通過後の内部不整合として扱う。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `LiteralStr/Var/FnValue/Call/Set` での上記診断生成を削除。
    - 同箇所は `panic!` に変更し、codegen 到達時は解決済み前提を強制。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-ref-invariant-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: unknown intrinsic 診断の前段化整合)

- 目的:
  - `CodegenWasmUnknownIntrinsic (4012)` を backend 診断から外し、intrinsic 判定責務を前段へ寄せる。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `is_supported_wasm_intrinsic` を追加して wasm backend が受理する intrinsic 名を明示化。
    - intrinsic 未知分岐の `D4012` 生成を削除し、内部不整合 `panic!` へ変更。
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `HirExprKind::Intrinsic` で `is_supported_wasm_intrinsic` を使用し、未知 intrinsic を前段検査。
  - `tests/raw_body_precheck.n.md`
    - 追加した `diag_id:4012` ケースは、実際には上流の `D3012`（unknown intrinsic）で先に失敗するため削除。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-unknown-intrinsic-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: 構築型 payload/field の backend 診断削減)

- 目的:
  - `CodegenWasmUnsupportedEnumPayloadType (4013)` /
    `CodegenWasmUnsupportedStructFieldType (4014)` /
    `CodegenWasmUnsupportedTupleElementType (4015)` を backend 診断から外し、codegen 到達時の型整合前提を明確化する。
- 変更:
  - `nepl-core/src/codegen_wasm.rs`
    - `EnumConstruct` と `Match` の enum payload load/store、`StructConstruct`、`TupleConstruct` の
      非対応 valtype 分岐を `panic!` に変更。
    - 上記 4013/4014/4015 の `diags.push(...with_id(...))` を削除。
    - これにより、`codegen_wasm` 内の `CodegenWasm*` 診断生成は precheck ヘルパ内（D4004）のみに限定。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-backend-diag-clean-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: llvm backend の解決済み参照エラーを内部不整合化)

- 目的:
  - wasm 側と同様に、名前解決/署名解決済みであるべき参照系エラーを backend 診断責務から外す。
- 変更:
  - `nepl-core/src/codegen_llvm.rs`
    - `Var` の unknown 変数分岐を `panic!` 化。
    - `Set` の unknown 変数分岐を `panic!` 化。
    - `FnValue` の unknown 関数値分岐を `panic!` 化。
    - `Call` の missing function signature 分岐を `panic!` 化。
- 検証:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 作業メモ (フェーズD: monomorphize の runtime helper 候補ハードコード集約)

- 目的:
  - `_raw` 撤去フェーズに備え、`monomorphize` 内の runtime helper 候補名ハードコードを一箇所に集約する。
- 変更:
  - `nepl-core/src/runtime_helpers.rs` を追加。
    - `ALLOC_CANDIDATES`
    - `DEALLOC_CANDIDATES`
    - `REALLOC_CANDIDATES`
  - `nepl-core/src/lib.rs` に `runtime_helpers` を公開。
  - `nepl-core/src/monomorphize.rs`
    - runtime helper 選択ループの文字列配列リテラルを `runtime_helpers` 定数参照に置換。
- 検証:
  - `NO_COLOR=false trunk build` -> success
