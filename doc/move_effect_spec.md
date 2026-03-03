# move規則・pure/impure規則 再設計仕様

最終更新: 2026-03-03

## 0. 3軸の分離

この仕様では、次の3軸を明確に分離して扱う。

- `Option/Result`: 欠損・失敗の表現
- `Pure/Impure`: 外部観測可能な副作用の表現
- `Move/Borrow/Copy`: 所有権と再利用可能性の表現

`Result` を返すこと自体は impure を意味しない。  
逆に pure 関数であっても、非Copy値を引数で受け取れば move は発生する。

## 1. 目的

- move規則と effect 規則の責務を分離し、判定理由を明確化する。
- heap / 線形メモリ操作は「外部世界への副作用ではない」ため pure として扱う。
- impure は I/O（入出力、OS呼び出し、環境依存値取得）に限定する。
- stdlib の API 設計（`mem` / `kpread` / `kpwrite`）を move 規則と整合させる。

## 2. 基本方針

### 2.1 effect は「外部観測可能な副作用」で判定する

- `->` : Pure
- `*>` : Impure

判定基準:

- Pure:
  - 算術・比較・分岐・束縛
  - heap / 線形メモリの `alloc/realloc/dealloc/load/store`
  - コンパイラ管理の内部状態のみを書き換える処理
- Impure:
  - 標準入力/標準出力
  - ファイルシステム
  - 環境変数・argv・時刻・乱数
  - syscall / extern によるホスト依存操作

### 2.2 move は「所有権と再利用可能性」で判定する

effect と独立に判定する。  
Pure でも move される値は move される。Impure でも Copy 値は再利用できる。

## 3. effect 規則（呼び出し規則）

### 3.1 呼び出し制約

- Pure 文脈から Impure 関数を呼び出してはならない。
- Impure 文脈から Pure/Impure の両方を呼び出してよい。

### 3.2 エントリ関数

- エントリ関数も署名どおりに effect を評価する。
- 既存実装の「entry を常に Impure 文脈で評価する」挙動は廃止する。
- これにより、I/O を行わない `main` を Pure のまま記述できる。

### 3.3 intrinsic / extern の effect

- intrinsic は effect を固定表で管理する。
  - `load/store/alloc/realloc/dealloc` は Pure
  - `fd_read/fd_write` 相当は Impure
- extern は既定で Impure とする。
  - Pure 指定を許可する場合は、個別 whitelist とテストで管理する。

## 4. move 規則

### 4.1 型カテゴリ

- Copy 型:
  - `unit`, `bool`, 数値型、参照型
  - 全フィールドが Copy の tuple/struct/enum
- Move 型:
  - `Box`, 所有コンテナ（`Vec`, `String`, `HashMap` 等）
  - 非Copyフィールドを含む tuple/struct/enum
- Resource 型（非Copy）:
  - `Scanner`, `Writer`, `File` など I/O ハンドル

### 4.2 関数引数の扱い

- デフォルトは値渡し（所有権移動）。
- Copy 型は実質コピーなので再利用可能。
- 非Copy 型は 1 回消費で moved 扱い。

### 4.3 分岐・ループの合流

- `if/match`: 分岐合流は `Valid / Moved / PossiblyMoved` の3値でマージする。
- `while`: ループ本体で move が起きる値は `PossiblyMoved` に昇格する。

### 4.4 borrow 相当の扱い

- アドレス引数を取る `load/store` 系は、アドレス式を borrow 扱いにする。
- borrow は move を発生させない。
- borrow 先が `Moved/PossiblyMoved` の場合はエラーにする。

## 5. heap / 線形メモリを Pure とするための条件

### 5.1 条件

- メモリ領域はランタイム内部状態としてコンパイラが一貫管理する。
- 同一入力・同一初期状態で評価結果が決定的である。
- 外部 I/O（fd/syscall）に依存しない。

### 5.2 API 設計制約

- `mem` は `Result/Option` を標準とし、失敗を値で返す。
- `_safe` 接尾辞は廃止し、安全版をデフォルト API とする。
- `_raw` 系は移行完了後に削除する。

## 6. stdlib 設計指針（move 規則整合）

### 6.1 破壊的更新 API

- `push/insert/update/remove` は更新後の `Self` を返す（チェーン可能）。
- `Pair<Self, T>` を返さない（取得系でのみ値を返す）。

### 6.2 取得 API

- `get/len/peek/contains` は取得値のみを返す。
- 継続利用が必要な場合は API 呼び出し前に値束縛を分離するか、Copy 化可能なラッパを採用する。

### 6.3 `kpread/kpwrite`

- `Scanner` / `Writer` は `i32` 生ハンドルを公開しない設計へ最終移行する。
- I/O 操作を含む関数は Impure、メモリ整形のみの補助関数は Pure に分離する。

## 7. 実装タスク（コンパイラ）

1. `entry` を強制 Impure にする特例を削除する。
2. `TypeCtx::is_copy` を構造的判定へ拡張する（struct/enum を含む）。
3. intrinsic の effect テーブルを実装し、typecheck で一元参照する。
4. move_check の borrow 判定を API 設計と一致させる。
5. 診断IDを move/effect 系へ拡充し、compile_fail テストで固定化する。

## 8. テスト計画

- `tests/move_effect.n.md`（新規）
  - pure から I/O 呼び出しが失敗すること
  - pure から `mem` 操作が通ること
  - 分岐/ループ合流で `PossiblyMoved` が検出されること
- `tests/overload.n.md`
  - effect が混在するオーバーロード解決の回帰
- `tests/kp.n.md`, `tests/stdin.n.md`
  - `Scanner` / `Writer` の move 安全性回帰

## 9. 非目標

- 暗黙 cast による overload 解決は行わない。
- 旧 API との後方互換は維持しない。

## 10. 現行実装との主要ギャップ（2026-03-03時点）

- `check_function` が entry 関数を強制的に Impure 文脈で評価している。
- `builtins` の `alloc/realloc/dealloc` が Impure として登録されている。
- `TypeCtx::is_copy` が `struct/enum` を常に非Copyとして扱っている。

これらは本仕様と不一致であり、`todo.md` の実装項目で順次解消する。
