/nepl-core に Rustで作ったNEPLG2コンパイラがあります
/stdlib/neplg2/src に NEPLG2 で NEPLG2コンパイラを作ります こっちがセルフホストです

/stdlib/neplg2/cli/main.nepl にインターフェイスを作ります
/stdlib/neplg2/src/core/ にコンパイラの本体を作ります

Rustで作ったNEPLG2コンパイラを使いながら、NEPLG2でNEPLG2コンパイラを作ってください
適宜テストを書き、適切に動いていることを確認してください