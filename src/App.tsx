import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  CheckCircle2,
  Hammer,
  Lightbulb,
  Play,
  Rocket,
  RotateCcw,
  Sparkles,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";

type CommandResult = {
  success: boolean;
  output: string;
};

type Action =
  | "generate"
  | "ai_plan"
  | "ai_refine"
  | "ai_execute"
  | "build"
  | "run_client"
  | "snapshot"
  | "rollback";

const defaultSpec = `name: Speed Candy\nmodId: speedcandy\nmcVersion: 1.20.1\nloader: fabric\nfeatures:\n  - type: consumable\n    id: speed_candy\n    effect: speed\n    amplifier: 1\n    durationSeconds: 60\n  - type: ui\n    id: tweak_panel\n`;

export default function App() {
  const [projectPath, setProjectPath] = useState("~/mc-mods/speedcandy");
  const [provider, setProvider] = useState<"claude" | "codex">("claude");
  const [learningMode, setLearningMode] = useState(true);

  const [idea, setIdea] = useState(
    "足が速くなるキャンディを作って。子ども向けにわかりやすいUIで秒数を調整できるように。",
  );
  const [planDraft, setPlanDraft] = useState("");
  const [approvedPlan, setApprovedPlan] = useState("");
  const [spec, setSpec] = useState(defaultSpec);
  const [snapshotName, setSnapshotName] = useState("before-change");

  const [logs, setLogs] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);

  const logText = useMemo(() => logs.join("\n"), [logs]);

  async function run(action: Action) {
    setBusy(true);
    setLogs((prev) => [...prev, `\n> ${action}`]);
    try {
      const result = await invoke<CommandResult>("run_workbench_action", {
        action,
        payload: {
          projectPath,
          prompt: idea,
          spec,
          snapshotName,
          provider,
          learningMode,
          planDraft,
          approvedPlan,
        },
      });
      setLogs((prev) => [...prev, result.output]);

      if (action === "ai_plan") {
        setPlanDraft(result.output);
      }
      if (action === "ai_refine") {
        setApprovedPlan(result.output);
      }
    } catch (error) {
      setLogs((prev) => [...prev, `ERROR: ${String(error)}`]);
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="min-h-screen bg-slate-950 p-6 text-slate-100">
      <div className="mx-auto grid max-w-7xl gap-4 lg:grid-cols-[1.2fr_1fr]">
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Sparkles size={16} /> こどもMODこうぼう
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <label className="mb-1 block text-xs text-slate-300">Project Path</label>
                <Input value={projectPath} onChange={(e) => setProjectPath(e.target.value)} />
              </div>
              <div className="grid grid-cols-[1fr_auto] items-center gap-2">
                <label className="text-xs text-slate-300">AI Provider</label>
                <select
                  className="rounded-md border border-slate-700 bg-slate-950 px-2 py-1 text-sm"
                  value={provider}
                  onChange={(e) => setProvider(e.target.value as "claude" | "codex")}
                >
                  <option value="claude">Claude CLI</option>
                  <option value="codex">Codex CLI</option>
                </select>
              </div>
              <div className="grid grid-cols-[1fr_auto] items-center gap-2">
                <label className="text-xs text-slate-300">学習モード</label>
                <button
                  type="button"
                  onClick={() => setLearningMode((v) => !v)}
                  className="rounded-md border border-slate-700 bg-slate-950 px-3 py-1 text-sm hover:bg-slate-800"
                >
                  {learningMode ? "ON（やさしい説明）" : "OFF（ふつう）"}
                </button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Lightbulb size={16} /> ステップ1：かんがえる
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <label className="mb-1 block text-xs text-slate-300">つくりたいもの（もじでOK）</label>
              <Textarea
                rows={4}
                value={idea}
                onChange={(e) => setIdea(e.target.value)}
                onKeyDown={(e) => {
                  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") run("ai_plan");
                }}
              />
              <Button disabled={busy} onClick={() => run("ai_plan")}>
                <Sparkles className="mr-2 h-4 w-4" /> けいかくを つくる
              </Button>
              <Textarea
                rows={8}
                value={planDraft}
                onChange={(e) => setPlanDraft(e.target.value)}
                placeholder="ここに けいかく が でるよ"
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CheckCircle2 size={16} /> ステップ2：ととのえる
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <label className="mb-1 block text-xs text-slate-300">ルール（Spec YAML）</label>
              <Textarea rows={8} value={spec} onChange={(e) => setSpec(e.target.value)} />
              <Button disabled={busy} variant="secondary" onClick={() => run("ai_refine")}>
                <Sparkles className="mr-2 h-4 w-4" /> けいかくを もっとよくする
              </Button>
              <Textarea
                rows={8}
                value={approvedPlan}
                onChange={(e) => setApprovedPlan(e.target.value)}
                placeholder="さいしゅう けいかく（これでいく！）"
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Hammer size={16} /> ステップ3：つくる
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid grid-cols-2 gap-2">
                <Button disabled={busy} onClick={() => run("generate")}>
                  <Hammer className="mr-2 h-4 w-4" /> ひな形をつくる
                </Button>
                <Button disabled={busy} variant="secondary" onClick={() => run("ai_execute")}>
                  <Play className="mr-2 h-4 w-4" /> けいかくどおり つくる
                </Button>
                <Button disabled={busy} variant="secondary" onClick={() => run("build")}>
                  <Play className="mr-2 h-4 w-4" /> Build
                </Button>
                <Button disabled={busy} variant="secondary" onClick={() => run("run_client")}>
                  <Rocket className="mr-2 h-4 w-4" /> Run Client
                </Button>
                <Button disabled={busy} variant="ghost" onClick={() => run("snapshot")}>
                  <RotateCcw className="mr-2 h-4 w-4" /> Snapshot
                </Button>
              </div>
              <div className="grid grid-cols-[1fr_auto] gap-2">
                <Input value={snapshotName} onChange={(e) => setSnapshotName(e.target.value)} />
                <Button disabled={busy} variant="ghost" onClick={() => run("rollback")}>
                  Rollback
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Logs</CardTitle>
          </CardHeader>
          <CardContent>
            <Textarea className="font-mono text-xs" rows={45} value={logText} readOnly />
          </CardContent>
        </Card>
      </div>
    </main>
  );
}
