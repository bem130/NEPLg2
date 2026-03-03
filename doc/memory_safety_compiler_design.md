# メモリ安全コンパイラ設計

最終更新: 2026-03-03

## 1. 目的

- GC なしで、コンパイラ管理のみでメモリ安全性を確保する。
- heap/線形メモリ操作を pure として扱うための実装条件を定義する。
- `mem` / `kpread` / `kpwrite` を `Result/Option` 前提の安全APIに統一する。

## 2. 公開モデル

### 2.1 公開型

- `MemPtr<T>`
  - 型付きメモリ参照。
  - 生アドレス整数は隠蔽する。
- `RegionToken`
  - 領域の所有権を表す線形トークン。
  - `dealloc` で消費される。

### 2.2 不変条件

- `MemPtr<T>` は有効な `RegionToken` と対応している。
- 解放済み `RegionToken` からのアクセスは不可能。
- `offset + sizeof(U) <= size` を満たす場合のみ `load/store<U>` を許可。
- 二重解放・解放後アクセスは検出して拒否する。

## 3. effect との整合

- メモリ操作（`alloc/realloc/dealloc/load/store`）は Pure。
- I/O 操作（stdin/stdout/fs/env/time/random/syscall）は Impure。
- したがって、I/O を含まないメモリ処理関数は `->` を保てる。

この整理は `doc/move_effect_spec.md` の規則を前提とする。

## 4. コンパイラで行う検査

### 4.1 型検査

- `load/store` などを `MemPtr<T>` 受け取りに統一する。
- 生 `i32` ポインタ受け取りを公開APIから禁止する。
- fallible 操作を `Result/Option` で型に反映する。

### 4.2 move/borrow 検査

- `RegionToken` は非Copy。
- `dealloc(token)` 後の token 再利用を禁止する。
- `MemPtr<T>` の借用中は可変性制約を適用する。
- 分岐/ループ合流で `PossiblyMoved` を保守的に維持する。

### 4.3 境界/生存検査

- `load/store` の境界検査を挿入する。
- 解放後アクセスを `Result::Err` 経路へ分岐させる。
- 定数証明可能な安全アクセスは最適化で検査削除可能。

### 4.4 trait 制約検査

- `Copy` 実装可否を構造的に検査し、リソース所有型の `Copy` 実装を禁止する。
- `Clone` 実装は move 規則と矛盾しない複製規約を満たすことを要求する。
- メモリ系 trait（`MemReadable<T>`, `MemWritable<T>`, `RegionOwned`）の境界を満たさない呼び出しは型エラーにする。

## 5. API 設計指針

### 5.1 core/mem

- `_raw` 公開関数は段階的に削除し最終的に廃止。
- `_safe` 接尾辞は廃止し、安全版を標準名へ統一。
- 失敗を `Result<_, Diag>` または `Option<_>` で返す。

### 5.2 kpread / kpwrite

- `Scanner` / `Writer` に所有権と領域情報を保持させる。
- ハンドル `i32` を外部APIへ露出しない。
- I/O 実行部のみ Impure として扱う。

### 5.3 trait ベース API

- `core/mem` の読み書きAPIは trait 境界で能力を表現する。
- `kpread/kpwrite` は `RegionOwned` を満たす型のみが解放操作を実行できるようにする。

## 6. 診断

少なくとも以下の診断カテゴリを持つ。

- メモリ型不一致
- 範囲外アクセス
- 解放後アクセス
- 二重解放
- moved 値使用
- pure 文脈での impure 呼び出し

compile_fail テストでは diag_id で固定検証する。

## 7. 段階導入

1. `MemPtr<T>` / `RegionToken` を core/mem で確立。
2. builtins/effect 判定を仕様へ合わせる。
3. move check を token 消費対応へ拡張。
4. stdlib (`mem/kpread/kpwrite`) を安全APIへ統一。
5. tests に memory/effect 回帰を追加。

## 8. 非目標

- GC 導入は行わない。
- 未定義動作で隠す設計は採用しない。
- 旧ポインタAPIとの後方互換は維持しない。
