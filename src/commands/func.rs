use crate::{crypto::Encryptor, util};
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use toml::Value;

fn load_website_url() -> String {
    let config_content = include_str!("../../config.toml");
    let config: Value = config_content.parse::<Value>().expect("配置文件格式错误");
    config["settings"]["website_url"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

pub fn handle_command(command_str: &str, parameters: &[String], language: &str) {
    let bundle = util::load_locale(language);
    let mut args = FluentArgs::new();

    match command_str {
        "help" => help(parameters, &bundle, &mut args),
        "version" => version(&bundle),
        "init" => init(&bundle),
        "set-key" => add_key(parameters, &bundle),
        "rm-key" => remove_key(parameters, &bundle),
        "status" => status(&bundle),
        "clean" => clean(parameters, &bundle),
        "smudge" => smudge(parameters, &bundle),
        "diff" => diff(parameters, &bundle),
        "reset" => reset_files(parameters, &bundle),
        _ => util::log_error("Unknown command"),
    }
}

fn help(_parameters: &[String], bundle: &FluentBundle<FluentResource>, args: &mut FluentArgs) {
    let website_url = load_website_url();
    let mut errors = vec![];
    println!(
        "{}: {}",
        util::format_pattern(bundle, "website-label", &mut errors),
        website_url
    );
    println!(
        "{}:",
        util::format_pattern(bundle, "usage-label", &mut errors)
    );
    println!("  git-cryptx <command> [arguments]\n");
    println!(
        "{}:",
        util::format_pattern(bundle, "available-commands-label", &mut errors)
    );

    // List all commands
    let commands = vec!["help", "version", "init", "set-key", "rm-key", "status"];
    for command in commands {
        let key = format!("{}-command", command);
        let msg = bundle.get_message(&key).expect("Message not found");
        let pattern = msg.value().expect("Message has no value");
        let value = bundle.format_pattern(pattern, Some(args), &mut errors);
        println!("  {} - {}", command, value);
    }
}

fn version(bundle: &FluentBundle<FluentResource>) {
    let website_url = load_website_url();
    let mut errors = vec![];
    let value = util::format_pattern(bundle, "version-command", &mut errors);

    println!("{}: v{}", value, env!("CARGO_PKG_VERSION"));
    println!(
        "{}: {}",
        util::format_pattern(bundle, "website-label", &mut errors),
        website_url
    );
}

fn init(bundle: &FluentBundle<FluentResource>) {
    let website_url = load_website_url();
    let mut errors = vec![];
    let value = util::format_pattern(bundle, "init-command", &mut errors);

    println!("{}", value);
    println!(
        "{}: {}",
        util::format_pattern(bundle, "website-label", &mut errors),
        website_url
    );

    let git_root = match util::find_git_root() {
        Some(path) => path,
        None => {
            let mut errors = vec![];
            util::log_error(&util::format_pattern(
                bundle,
                "not-git-repo-error",
                &mut errors,
            ));
            return;
        }
    };

    let gitattributes_path = git_root.join(".gitattributes");
    let default_config = "*.secret filter=git-cryptx diff=git-cryptx";

    if gitattributes_path.exists() {
        let content = match fs::read_to_string(&gitattributes_path) {
            Ok(content) => content,
            Err(e) => {
                let mut errors = vec![];
                util::log_error(
                    &util::format_pattern(bundle, "read-gitattributes-error", &mut errors)
                        .replace("{}", &e.to_string()),
                );
                return;
            }
        };

        let has_filter = content
            .lines()
            .any(|line| line.contains("filter=git-cryptx") && line.contains("diff=git-cryptx"));

        if has_filter {
            println!(
                "{}",
                util::format_pattern(bundle, "git-cryptx-filter-configured", &mut errors)
            );
        } else {
            // 添加默认配置
            if let Err(e) = fs::OpenOptions::new()
                .append(true)
                .open(&gitattributes_path)
                .and_then(|mut file| {
                    std::io::Write::write_all(&mut file, default_config.as_bytes())
                })
            {
                let mut errors = vec![];
                util::log_error(
                    &util::format_pattern(bundle, "update-gitattributes-error", &mut errors)
                        .replace("{}", &e.to_string()),
                );
                return;
            }
            println!(
                "{}",
                util::format_pattern(bundle, "add-git-cryptx-config", &mut errors)
            );
        }
    } else {
        // 创建新文件，使用默认配置
        if let Err(e) = fs::write(&gitattributes_path, default_config) {
            let mut errors = vec![];
            util::log_error(
                &util::format_pattern(bundle, "create-gitattributes-error", &mut errors)
                    .replace("{}", &e.to_string()),
            );
            return;
        }
        println!(
            "{}",
            util::format_pattern(bundle, "create-gitattributes-success", &mut errors)
        );
    }

    // 配置 Git 过滤器
    if let Err(e) = util::configure_git_filter(&git_root) {
        let mut errors = vec![];
        util::log_error(
            &util::format_pattern(bundle, "configure-git-filter-error", &mut errors)
                .replace("{}", &e.to_string()),
        );
        return;
    }

    println!(
        "{}",
        util::format_pattern(bundle, "init-success", &mut errors)
    );
}

fn add_key(parameters: &[String], bundle: &FluentBundle<FluentResource>) {
    if parameters.is_empty() {
        let mut errors = vec![];
        let value = util::format_pattern(bundle, "set-key-empty-error", &mut errors);
        util::log_error(&value);
        return;
    }

    let key = &parameters[0];
    if key.len() < 8 {
        let mut errors = vec![];
        let value = util::format_pattern(bundle, "set-key-length-error", &mut errors);
        util::log_error(&value);
        return;
    }

    // 获取 Git 仓库根目录
    let git_root = match util::find_git_root() {
        Some(path) => path,
        None => {
            let mut errors = vec![];
            util::log_error(&util::format_pattern(
                bundle,
                "not-git-repo-error",
                &mut errors,
            ));
            return;
        }
    };

    // 确保 .git/cryptx 目录存在
    let encrypt_dir = match util::ensure_git_cryptx_dir(&git_root) {
        Ok(dir) => dir,
        Err(e) => {
            let mut errors = vec![];
            util::log_error(
                &util::format_pattern(bundle, "ensure-git-cryptx-dir-error", &mut errors)
                    .replace("{}", &e.to_string()),
            );
            return;
        }
    };

    // 检查密钥文件是否已存在
    let key_file = encrypt_dir.join("keys").join("global_ase_key");
    if key_file.exists() {
        let mut errors = vec![];
        let value = util::format_pattern(bundle, "set-key-exists-error", &mut errors);
        util::log_error(&value);
        return;
    }

    // 将密钥写入文件
    if let Err(e) = fs::write(&key_file, key) {
        let mut errors = vec![];
        let value = util::format_pattern(bundle, "set-key-write-error", &mut errors);
        util::log_error(&format!("{}: {}", value, e));
        return;
    }

    let mut errors = vec![];
    let value = util::format_pattern(bundle, "set-key-success", &mut errors);
    println!("{}", value);
}

fn remove_key(_parameters: &[String], bundle: &FluentBundle<FluentResource>) {
    // 获取 Git 仓库根目录
    let git_root = match util::find_git_root() {
        Some(path) => path,
        None => {
            let mut errors = vec![];
            util::log_error(&util::format_pattern(
                bundle,
                "not-git-repo-error",
                &mut errors,
            ));
            return;
        }
    };

    // 使用新的密钥路径
    let key_file = util::get_key_path(&git_root);
    if !key_file.exists() {
        let mut errors = vec![];
        let value = util::format_pattern(bundle, "rm-key-not-exists-error", &mut errors);
        util::log_error(&value);
        return;
    }

    if let Err(e) = fs::remove_file(&key_file) {
        let mut errors = vec![];
        let value = util::format_pattern(bundle, "rm-key-remove-error", &mut errors);
        util::log_error(&format!("{}: {}", value, e));
        return;
    }

    let mut errors = vec![];
    let value = util::format_pattern(bundle, "rm-key-success", &mut errors);
    println!("{}", value);
}

fn status(bundle: &FluentBundle<FluentResource>) {
    let git_root = match util::find_git_root() {
        Some(path) => path,
        None => {
            let mut errors = vec![];
            util::log_error(&util::format_pattern(
                bundle,
                "not-git-repo-error",
                &mut errors,
            ));
            return;
        }
    };

    let mut issues = Vec::new();

    // 检查密钥
    let mut errors = vec![];
    if !util::get_key_path(&git_root).exists() {
        issues.push(util::format_pattern(
            bundle,
            "key-not-configured",
            &mut errors,
        ));
    }

    // 检查 .gitattributes
    if !git_root.join(".gitattributes").exists() {
        issues.push(util::format_pattern(
            bundle,
            "gitattributes-not-configured",
            &mut errors,
        ));
    }

    // 检查 Git 过滤器配置
    if !util::check_git_filter(&git_root) {
        issues.push(util::format_pattern(
            bundle,
            "git-filter-not-configured",
            &mut errors,
        ));
    }

    if issues.is_empty() {
        let value = util::format_pattern(bundle, "status-ok", &mut errors);
        println!("{}", value);
        println!(
            "  {}",
            util::format_pattern(bundle, "key-configured", &mut errors)
        );
        println!(
            "  {}",
            util::format_pattern(bundle, "git-filter-configured", &mut errors)
        );
    } else {
        let value = util::format_pattern(bundle, "status-issue", &mut errors);
        println!("{}", value);
        for issue in issues {
            println!("  {}", issue);
        }
    }
}

fn clean(parameters: &[String], _bundle: &FluentBundle<FluentResource>) {
    if parameters.is_empty() {
        util::log_error("clean-error");
        return;
    }

    let file_path = Path::new(&parameters[0]);
    let git_root = match util::find_git_root() {
        Some(path) => path,
        None => {
            util::log_error("not-git-repo-error");
            return;
        }
    };

    // 读取文件内容
    let content = match fs::read(file_path) {
        Ok(content) => content,
        Err(e) => {
            util::log_error(&format!("{}: {}", "clean-read-error", e));
            return;
        }
    };

    // 获取密钥
    let key = match fs::read(util::get_key_path(&git_root)) {
        Ok(key) => key,
        Err(_) => {
            // 如果没有密钥，直接输出原内容
            io::stdout().write_all(&content).unwrap();
            return;
        }
    };

    // 创建加密器
    let encryptor = match Encryptor::new(&key) {
        Ok(e) => e,
        Err(e) => {
            util::log_error(&format!("{}: {}", "clean-encryptor-error", e));
            return;
        }
    };

    // 加密内容并输出到标准输出
    match encryptor.encrypt(&content) {
        Ok(encrypted) => {
            std::io::stdout().write_all(&encrypted).unwrap();
        }
        Err(e) => {
            util::log_error(&format!("{}: {}", "clean-encrypt-error", e));
        }
    }
}

fn smudge(parameters: &[String], _bundle: &FluentBundle<FluentResource>) {
    if parameters.is_empty() {
        if let Err(_) = io::copy(&mut io::stdin(), &mut io::stdout()) {
            eprintln!("{}", "smudge-error");
            std::process::exit(1);
        }
        return;
    }

    let mut content = Vec::new();
    if let Err(_) = std::io::stdin().read_to_end(&mut content) {
        eprintln!("{}", "smudge-read-error");
        std::process::exit(1);
    }

    // 如果内容不是加密的，直接输出
    if !Encryptor::is_encrypted(&content) {
        if let Err(_) = io::stdout().write_all(&content) {
            eprintln!("{}", "smudge-write-error");
            std::process::exit(1);
        }
        return;
    }

    let git_root = match util::find_git_root() {
        Some(path) => path,
        None => {
            eprintln!("{}", "smudge-not-git-error");
            if let Err(_) = io::stdout().write_all(&content) {
                std::process::exit(1);
            }
            return;
        }
    };

    // 获取密钥
    let key = match fs::read(util::get_key_path(&git_root)) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("{}: {}", "smudge-key-error", e);
            if let Err(_) = io::stdout().write_all(&content) {
                std::process::exit(1);
            }
            return;
        }
    };

    // 创建加密器
    let encryptor = match Encryptor::new(&key) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("{}: {}", "smudge-encryptor-error", e);
            if let Err(_) = io::stdout().write_all(&content) {
                std::process::exit(1);
            }
            return;
        }
    };

    // 解密内容
    match encryptor.decrypt(&content) {
        Ok(decrypted) => {
            if let Err(_) = io::stdout().write_all(&decrypted) {
                // 如果写入失败，返回原始内容而不是退出
                if let Err(_) = io::stdout().write_all(&content) {
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            // 解密失败时，静默返回原始内容
            if let Err(_) = io::stdout().write_all(&content) {
                std::process::exit(1);
            }
        }
    }
}

fn diff(parameters: &[String], bundle: &FluentBundle<FluentResource>) {
    if parameters.is_empty() {
        util::log_error("diff-error");
        return;
    }

    let file_path = Path::new(&parameters[0]);
    let git_root = match util::find_git_root() {
        Some(path) => path,
        None => {
            let mut errors = vec![];
            util::log_error(&util::format_pattern(
                bundle,
                "not-git-repo-error",
                &mut errors,
            ));
            return;
        }
    };

    // 读取文件内容
    let content = match fs::read(file_path) {
        Ok(content) => content,
        Err(e) => {
            util::log_error(&format!("{}: {}", "diff-read-error", e));
            return;
        }
    };

    // 如果内容不是加密的，直接输出
    if !Encryptor::is_encrypted(&content) {
        io::stdout().write_all(&content).unwrap();
        return;
    }

    // 获取密钥
    let key = match fs::read(util::get_key_path(&git_root)) {
        Ok(key) => key,
        Err(_) => {
            // 如果没有密钥，输出提示信息
            println!("{}", "diff-key-not-exists");
            return;
        }
    };

    // 创建加密器
    let encryptor = match Encryptor::new(&key) {
        Ok(e) => e,
        Err(e) => {
            util::log_error(&format!("{}: {}", "diff-encryptor-error", e));
            return;
        }
    };

    // 解密内容并输出到标准输出
    match encryptor.decrypt(&content) {
        Ok(decrypted) => {
            std::io::stdout().write_all(&decrypted).unwrap();
        }
        Err(e) => {
            util::log_error(&format!("{}: {}", "diff-decrypt-error", e));
        }
    }
}

fn reset_files(parameters: &[String], _bundle: &FluentBundle<FluentResource>) {
    if parameters.is_empty() {
        util::log_error("reset-error");
        return;
    }

    let path = Path::new(&parameters[0]);

    // 检查文件是否真的被修改
    if !util::is_truly_modified(path) {
        // 如果文件没有真正被修改，只是解密状态不同
        if let Err(e) = util::reset_file(path) {
            util::log_error(&format!("{}: {}", "reset-file-error", e));
            return;
        }
        println!("{}", "reset-file-success");
    } else {
        println!("{}", "reset-file-modified");
    }
}
