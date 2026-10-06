use crate::Result;
use std::path::Path;
use std::process::Command;

pub fn learn(db_path: &Path, epochs: usize) -> Result<()> {
    let script = find_trainer_script()?;

    let model_dir = db_path.with_extension("db.jmesh-model");
    std::fs::create_dir_all(&model_dir)?;

    println!("[learn] Extracting schema from {}...", db_path.display());
    println!("[learn] Training neural adapter ({} epochs)...", epochs);

    let output = Command::new("python3")
        .arg(&script)
        .arg("--db")
        .arg(db_path)
        .arg("--output-dir")
        .arg(&model_dir)
        .arg("--epochs")
        .arg(epochs.to_string())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        return Err(crate::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Training script failed",
        )));
    }

    println!("[learn] Model saved to {}", model_dir.display());
    Ok(())
}

fn find_trainer_script() -> Result<std::path::PathBuf> {
    for candidate in ["scripts/tinygrad_trainer.py", "script/tinygrad_trainer.py"] {
        let path = std::path::PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(crate::Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "tinygrad_trainer.py not found — the phase-2 self-training script is not available yet",
    )))
}
