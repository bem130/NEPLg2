# stdlib 破壊的再設計案（後方互換なし）

最終更新: 2026-03-08

## 1. 目的

- stdlib 全体を「安全 API が標準、低レベル API は隔離」の方針で再構築する。
- `plan.md` の言語仕様、特に式指向・前置記法・パイプ合成・型注釈・effect 規則と矛盾しないライブラリ構成を確立する。
- `Copy` / `Clone` / move / effect / memory の規則を、compiler と stdlib の両方で同じ能力モデルに基づいて扱えるようにする。
- target 依存部分を adapter 層へ閉じ込め、利用者が触れる API 面では target 差分を最小化する。

## 2. 非目標

- 既存 stdlib API とのソース互換・バイナリ互換は提供しない。
- 段階的 deprecate は行わず、最終的には新構成へ一本化する。
- 暗黙変換・暗黙効果昇格・暗黙 import による利便性向上は行わない。
- compiler 内に「特定型は当然 Copy/Clone である」といった固定知識を持ち込まない。

## 3. 設計原則

### 3.1 値中心・式指向

- `if` / `match` / block が値を返す言語仕様に合わせ、ライブラリ API も「値を返す関数」を標準形とする。
- 更新系 API でも、可能な限り合成しやすい返り値設計を優先する。
- パイプ `|>` で自然に読めるよう、変換対象は原則第1引数に置く。

### 3.2 安全 API 優先

- 公開 API の戻り値は `Result` / `Option` を標準とし、失敗理由を観測可能にする。
- `_raw` / `_safe` のような実装都合の命名は公開面から排除する。
- 低レベル API は内部実装または隔離層に閉じ込め、利用者に直接露出しない。

### 3.3 能力は trait で表す

- `Copy` / `Clone` / `Eq` / `Ord` / `Hash` / `Show` / `Parse` などの能力は trait で表す。
- どの型がどの能力を持つかは `.nepl` ソース上の宣言を唯一の根拠とし、compiler 内部に型ごとの固定表を持たない。
- trait 解決とオーバーロード解決は同じ型同値判定に基づいて処理する。

### 3.4 責務を層で分離する

- `core` / `alloc` は target 非依存の基盤層とする。
- `core` は heap 不要でほぼすべての target で共通に提供される最小基盤とする。
- `alloc` は heap 依存だが、heap さえあればどの target でも動く汎用ライブラリ層とする。
- `std` は各個別 target を束ね、device / OS 依存 API を標準的に扱えるようにする facade 層とする。
- `runtime` / `platforms` は target・platform の実装差分を吸収する adapter 層とする。

## 4. 層構造

### 4.1 層の分類

- 基盤層
  - `core`
  - `alloc`
- 標準 API 層
  - `std`
- 暫定・実験層
  - `kp`
- 独立ライブラリ層
  - `nm`
  - `neplg2`
- 実装差分吸収層
  - `runtime`
  - `platforms`

### 4.2 依存方向

依存は原則として次の方向にのみ流れる。

```text
alloc
  -> core

alloc/collections / alloc/text / alloc/io
  -> alloc
  -> core

std/streamio
  -> alloc/io
  -> alloc/text
  -> runtime / platforms

std
  -> core
  -> alloc
  -> std/streamio
  -> runtime / platforms

kp
  -> core / alloc
  -> alloc/collections / alloc/text / alloc/io
  -> std / std/streamio

nm / neplg2
  -> 必要な基盤層へ依存する独立ライブラリ
```

### 4.3 各層の意味

- 基盤層は、他のすべての層が依存する最小土台を提供する。
- `core` は heap 不要の最小土台を提供する。
- `alloc` は heap 依存の一般用途機能を提供する。
- 標準 API 層は、OS / device 依存だが利用者が日常的に使う安定した入口を提供する。
- 暫定・実験層は、設計が固まりきっていない高水準 API を先行投入する場所とする。
- 独立ライブラリ層は、stdlib に含むが一般利用者向け標準 API とは別の責務を持つ。
- 実装差分吸収層は、target / platform 差分を閉じ込める。

## 5. 新しいパッケージ構成

```text
stdlib/
    core/                # 純粋計算・基本型・基本 trait
        rand/            # target 非依存の乱数インタフェースと基本実装

    alloc/               # heap 依存だが target 非依存の汎用ライブラリ
        collections/     # Vec/Map/Set/Queue/Stack などの汎用データ構造
        text/            # str/String/Unicode/format/parse/文字列表現変換
        io/              # Reader/Writer/Seekable/Buffered などの低水準抽象
        diag/            # 診断値・診断構築補助
        encoding/        # json などの符号化/復号
        hash/            # HashMap/HashSet 向けのハッシュ実装

    std/                 # 各個別 target を束ねる標準 API 面
        streamio/        # fs/stdio などを束ねる高水準 stream 抽象
        stdio/
        fs/
        env/

    kp/                  # 競技プログラミング向け暫定ライブラリ
    nm/                  # 拡張 markdown・doc comment・HTML 変換
    neplg2/              # セルフホスト compiler 用ライブラリ
        core/
        cli/

    runtime/             # target 別 adapter 実装
        wasi/
        wasm/
        nasm/
        c/

    platforms/           # wasip2/wasix など追加 platform 能力
    tests/               # stdlib 専用 fixture / test support

    prelude/
        core.nepl
        std.nepl
```

## 6. 各パッケージの責務

### 6.1 `core`

- 純粋で target 非依存な基本機能だけを置く。
- 基本型、演算、基本 trait、`Result` / `Option` などの最小集合を担当する。
- OS / device / syscall / allocator 詳細に依存しない。
- heap を必要としないものを置く。
- `math.nepl` のように、厚い runtime wrapper を必要とせず、ほぼすべての runtime で共通に提供されるべき機能はここに置く。
- heap に依存しない計算ライブラリは原則としてすべて `core` に置く。
- `rand` も heap 不要な実装は `core/rand` に置き、heap を必要とする乱数アルゴリズムだけを将来的に `alloc/rand` へ分離する。

### 6.2 `alloc`

- `core` の上で、heap 依存だが target 非依存の汎用機能を提供する。
- `MemPtr<T>` / `RegionToken<T>` / allocator / 領域管理 / 診断補助 / encoding / hash をここへ置く。
- 低レベル API はここに隔離し、上位層はできる限り安全 API だけを使う。
- `collections` / `text` / `io` のように、heap は使うが device や OS に依存しない層は `alloc` 配下に置く。

### 6.3 `alloc/collections`

- `Vec` / `Map` / `Set` / `Queue` / `Stack` など、一般用途のデータ構造を置く。
- device や OS に依存しないため `std` には入れない。
- allocator と trait 能力に基づき、move/effect/memory 規則と整合する API を提供する。

### 6.4 `alloc/text`

- `str` / `String` / Unicode / format / parse / 文字列表現変換を担当する。
- `bool/i32/i64/i128/...` と文字列の相互変換はここに置く。
- 標準入出力そのものは `text` の責務ではなく、`std/stdio` や `std/streamio` が利用する側とする。

### 6.5 `alloc/io`

- `Reader` / `Writer` / `Seekable` / `Buffered` など、I/O の低水準抽象を置く。
- syscall や descriptor などの具体実装は持たない。
- 「何が読めるか / 書けるか」の能力を trait として表現する。

### 6.6 `std/streamio`

- `alloc/io` の抽象を束ね、`read` / `write` / `flush` / `close` / event の高水準 API を提供する。
- `stdio`、`fs`、将来の socket / timer / process event / UI event などを、同じ stream 能力モデルで扱えるようにする。
- `streamio` は `std` 配下で、各個別 target の `stdio` / `fs` / platform 機能を束ねる標準 stream facade とする。
- `kpread` / `kpwrite` の中核は最終的にこの層へ昇格させる。

### 6.7 `std`

- `core` / `alloc` ではないが、多くのプログラムで標準的に使う device / OS 依存 API を置く。
- `runtime` / `platforms` / `std/streamio` を適切に包み、利用者向けに安定した標準 API 面を提供する。
- `std/stdio` は標準入出力、`std/fs` は標準ファイル操作、`std/env` は CLI 引数や環境取得を担当する。
- `std` は抽象化層ではなく facade であり、下位の複雑さを隠す。

### 6.8 `kp`

- 競技プログラミング向けの先行ライブラリを置く暫定層とする。
- ここは破壊的変更が頻発してよい前提で運用する。
- 十分に成熟し、責務が一般化できた機能は `std` または `alloc/collections` / `alloc/text` / `std/streamio` などへ昇格させる。
- 今回の `kpread` / `kpwrite` 由来の stream まとめ読み書き機能は、その昇格対象に当たる。

### 6.9 `nm`

- 拡張 markdown・doc comment・HTML 変換など、周辺ツールチェーン用の独立ライブラリとする。
- 一般的な標準 API 面ではないため `std` 配下へは入れない。

### 6.10 `neplg2`

- セルフホスト compiler 用ライブラリとする。
- 一般利用者向け標準 API とは別系統であり、`std` 配下へは入れない。

### 6.11 `runtime` / `platforms`

- `runtime` は target ごとの syscall / ABI / descriptor 差分を吸収する adapter 層。
- 各 platform で挙動を共通化するために厚い wrapper が必要なものだけをここに置く。
- `math.nepl` のように厚い wrapper を必要とせず、runtime 差分をほぼ意識せずに提供できる機能は `runtime` には置かない。
- `platforms` は `wasix` など追加 platform capability を扱う層。
- 利用者は原則としてこれらを直接使わず、`std` や `std/streamio` を通して利用する。

## 7. trait と能力モデル

### 7.1 基本能力 trait

- `Eq<T>`, `Ord<T>`, `Hash<T>`
- `Show<T>`, `Parse<T>`
- `Default<T>`, `Clone<T>`, `Copy<T>`
- `Add<T,U,R>`, `Sub<T,U,R>` などの演算 trait

### 7.2 Copy / Clone の扱い

- `Copy<T>` / `Clone<T>` は「組み込み型だから当然持つ能力」ではなく、`.nepl` ソース上で宣言される能力として扱う。
- compiler は「この型は Copy/Clone である」という固定知識をハードコードせず、trait 解決結果だけを move/effect 判定へ渡す。
- 将来的な derive 相当構文を導入する場合も、それは compiler 内固定表ではなく `.nepl` 側の明示宣言として扱う。

### 7.3 メモリ能力 trait

- `RegionOwned<T>`: 領域の所有権を保持する。
- `MemReadable<T>`: `T` の読み取り能力を持つ。
- `MemWritable<T>`: `T` の書き込み能力を持つ。
- `Allocator<A>`: 領域確保・解放ポリシーを供給する。

### 7.4 I/O 能力 trait

- `Reader<R>` / `Writer<W>`
- `Seekable<S>`
- `Buffered<B>`
- `Stream<S>`: `read/write/flush/close` を統一的に扱う。
- `EventSource<E>` / `EventSink<E>`: 将来的に stream 以外のイベントも同じ能力モデルへ乗せる。

## 8. エラー体系

- モジュールごとの個別エラーを乱立させず、共通エラー体系へ集約する。
- 基本形は `Result<T, StdError>` または `Option<T>` とする。
- `StdError` の代表例:

```text
StdError:
    Alloc AllocError
    Io IoError
    Fs FsError
    Parse ParseError
    InvalidState str
```

- debug 補助以外の公開 API で panic 相当挙動を標準にしない。

## 9. 命名方針

- `_raw`, `_safe` 接尾辞は最終的に公開面から廃止する。
- 実装都合ではなく能力と意味に基づく名前を付ける。
- `to_xxx` は曖昧なので、
  - 失敗しない変換は `into_xxx`
  - 失敗する解析は `parse_xxx`
  に寄せる。
- ただし既存 API からの移行経路を考え、最終命名は各層ごとに段階的に揃える。

## 10. 個別論点

### 10.1 `kpread` / `kpwrite` の昇格

- 現在の `kpread` / `kpwrite` は `kp` にあるが、責務としては「まとめて入出力する stream parser / formatter」であり、一般化可能である。
- そのため中核機能は `std/streamio` へ昇格させる。
- `kp` 側には、競技向けの薄いラッパ・ショートカット・テンプレート的機能だけを残す。
- `stdio` だけでなく `fs`、将来の他 stream 実装にも同じ parser / formatter を適用できる設計を目指す。

### 10.2 `std` と `streamio` の関係

- `alloc/io` は heap 依存だが target 非依存の低水準抽象を提供する。
- `std/streamio` は各 target の `stdio` / `fs` / platform 機能を束ねる標準 stream facade である。
- `std` は `std/streamio` と `runtime/platforms` を包み、利用者向けの安定した入口を提供する。

### 10.3 `nm` / `neplg2` の独立性

- `nm` / `neplg2` は stdlib に含むが、言語の標準 API 面とは別の責務を持つ。
- したがって `std` や汎用層へ吸収せず、独立パッケージとして維持する。

## 11. target と platform の分離

- `runtime/<target>` は target 別の最下層 adapter とする。
- `platforms/` は `wasix` など追加 platform capability をまとめる。
- `std` はそれらを直接見せず、安定した標準 API として包む。
- `std/streamio` は `runtime` や `platforms` を束ねる標準 stream facade であり、利用者には target 差分を意識させない。

## 12. 移行の考え方

- 最終目標は新構成への全面移行であり、旧 API は最終的に完全削除する。
- ただし実装は段階的に進める必要があるため、移行途中では旧実装と新実装が一時的に併存しうる。
- その段階的移行手順そのものは `todo.md` に記述し、この仕様書には最終到達形だけを書く。

## 13. テスト方針

- trait 解決の曖昧性・重複 impl・能力不足は `compile_fail` で固定する。
- `alloc` / `alloc/collections` / `alloc/text` / `alloc/io` / `std/streamio` / `std` ごとに edge case を含む回帰を追加する。
- target 差分は adapter テストへ局所化し、共通挙動は同一ケースで検証する。
- `trunk build` + `nodesrc/cli.js` 系テスト + stdlib doctest をリリースゲートにする。

## 14. 期待効果

- 命名統一により API 学習コストを削減できる。
- trait 能力モデルにより、move/effect/memory と stdlib API の不整合を減らせる。
- target 依存責務の分離により、llvm/nasm/c/wasm の拡張が容易になる。
- `kp` の暫定実装から、成熟した機能を `std` / 汎用層へ順次昇格できる。
- compiler 側のハードコード依存を減らし、診断と意味論の一貫性を高められる。

## 15. 誤解しやすい点

- `core` / `alloc` / `std` の区別は「名前」や「利用頻度」ではなく、heap 依存性と target 依存性で決める。
- 計算ライブラリは、標準的によく使うから `std` に入るのではない。heap に依存しないなら原則として `core` に置く。
- `math` はその代表例であり、厚い runtime wrapper を必要としないため `core` に置く。
- `rand` も同様で、heap 不要な実装は `core/rand`、heap を必要とする実装だけを `alloc/rand` に置く。
- `std` は「よく使う汎用機能の置き場」ではなく、各 target / platform を束ねて利用者向けの標準 API を提供する facade である。
- `alloc` は heap を使うが target 非依存な汎用ライブラリの層であり、`collections` / `text` / `io` はこの基準で `alloc` 配下に置く。
- `std/streamio` は `alloc/io` の抽象を使って各 target の `stdio` / `fs` / platform 機能を統一する層であり、`alloc/io` と役割が異なる。
