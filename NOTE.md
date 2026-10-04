# セーブデータの保存先に関する調査メモ

検索対象を検討するためのメモです。以下の記述は当初の調査・検討を残したもので、各ゲームの現在の保存先を確認した一覧ではありません。利用手順は [README.md](README.md) を参照してください。

## バックアップ対象の検討

[Windowsのディレクトリ構成ガイドライン - torutkのブログ](https://torutk.hatenablog.jp/entry/20110604/p1) を参考に、`C:\Users\<ユーザー名>` 以下を検討しました。

- `Roaming` はバックアップ対象候補になりそう。
- 当初は `Local`、`LocalLow` は不要ではないかと考えていた。ただし、下記のゲーム個別のメモにも `Local` 配下の保存先があるため、この推測だけで検索対象から除外しない。

[バーチャルストアに関するサポート情報](https://www.eukleia.co.jp/eushully/support/html/spdl_vis.html#sap2_1) を参考に、次の場所も検索対象に含めるか検討しています。

```text
C:\Users\<ユーザー名>\AppData\Local\VirtualStore\
```

## ゲーム個別のメモ

[姫狩りダンジョンマイスターのサポート情報](https://www.eukleia.co.jp/eushully/support/html/spdl_e10.html) を参考に記録した保存先です。

```text
C:\Users\<ユーザー名>\AppData\Local\Eushully\姫狩りダンジョンマイスター\SAVE
```
