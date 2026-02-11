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
}

#[derive(Debug, Serialize)]
struct ActionResult {
    success: bool,
    output: String,
}

#[tauri::command]
fn run_workbench_action(action: String, payload: ActionPayload) -> Result<ActionResult, String> {
    let project_path = expand_tilde(&payload.project_path)?;

    match action.as_str() {
        "generate" => generate_scaffold(&project_path, &payload.prompt, &payload.spec),
        "ai_generate" => run_ai_generate(&project_path, &payload.provider, &payload.prompt, &payload.spec),
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

fn run_ai_generate(
    project_path: &PathBuf,
    provider: &str,
    prompt: &str,
    spec: &str,
) -> Result<ActionResult, String> {
    let instruction = format!(
        "You are editing a Minecraft mod project.\nUser prompt:\n{}\n\nSpec:\n{}\n\nApply changes directly in this project and explain what you changed.",
        prompt, spec
    );

    let (cmd, args): (&str, Vec<&str>) = match provider {
        "codex" => ("codex", vec!["exec", &instruction]),
        _ => ("claude", vec!["-p", &instruction]),
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
