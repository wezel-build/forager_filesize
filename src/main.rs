use anyhow::{Context, Result};
use forager_sdk::Forager;
use schemars::JsonSchema;
use serde::Deserialize;
use forager_sdk::ForagerPluginOutput;

#[derive(Deserialize, JsonSchema)]
struct FilesizeInputs {
    /// Glob pattern, evaluated against the experiment workspace cwd.
    glob: String,
}

struct Filesize;

impl Forager for Filesize {
    const NAME: &'static str = "filesize";
    const DESCRIPTION: &'static str = "Reports byte size of every file matching a glob";
    const OUTCOMES_DOC: &'static str = "One outcome per matched file. The outcome name is the file's path \
         (relative to cwd); the value is its size in bytes.";
    type Inputs = FilesizeInputs;

    fn run(inputs: FilesizeInputs) -> Result<Vec<ForagerPluginOutput>> {
        let mut outcomes = Vec::new();
        for entry in
            glob::glob(&inputs.glob).with_context(|| format!("invalid glob: {}", inputs.glob))?
        {
            let path = entry.with_context(|| format!("error walking glob {}", inputs.glob))?;
            let metadata =
                std::fs::metadata(&path).with_context(|| format!("stat {}", path.display()))?;
            if !metadata.is_file() {
                continue;
            }
            outcomes.push(ForagerPluginOutput {
                name: path.to_string_lossy().into_owned(),
                value: serde_json::json!(metadata.len()),
                tags: Default::default(),
            });
        }
        Ok(outcomes)
    }
}

forager_sdk::forager_main!(Filesize);
