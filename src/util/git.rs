use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_git_root() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(PathBuf::from(path))
    } else {
        None
    }
}

pub fn ensure_git_cryptx_dir(git_root: &Path) -> Result<PathBuf, String> {
    let git_dir = git_root.join(".git");
    let encrypt_dir = git_dir.join("cryptx");
    let keys_dir = encrypt_dir.join("keys");

    fs::create_dir_all(&keys_dir).map_err(|e| format!("无法创建加密目录: {}", e))?;

    Ok(encrypt_dir)
}

pub fn get_key_path(git_root: &Path) -> PathBuf {
    git_root.join(".git/cryptx/keys/global_ase_key")
}

// 新增：配置 Git 过滤器
pub fn configure_git_filter(git_root: &Path) -> Result<(), String> {
    let configs = [
        ("filter.git-cryptx.clean", "git-cryptx clean %f"),
        ("filter.git-cryptx.smudge", "git-cryptx smudge %f"),
        ("filter.git-cryptx.required", "true"),
        ("diff.git-cryptx.textconv", "git-cryptx diff"),
    ];

    for (key, value) in configs.iter() {
        let output = Command::new("git")
            .args(["config", key, value])
            .current_dir(git_root)
            .output()
            .map_err(|e| format!("无法执行 git config: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "无法配置 Git {}: {}",
                key,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(())
}

// 新增：检查 Git 过滤器配置
pub fn check_git_filter(git_root: &Path) -> bool {
    let configs = [
        "filter.git-cryptx.clean",
        "filter.git-cryptx.smudge",
        "filter.git-cryptx.required",
        "diff.git-cryptx.textconv",
    ];

    for key in configs.iter() {
        let output = Command::new("git")
            .args(["config", "--get", key])
            .current_dir(git_root)
            .output();

        if output.is_err() || !output.unwrap().status.success() {
            return false;
        }
    }

    true
}

pub fn is_truly_modified(path: &Path) -> bool {
    // 获取文件的 Git 状态
    let output = Command::new("git")
        .args(["diff", "--no-ext-diff", "--quiet", path.to_str().unwrap()])
        .output();

    match output {
        Ok(output) => !output.status.success(), // 如果命令返回非0，说明文件确实被修改
        Err(_) => true,                         // 如果无法执行命令，保守起见认为文件被修改
    }
}

pub fn reset_file(path: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", "--", path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("无法重置文件: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "重置文件失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}
