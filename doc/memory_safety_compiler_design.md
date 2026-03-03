# メモリ操作を安全に扱うコンパイラ設計

最終更新: 2026-03-03

## 1. 目的

- `mem` / `kpread` / `kpwrite` を含む線形メモリ操作を、言語仕様として安全に扱う。
- `i32` 生ポインタ露出を段階的に排除し、型と診断で誤用を防止する。
- 実行時クラッシュ（OOB, use-after-free, double free）を減らし、失敗を `Result/Option` へ収束させる。

## 2. 安全性モデル

## 2.1 値カテゴリ

- `RawAddr`（内部専用）
  - 実体は `i32` オフセット。
  - ユーザコードには公開しない。
- `MemPtr<T>`（公開）
  - `addr`, `size`, `region_id`, `alive` を保持する構造体。
  - `T` は論理型タグとして使う。
- `RegionToken`
  - 割り当て単位の所有権トークン。
  - `dealloc` は `RegionToken` を消費し、再利用不能にする。

## 2.2 基本不変条件

- `MemPtr<T>` の `region_id` は生成元 `RegionToken` と一致しなければならない。
- `alive=false` の領域は読み書き不可。
- `offset + sizeof(U) <= size` を満たす場合のみ `load/store<U>` を許可する。
- `dealloc` 済みトークンは再利用不可。

## 3. コンパイラで行う検査

## 3.1 型検査フェーズ

- `load/store/alloc/realloc/dealloc` の公開APIを `MemPtr<T>` ベースへ統一。
- `RawAddr` を受け取る API はコンパイラ内部/stdlib内部限定にする。
- `MemPtr<T>` 以外を `load/store` に渡したら型エラーにする。

## 3.2 move/所有権検査フェーズ

- `RegionToken` は非Copy。
- `dealloc(token)` 呼び出し時に token を moved にする。
- moved token の再利用は `use of moved value` で拒否する。
- `MemPtr<T>` は Copy 可否を設計で選択:
  - 初期方針: 非Copy（厳格）
  - 最適化方針: `RegionToken` 管理が安定後に Copy 化を検討

## 3.3 境界検査フェーズ（新設）

- `load/store` 前に bounds check を自動挿入:
  - 失敗時は `Err(Diag::OutOfBounds)` を返す分岐へ lower。
- 既知定数で安全が証明できる場合はチェックを削除（定数畳み込み）。

## 3.4 解放検査フェーズ（新設）

- `RegionToken` の状態遷移:
  - `Alive -> Freed`
- `Freed` への二重 `dealloc` は `Err(Diag::DoubleFree)` へ変換。
- `Freed` 領域へのアクセスは `Err(Diag::UseAfterFree)` へ変換。

## 4. effect 規則との関係

- `alloc/realloc/dealloc/load/store` は Pure。
- `stdin/stdout/fs/env/time/random/syscall` は Impure。
- したがって「メモリ操作を含むが I/O を含まない関数」は Pure で記述できる。

## 5. stdlib API への適用方針

## 5.1 `core/mem.nepl`

- `_raw` を最終削除し、`Result/Option` 版のみ残す。
- `alloc_ptr` は `Result<MemPtr<u8>, Diag>` を返す。
- `load/store` は `MemPtr<T>` を受け、成功/失敗を返す。

## 5.2 `kp/kpread.nepl`

- `Scanner` 内部に `MemPtr<u8>` と `RegionToken` を保持する。
- `scanner_free` は token を消費する。
- `scanner_read_*` は `Result<value, Diag>` を標準化する。

## 5.3 `kp/kpwrite.nepl`

- `Writer` も同様に `MemPtr<u8>` と `RegionToken` を保持する。
- バッファ拡張失敗を `Err(Diag::OutOfMemory)` で返す。
- flush/write の I/O 本体のみ Impure とする。

## 6. 診断方針

追加する診断カテゴリ（IDは実装時に採番）:

- メモリ型不一致（`MemPtr<T>` 必須箇所）
- 範囲外アクセス
- 解放後アクセス
- 二重解放
- 解放漏れ（将来的に関数境界検査を導入）

`compile_fail` では `diag_id` で固定検証する。

## 7. 最小導入ステップ

1. `MemPtr<T>` / `RegionToken` 型を `core/mem` に定義。
2. `mem` 公開APIを `Result/Option` 版に一本化。
3. move_check に token 消費検査を追加。
4. `kpread/kpwrite` を新APIへ移行。
5. `tests/memory_safety.n.md` を追加し、OOB/二重解放/解放後アクセスを検証。

## 8. 非目標

- GC 導入は行わない。
- 暗黙回復（不正アクセスを自動修復）は行わない。
- 旧 `i32` ポインタ API との後方互換は維持しない。
