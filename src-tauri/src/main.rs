#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};

#[derive(Debug, Deserialize)]
struct ActionPayload {
    #[serde(rename = "projectPath")]
    project_path: String,
    prompt: String,
    spec: String,
    #[serde(rename = "snapshotName")]
    snapshot_name: String,
    provider: String,
    #[serde(rename = "learningMode")]
    learning_mode: Option<bool>,
    #[serde(rename = "planDraft")]
    plan_draft: Option<String>,
    #[serde(rename = "approvedPlan")]
    approved_plan: Option<String>,
}

#[derive(Debug, Serialize)]
struct ActionResult {
    success: bool,
    output: String,
}

#[tauri::command]
fn run_workbench_action(action: String, payload: ActionPayload) -> Result<ActionResult, String> {
    let project_path = expand_tilde(&payload.project_path)?;

    let learning_mode = payload.learning_mode.unwrap_or(true);

    match action.as_str() {
        "generate" => generate_scaffold(&project_path, &payload.prompt, &payload.spec),
        "ai_plan" => run_ai_plan(&project_path, &payload.provider, &payload.prompt, learning_mode),
        "ai_refine" => run_ai_refine(
            &project_path,
            &payload.provider,
            &payload.prompt,
            &payload.spec,
            payload.plan_draft.as_deref().unwrap_or(""),
            learning_mode,
        ),
        "ai_execute" => run_ai_execute(
            &project_path,
            &payload.provider,
            &payload.prompt,
            &payload.spec,
            payload.approved_plan.as_deref().unwrap_or(""),
            learning_mode,
        ),
        "build" => run_gradle(&project_path, "build"),
        "run_client" => run_gradle(&project_path, "runClient"),
        "snapshot" => create_snapshot(&project_path, &payload.snapshot_name),
        "rollback" => rollback_snapshot(&project_path, &payload.snapshot_name),
        _ => Err("Unsupported action".to_string()),
    }
}

fn expand_tilde(input: &str) -> Result<PathBuf, String> {
    if let Some(stripped) = input.strip_prefix("~/") {
        let home = std::env::var("HOME").map_err(|_| "HOME not found")?;
        return Ok(PathBuf::from(home).join(stripped));
    }
    Ok(PathBuf::from(input))
}

fn generate_scaffold(project_path: &PathBuf, prompt: &str, spec: &str) -> Result<ActionResult, String> {
    fs::create_dir_all(project_path).map_err(|e| e.to_string())?;
    fs::create_dir_all(project_path.join("spec")).map_err(|e| e.to_string())?;
    fs::create_dir_all(project_path.join("generated")).map_err(|e| e.to_string())?;

    fs::write(project_path.join("spec/mod.spec.yaml"), spec).map_err(|e| e.to_string())?;
    fs::write(project_path.join("generated/prompt.txt"), prompt).map_err(|e| e.to_string())?;

    let readme = format!(
        "# Generated Mod Workspace\n\n## Prompt\n{}\n\n## Next\n- Run build\n- Run runClient\n- Iterate with prompt/spec\n",
        prompt
    );
    fs::write(project_path.join("generated/README.generated.md"), readme).map_err(|e| e.to_string())?;

    Ok(ActionResult {
        success: true,
        output: format!("Generated scaffold in {}", project_path.display()),
    })
}

fn run_ai_plan(
    project_path: &PathBuf,
    provider: &str,
    prompt: &str,
    learning_mode: bool,
) -> Result<ActionResult, String> {
    let style = if learning_mode {
        "Write in very easy Japanese for 小学2年生. Short sentences."
    } else {
        "Write in standard concise Japanese for developers."
    };

    let instruction = format!(
        "Create a plan only (no code changes).\n\nIdea:\n{}\n\n{}\nReturn with headings:\n1) なにをつくる？\n2) どうなったらせいこう？\n3) こまったらどうする？\n4) さいしょのいっぽ",
        prompt, style
    );
    run_ai_command(project_path, provider, &instruction)
}

fn run_ai_refine(
    project_path: &PathBuf,
    provider: &str,
    prompt: &str,
    spec: &str,
    draft_plan: &str,
    learning_mode: bool,
) -> Result<ActionResult, String> {
    let style = if learning_mode {
        "Write in very easy Japanese for 小学2年生."
    } else {
        "Write in standard concise Japanese for developers."
    };

    let instruction = format!(
        "Refine this mod plan for execution (no code changes).\n\nIdea:\n{}\n\nDraft plan:\n{}\n\nSpec:\n{}\n\n{}\nReturn checklist with boxes style:\n- [ ] ...\nAlso add one line:『ここを見ればOK』",
        prompt, draft_plan, spec, style
    );
    run_ai_command(project_path, provider, &instruction)
}

fn run_ai_execute(
    project_path: &PathBuf,
    provider: &str,
    prompt: &str,
    spec: &str,
    approved_plan: &str,
    learning_mode: bool,
) -> Result<ActionResult, String> {
    let style = if learning_mode {
        "After implementation, explain in easy Japanese for 小学2年生."
    } else {
        "After implementation, explain clearly for developers in Japanese."
    };

    let instruction = format!(
        "You are editing a Minecraft mod project. Apply approved plan directly in code.\n\nIdea:\n{}\n\nApproved Plan:\n{}\n\nSpec:\n{}\n\n{}\nInclude:\n1) どのファイルをかえたか\n2) なにができるようになったか\n3) つぎにためすこと",
        prompt, approved_plan, spec, style
    );
    run_ai_command(project_path, provider, &instruction)
}

fn run_ai_command(
    project_path: &PathBuf,
    provider: &str,
    instruction: &str,
) -> Result<ActionResult, String> {
    let (cmd, args): (&str, Vec<&str>) = match provider {
        "codex" => ("codex", vec!["exec", instruction]),
        _ => ("claude", vec!["-p", instruction]),
    };

    let output = Command::new(cmd)
        .args(args)
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run {} CLI: {}", cmd, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(ActionResult {
        success: output.status.success(),
        output: format!("{}\n{}", stdout, stderr),
    })
}

fn run_gradle(project_path: &PathBuf, task: &str) -> Result<ActionResult, String> {
    let gradlew = if cfg!(target_os = "windows") { "gradlew.bat" } else { "./gradlew" };
    let output = Command::new(gradlew)
        .arg(task)
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to run gradle command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(ActionResult {
        success: output.status.success(),
        output: format!("{}\n{}", stdout, stderr),
    })
}

fn create_snapshot(project_path: &PathBuf, name: &str) -> Result<ActionResult, String> {
    let snapshots = project_path.join(".workbench-snapshots");
    fs::create_dir_all(&snapshots).map_err(|e| e.to_string())?;
    let target = snapshots.join(name);

    if target.exists() {
        fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
    }
    copy_dir(project_path, &target, Some(&snapshots))?;

    Ok(ActionResult {
        success: true,
        output: format!("Snapshot created: {}", name),
    })
}

fn rollback_snapshot(project_path: &PathBuf, name: &str) -> Result<ActionResult, String> {
    let source = project_path.join(".workbench-snapshots").join(name);
    if !source.exists() {
        return Err(format!("Snapshot not found: {}", name));
    }

    let entries = fs::read_dir(project_path).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if file_name == ".workbench-snapshots" {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }

    copy_dir(&source, project_path, None)?;

    Ok(ActionResult {
        success: true,
        output: format!("Rolled back to snapshot: {}", name),
    })
}

fn copy_dir(from: &PathBuf, to: &PathBuf, exclude: Option<&PathBuf>) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if let Some(exclude_path) = exclude {
            if path == *exclude_path {
                continue;
            }
        }

        let dest = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &dest, exclude)?;
        } else {
            fs::copy(&path, &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_workbench_action])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
