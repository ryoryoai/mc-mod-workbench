# MC Mod Workbench

子ども向け Minecraft Fabric 1.20.1 MOD 作成ツール（Tauri + React）。

## Mods ディレクトリ（規約）
- `~/mc-mods/` に MOD プロジェクトを配置する想定（パスはアプリ上で変更可能）
- 共有実行ディレクトリ: `shared-run/`（ワールド永続化）

## テクスチャ生成ルール
- 必ず Python (PIL/Pillow) で作成する
- nanobanana / AI 画像生成は使わない
- 16x16 RGBA PNG、透明背景、ドット打ちで直接描画

## Java / Gradle 環境
- Java 17 が必要（JAVA_HOME を設定）
- fabric-loom 1.5.4 は Gradle 8.x 必須（9.x 非対応）

## Fabric Mixin Tips
- Mixin 作成前に `./gradlew genSources` でメソッド名を必ず検証
- EnderDragonEntity: `tick()` ではなく `tickMovement()` を使う
- `canAddPassenger()` は Entity から継承、オーバーライドなし → `startRiding(entity, true)` で強制搭乗
- "No refMap loaded" = ターゲットメソッドが存在しない

## マルチMOD開発
- `./gradlew runClient` は自プロジェクトのみロード
- 他MODを同時ロードするには build.gradle に `modRuntimeOnly files(...)` を追加
