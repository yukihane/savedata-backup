# OpenSave：共通の保存配置とデータベースを併用する

調査日：2026-10-04。対象コミット：`346d9bda0749fb57b48fd266f13e535af80f0285`。

[調査概要](README.md) / [公式リポジトリ](https://github.com/Liquid-co/OpenSave)

## 調査対象

今回確認した版は Go による実装です。バックアップ・同期・履歴管理のうち、保存先の検出と候補表示に関係する処理を調べました。[対象版の README](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/README.md)

## 保存先を見つける仕組み

中心となる [scanner.go](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/scanner.go) は、複数の探索方法の結果を集めます。Windows の主な方法は次のとおりです。

| 探索方法 | 実装で確認した条件 |
| --- | --- |
| Unity の慣例 | `LocalLow/<会社>/<ゲーム>` を列挙。空の候補、既知の非ゲーム系メーカー、長い16進数名のキャッシュ候補を除く |
| Unreal の慣例 | `LocalAppData/<ゲーム>/Saved/SaveGames` の存在と内容を確認。インストール先の同様の配置も探す |
| Steam | ライブラリー情報とインストール情報からゲーム名・AppID・場所を取得し、userdata やゲーム内の保存フォルダーを探す |
| 保存領域の列挙 | Saved Games／Documents の My Games 配下を確認。メーカーとゲームの区別には既知のゲーム名も使う |
| ユーザー指定範囲 | 指定された場所や子フォルダーを候補にし、保存フォルダーへの絞り込みを試みる |
| 既知のパス定義 | Ludusavi マニフェストのパスを展開し、存在する場所を追加する |

エミュレーターや Wine／Proton の探索もありますが、今回の詳細確認は Windows の通常のゲーム保存先を中心にしました。

### Unity：ゲーム単位の登録なしでも候補を列挙する

この経路は、exe を解析して Unity と判定するものではありません。LocalLow の会社・ゲームに相当する2階層を列挙します。ゲーム個別のデータベース情報がなくても動く経路です。一方、フォルダーがあることだけでセーブデータとは確定しません。[実装](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/scanner.go#L492)

Unity の公式仕様では、Windows の `persistentDataPath` は通常 `LocalLow/<companyname>/<productname>` です。この慣例が探索の根拠になります。ゲームが独自の保存先を使う場合は例外です。[Unity の公式仕様](https://docs.unity3d.com/ScriptReference/Application-persistentDataPath.html)

### ゲーム内の保存フォルダー：名前と構造を使う

ゲームのインストール先では、区切り文字と英字の大文字・小文字を正規化し、名前に `save` を含むフォルダーを探します。対象版では探索深度3、分岐数120、候補数4を上限にしています。`Saved/SaveGames` のような配置では、より具体的な子を選びます。[保存フォルダー探索](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/savedirs.go)

これらはその実装の制限値です。大量の分岐や候補数の上限を超える場所では、必要な保存先も見落とし得ます。

キャッシュ・ログなどは名前で除外します。また、ゲームをまとめる親フォルダーを細かい保存先に絞る処理は、候補が一意の場合に限って絞り込む経路があります。複数候補がある場合などには親が残り得るため、不要なデータの混入を完全には防ぎません。[除外条件](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/junkdirs.go) / [親の絞り込み](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/savedirs.go#L39)

### データベース：慣例探索とは独立した補完経路

Ludusavi マニフェストの変数やパスパターンを展開して、ディスク上にある保存先を探します。これは、個別に定義されたパスを使う方法です。先の Unity 等の一般的な配置探索とは区別できます。[マニフェスト探索](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/ludusavi.go#L94)

## 候補検出と計測を分けている

候補を作る処理とは別に、ファイル数・容量・最終更新日時を計測します。計測できなかった候補や、時間制限で途中までしか調べられなかった候補を、空と扱わない設計です。CLI のスキャン処理からも計測・空候補の除外が呼ばれています。[計測処理](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/savestats.go) / [CLI からの呼び出し](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/cliapp/cliapp.go#L356)

複数候補を同じゲームとしてまとめる処理では、親子の重複、並列の保存先、別の場所に残った保存先を区別しています。新しい更新日時の候補を主候補にする考え方もありますが、最新のファイルが進行状況の保存先である保証はありません。[候補のグループ化](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/group.go)

## 具体例

```text
%USERPROFILE%\AppData\LocalLow\同人サークル\星の冒険\progress.dat
%LOCALAPPDATA%\AdventureProject\Saved\SaveGames\slot1.sav
D:\Games\PuzzleGame\SaveData\slot1.dat
```

最初の2つはエンジンの慣例、最後は探索対象となったゲームフォルダー内の名前・構造を手掛かりにできます。完全な保存パスが exe 内に含まれている必要はありません。

一方、`D:\Games\PuzzleGame\progress.dat` のように、保存先らしいフォルダー名も既知のパス定義もない場合は、この説明だけでは発見を期待できません。

## テストコードで確認したシナリオ

以下はテストを読んで確認した設計意図です。今回テストを実行したわけではありません。

- Unity の会社・ゲーム階層を検出し、空・メーカー・ハッシュ名の候補を除く。[該当テスト](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/presets_test.go#L281)
- 保存フォルダーの子にあるログ・設定を含む親より、`Saved/SaveGames` を選ぶ。深度・分岐数の制限や空フォルダーも確認する。[該当テスト](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/savedirs_test.go)
- 計測が打ち切られてファイル数が0になった候補を、空と誤認して消さない。[該当テスト](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/internal/presets/truncated_test.go)

## 未登録ゲームへの見込みと限界

**考察：** 共通する保存配置の列挙は、個々のゲームを指定せずに候補を集める用途の参考になります。ただし、エンジンの慣例に沿った構造と、その場所にあるデータが前提です。日本語の実ゲームでの検出精度や、特殊な保存方法への対応範囲はまだ確認できていません。

後で検証したいこと：

- Unity のフォルダーにログ・設定しかない場合の誤検出。
- エンジンの標準配置を変更したゲームや、無関係な内部プロジェクト名の扱い。
- 保存先が複数あるゲームで、候補の絞り込みやグループ化が必要なデータを落とさないか。
- 指定範囲の親フォルダーが残った場合の余分なデータ量。
- 日本語名の識別・グループ化と、探索制限による見落とし。

ライセンスの参照先：[MIT License](https://github.com/Liquid-co/OpenSave/blob/346d9bda0749fb57b48fd266f13e535af80f0285/LICENSE)。
