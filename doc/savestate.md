# SaveState：名前・配置・内容の特徴から候補を採点する

調査日：2026-10-04。対象コミット：`3b550e0dc38c001c42931cf9f476b4d8194420b1`。

[調査概要](README.md) / [公式リポジトリ](https://github.com/Matteo842/SaveState)

## 入力と探索の入口

ゲームをショートカットのドラッグ＆ドロップ、Steam、手動操作などで追加し、保存先候補を探すツールです。以前の NOTE.md で取得できなかった [How Save Search Works](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/docs/How-Save-Search-Works.md) も、今回は clone したリポジトリで確認しました。

探索処理はゲーム名を受け取り、インストール先や Steam AppID などを補助情報にします。ショートカットについてはリンク先を解決し、ゲームのあるディレクトリーを取得する処理があります。[入力処理](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/gui_components/drag_drop_handler.py#L349)

## 保存先を推測する流れ

Windows の中心的な処理は [save_path_finder.py](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/save_path_finder.py#L1127) です。概略は次のとおりです。

1. ゲーム名・exe 名・インストールフォルダー名から、比較用の名前や略称を作る。
2. AppData の Roaming／Local／LocalLow、Documents、My Games、Saved Games、Public Documents、ProgramData などを確認する。
3. 名前から組み立てたパス、各領域のゲーム名・メーカー名の配下、インストール先とその親を探索する。Steam は AppID に対応する userdata も確認する。
4. 存在するフォルダーを候補にし、名前の類似度、場所、保存先らしい構造、ファイルの特徴などで採点する。
5. 上位候補には深さ制限付きの追加探索を行い、並べ直す。

深い探索もフォルダー名・拡張子による判定です。ゲームの保存処理を実行したり、ファイルの内容からセーブ形式を解読したりしているわけではありません。

## 使う特徴と評価

名前については、空白や記号の違い、略称、ローマ数字・数字の違い、あいまいな一致を扱います。exe 名からビルド用の接尾辞を除く処理もあります。似た続編を混同しないための条件も設けています。[名前の派生処理](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/save_path_finder.py#L847)

設定には `SaveData`、`SaveGames`、`Profiles` などのフォルダー名、`.sav` に加えて `.dat`・`.bin`・`.json` などの拡張子、`progress`・`slot`・`persistent` などのファイル名の手掛かりがあります。一般的なデータや設定も候補になる、広い条件です。[設定値](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/config.py#L418)

候補評価では、一般的な保存領域、名前の一致、保存先らしいファイルや子フォルダーを加点し、汎用的なフォルダーなどを減点します。追加探索ではより限定した拡張子も使います。得られる順位は推測上の優先度であり、保存先である確率や実測精度を示すものではありません。[評価処理](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/save_path_finder.py#L479)

システムやキャッシュなどのフォルダー名を除外する設定もあります。ただし、拡張子の除外は一律ではありません。通常の内容確認では、既知の保存拡張子に該当するかを先に調べ、その後、ファイル名による弱い判定からログ等を除いています。[除外設定](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/config.py#L480) / [内容確認](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/save_path_finder.py#L1287)

## 具体例

ゲーム名が `星の冒険`、インストール先が `D:\Games\星の冒険` で、次の場所にデータがあるとします。

```text
D:\Games\星の冒険\Save\slot1.dat
%APPDATA%\同人サークル\星の冒険\SaveData\progress.dat
```

インストール先の `Save` はフォルダー名、外部保存先はゲーム名や配置を手掛かりに候補になり得ます。exe 内部にこれらの完全なパスが残っている必要はありません。この例の検出を実際に実行したわけではありません。

## 日本語名について確認した注意点

名前の一部の比較処理では、英数字と空白以外を取り除き、その後の一致を判定しています。日本語だけの異なる名前が両方とも空文字列になり、同じと扱われる経路があります。[該当処理](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/save_path_finder.py#L1464)

たとえば `星の冒険` と `月の物語` は、この処理では両方とも空になります。これはソース上の条件から確認した問題です。UI 全体での誤検出件数は測定していません。また、元の名前を保持する処理や直接の名前一致もあるため、「日本語ゲームは一切扱えない」という意味ではありません。

## 未登録ゲームへの見込みと限界

**考察：** ゲーム名や一般的な保存構造を使うため、個別の保存先定義がなくても探せる可能性があります。一方、汎用 exe 名、無関係な内部名、独自構造には手掛かりが不足します。JSON や DAT はゲームのアセットや設定にも使われるため、候補の内容確認が必要です。

ゲーム単位で情報を与える入口が中心ですが、ドラッグ＆ドロップ側には複数の対象を順に処理する仕組みもあります。任意の範囲にある未知のゲームをすべて自動識別する能力とは分けて考える必要があります。[複数対象の処理](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/gui_components/drag_drop_handler.py#L255)

後で検証したいこと：

- 異なる日本語タイトル、全角・半角、汎用 exe 名での誤一致。
- タイトルと保存フォルダー名が異なるゲームの見落とし。
- 設定・アセットを含むフォルダーが、保存先より上位に来る頻度。
- 深さや除外条件による見落とし、手動修正に必要な操作数。

## 再調査用の参照先

- [探索ソース](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/save_path_finder.py)
- [候補判定の設定](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/config.py)
- [探索の公式説明](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/docs/How-Save-Search-Works.md)
- [ライセンス：GPLv3](https://github.com/Matteo842/SaveState/blob/3b550e0dc38c001c42931cf9f476b4d8194420b1/LICENSE)
