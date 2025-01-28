use git_cryptx::{commands, util}; // 使用 git_cryptx 作为 crate 名称
use toml::Value;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        util::log_error("未提供命令参数");
        std::process::exit(1);
    }

    let command = &args[0];
    let parameters = &args[1..];

    // 使用 include_str! 宏在编译时加载配置文件
    let config_content = include_str!("../config.toml");
    let config: Value = config_content.parse::<Value>().expect("配置文件格式错误");
    let mut default_language = config["settings"]["default_language"]
        .as_str()
        .unwrap_or("en")
        .to_string();

    // Ensure the language identifier is in the correct format
    if default_language != "en" && default_language != "zh" {
        default_language = "en".to_string();
    }

    commands::handle_command(command, parameters, &default_language);
}
