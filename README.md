# MC Mod Workbench

Tauri + React + shadcn UI app for a **CLI-first Minecraft mod workflow**.

## What is implemented

- Prompt + Spec editor UI
- Actions:
  - Generate scaffold files from prompt/spec
  - Gradle build
  - Gradle runClient
  - Snapshot
  - Rollback
- Cross-platform gradle command switching (`./gradlew` / `gradlew.bat`)
- Snapshot/rollback storage under `.workbench-snapshots`

## Stack

- Tauri v2 (Rust backend)
- React + Vite + TypeScript
- Tailwind + shadcn-style components

## Quick start

```bash
cd projects/mc-mod-workbench
npm install
npm run tauri:dev
```

## Usage

1. Set `Project Path` to your Fabric mod project root (where `gradlew` exists).
2. Enter voice/text prompt + spec YAML.
3. Click **Generate**.
4. Click **Build** and/or **Run Client**.
5. Use **Snapshot** before risky edits, **Rollback** to restore.

## Notes

- No API key flow is embedded. Integrate your authenticated Codex/Claude CLI separately as next step.
- Current `Generate` writes scaffold files (`spec/mod.spec.yaml`, `generated/*`). You can extend this with CLI-driven code synthesis.
