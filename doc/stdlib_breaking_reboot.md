# stdlib 破壊的再設計案（後方互換なし）

最終更新: 2026-03-08

## 1. 目的

- stdlib 全体を「安全 API が標準、低レベル API は隔離」の方針で再構築する。
- ジェネリクスと trait を中心に据え、型能力で API 利用可否を決定する。
- 現行の `_raw` / `*_safe` 併存を廃止し、命名・責務・所有権規則を単純化する。
- `plan.md` と `tutorials` の思想（式指向・前置記法・オフサイドルール・パイプによる合成）を stdlib API 設計へ直接反映する。

## 2. 非目標

- 既存 stdlib API とのソース互換・バイナリ互換は提供しない。
- 段階的 deprecate は行わず、`vNext` で一括置換する。
- 暗黙変換・暗黙効果昇格・暗黙 import による利便性向上は行わない。

## 3. NEPLg2 哲学との整合要件

### 3.1 式指向との整合

- `if`/`match` を値として返す設計を活かし、API は「値を返す関数」を優先する。
- `let mut` と手続き更新だけを前提にした API は避け、合成しやすい返り値設計を標準にする。

### 3.2 前置記法・パイプとの整合

- 引数順は `value |> transform ...` で読める順序を標準にする。
- データ変換 API は「対象値を第1引数」に統一し、tutorials の関数合成スタイルと揃える。

### 3.3 effect 規則との整合

- 純粋変換は `->`、I/O やメモリ更新を伴う処理は `*>` を明示する。
- 失敗可能性は `Result`/`Option` に寄せ、`unreachable` 依存を公開面から排除する。

### 3.4 型駆動設計との整合

- 型注釈 `<T>` による曖昧性解消を前提に、過剰な名前分岐を避ける。
- trait 境界で能力を明示し、「使える理由」を型として表現する。

## 4. 設計原則

- 公開 API の戻り値は `Result` / `Option` を標準とし、エラー理由を観測可能にする。
- メモリ境界・所有権・副作用可否は trait 境界で明示する。
- trait 解決とオーバーロード解決は同じ型同値判定で処理する。
- API 名は能力を表す語彙で統一し、実装都合（raw/safe）を露出しない。
- target 依存実装は adapter 層へ閉じ込め、`core`/`alloc` は target 非依存を維持する。

## 5. 新しいパッケージ構成

```text
stdlib/
    core/                # 純粋計算・基本型・trait 定義
    alloc/               # メモリ管理、所有権付き型
    collections/         # Vec/Map/Set/Queue/Stack など
    text/                # 文字列・Unicode・format/parse
    io/                  # Reader/Writer 抽象
    fs/                  # ファイルシステム抽象
    runtime/             # target 別 adapter 実装
        wasi/
        wasm/
        nasm/
        c/
    prelude/
        core.nepl
        std.nepl
```

- 旧 `std/*` の集中を分離し、責務境界を明確化する。
- 旧 `mem/kpread/kpwrite` は `io` と `alloc` の能力モデルへ再配置する。

## 6. trait 中心の能力モデル

### 6.1 基本能力 trait

- `Eq<T>`, `Ord<T>`, `Hash<T>`
- `Show<T>`, `Parse<T>`
- `Default<T>`, `Clone<T>`, `Copy<T>`
- `Add<T,U,R>`, `Sub<T,U,R>` など演算 trait

### 6.2 メモリ能力 trait

- `RegionOwned<T>`: 領域の所有権を保持する。
- `MemReadable<T>`: `T` の読み取り能力を持つ。
- `MemWritable<T>`: `T` の書き込み能力を持つ。
- `Allocator<A>`: 領域確保・解放ポリシーを供給する。

### 6.3 I/O 能力 trait

- `Reader<R>` / `Writer<W>`
- `Seekable<S>`
- `Buffered<B>`

## 7. ジェネリクス再設計

### 7.1 コンテナ

- `Vec<T, A: Allocator>`
- `Map<K, V, H: Hash, A: Allocator>`
- `Set<T, H: Hash, A: Allocator>`
- `String<A: Allocator>`

### 7.2 エラー型

- モジュールごとの個別エラーを乱立させず、共通エラーへ集約する。
- `IoError`, `FsError`, `ParseError`, `AllocError` を `StdError` に合流する。

```text
StdError:
    Alloc AllocError
    Io IoError
    Fs FsError
    Parse ParseError
    InvalidState str
```

### 7.3 失敗の表現

- debug 補助以外の公開 API で panic 相当挙動を使わない。
- 本番 API は `Result<T, StdError>` または `Option<T>` を返す。

## 8. 命名・API 方針（破壊的変更）

- `_raw`, `_safe` 接尾辞を全面廃止する。
- 低レベル API は公開面から隠蔽し、`alloc::region::*` に集約する。
- `to_xxx` は意味が曖昧なため廃止し、
    - 失敗しない変換は `into_xxx`
    - 失敗する変換は `parse_xxx`
  に統一する。
- 出力 API は `Writer` trait ベースへ移行し、`print_*` の型別増殖を止める。

## 9. I/O と platform 分離

- `io` は抽象 trait とバッファ実装のみを持つ。
- WASI などの syscall 依存は `runtime/<target>` adapter に閉じ込める。
- CLI は `runtime::wasi` を選択して注入し、`core`/`alloc` は target 非依存を維持する。

## 10. 移行フェーズ（互換なし前提）

1. trait 契約の確定
    - `doc/trait_system_design.md` と統合し、能力 trait と coherence 規則を固定する。
2. alloc 再実装
    - `RegionToken<T>` 中心の API へ一本化する。
3. collections/text 再実装
    - allocator を明示パラメータ化し、合成しやすい API へ揃える。
4. io/fs 抽象化
    - runtime adapter 経由のみで platform API を呼ぶ。
5. tutorials/examples 置換
    - 前置記法・パイプ合成を活かす書き方でサンプルを再構成する。
6. compiler helper 依存除去
    - runtime helper の `_raw` 名依存を完全削除する。

## 11. テスト戦略

- trait 解決の曖昧性・重複 impl を compile_fail で固定する。
- `alloc/collections/text/io/fs` それぞれに edge case を含む回帰を追加する。
- `trunk build` + `nodesrc/cli.js` 系テスト + stdlib doctest をリリースゲートにする。
- target ごとの差分は adapter テストに限定し、共通挙動は同一ケースで検証する。

## 12. 期待効果

- 命名統一で API 学習コストを削減する。
- trait 境界により、move/effect/memory ルールと stdlib の不整合を減らす。
- target 依存の責務分離により、llvm/nasm/c/wasm の拡張を容易にする。
- compiler 側の文字列ハードコード依存を減らし、診断の一貫性を高める。
