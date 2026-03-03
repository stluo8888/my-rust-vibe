use std::fs;

// 使用 rpa-core 提供的许可工具
use rpa_core::license::{verify_license, make_license};

fn main() {
    let mid = sys_info::os_type().unwrap_or_default(); // placeholder 为唯一硬件 id

    // 命令行工具支持：`gen-license [ID]` 会生成签名的 license.txt，
    // 私钥由 `LICENSE_PRIVATE_KEY` 环境变量提供。
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "gen-license" {
        let lic_mid = if args.len() > 2 { args[2].clone() } else { mid.clone() };
        let key_b64 = std::env::var("LICENSE_PRIVATE_KEY").expect("请设置 LICENSE_PRIVATE_KEY 环境变量以生成许可证");
        let lic = make_license(&lic_mid, &key_b64);
        let json = serde_json::to_string(&lic).unwrap();
        let used_pub = std::env::var("LICENSE_PUBLIC_KEY").unwrap_or_else(|_| String::from("<embedded_pub_key>"));
        match fs::write("license.txt", &json) {
            Ok(()) => println!("signed license.txt 已生成, mid={} json={} pub={}", lic_mid, json, used_pub),
            Err(e) => eprintln!("生成 license 文件失败：{}", e),
        }
        return;
    }

    // CI or developers may want to bypass license validation with an env var.
    if std::env::var("RPA_SKIP_LICENSE").is_err() {
        if !verify_license(&mid, "license.txt") {
            eprintln!("错误: 未检测到有效授权! 请联系作者获取许可。机器 ID={}", mid);
            eprintln!("你可以运行 `rpa-bin gen-license` 在本机创建许可证文件");
            return;
        }
    } else {
        println!("RPA_SKIP_LICENSE set; skipping license check");
    }

    // 只有校验通过，才会启动 RHAI 引擎
    let engine = rpa_core::engine::init_engine();
    println!("授权通过，rpa-bin 启动成功");

    // 检查命令行参数是否指定了脚本路径
    let script_path = if args.len() > 1 && !args[1].starts_with("gen-") {
        args[1].clone()
    } else {
        // 默认运行 demo.rhai
        String::from("../scripts/demo.rhai")
    };

    if std::path::Path::new(&script_path).exists() {
        println!("执行脚本：{}", script_path);
        if let Err(e) = engine.eval_file::<()>(script_path.into()) {
            eprintln!("脚本执行失败：{}", e);
        }
    } else {
        println!("脚本不存在：{}", script_path);
    }
}
