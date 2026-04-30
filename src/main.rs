use anyhow::{Context, Result};
use serde::Deserialize;
use wezel_types::{ForagerPluginEnvelope, ForagerPluginOutput};

#[derive(Deserialize)]
struct FilesizeInputs {
    glob: String,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--schema") {
        println!(
            "{}",
            serde_json::json!({
                "name": "filesize",
                "description": "Reports byte size of every file matching a glob",
                "inputs": {
                    "glob": { "type": "string", "description": "Glob pattern, evaluated against the experiment workspace cwd" }
                },
                "output": {
                    "description": "One measurement per matched file. `name` is the matched path (relative to cwd), `value` is the size in bytes."
                }
            })
        );
        return Ok(());
    }

    let out_path = std::env::var("FORAGER_OUT").context("FORAGER_OUT not set")?;
    let inputs_path = std::env::var("FORAGER_INPUTS").context("FORAGER_INPUTS not set")?;
    let inputs: FilesizeInputs = serde_json::from_str(
        &std::fs::read_to_string(&inputs_path).with_context(|| format!("reading {inputs_path}"))?,
    )
    .context("parsing FORAGER_INPUTS")?;

    let mut measurements = Vec::new();
    for entry in glob::glob(&inputs.glob).with_context(|| format!("invalid glob: {}", inputs.glob))?
    {
        let path = entry.with_context(|| format!("error walking glob {}", inputs.glob))?;
        let metadata = std::fs::metadata(&path)
            .with_context(|| format!("stat {}", path.display()))?;
        if !metadata.is_file() {
            continue;
        }
        measurements.push(ForagerPluginOutput {
            name: path.to_string_lossy().into_owned(),
            value: serde_json::json!(metadata.len()),
            tags: Default::default(),
        });
    }

    let envelope = ForagerPluginEnvelope { measurements };
    std::fs::write(&out_path, serde_json::to_string(&envelope)?)
        .with_context(|| format!("writing {out_path}"))?;

    Ok(())
}
