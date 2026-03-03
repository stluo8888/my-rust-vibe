# my-rust-vibe

本仓库包含一个简单的 RPA 工具示例，使用 Rhai 脚本与鼠标/屏幕交互。

> **重要**：本项目含有演示用的签名授权逻辑。请勿将私钥提交到仓库，
> 并确保在发布前使用安全的密钥管理策略。


This workspace has been reorganized to match the recommended structure:

## Running the demo
Place an icon image such as `start_btn.png` in the root of the repository (or adjust the path in `scripts/demo.rhai`).
Then simply run:

```bash
cd rpa-bin
cargo run --release
```

## 下一步 (Next Steps)

1. **提交当前更改**：建立干净的 commit，把这套签名授权逻辑和示例说明写入历史。
2. **移除所有私钥**：确保 `.gitignore` 中包含 `license.txt`/私钥文件，以免意外提交。
3. **许可证服务器**：
   * 仓库已包含 `license-server` 子项目，使用 Warp 实现简单 HTTP API。
   * 在你的生产服务器上部署该服务，设置 `LICENSE_PRIVATE_KEY` 环境变量。
   * 客户端购买后调用 `/issue` 端点获取签名许可证。
4. **扩展机器指纹**：`sys_info::os_type()` 只是占位符，考虑采集 MAC 地址、CPU 序列号等并混淆。
5. **测试覆盖**：
   * `rpa-core` 中已有四个单元测试。
   * `license-server` 包含一个集成测试验证 `/issue` 返回有效签名。
   * 继续在本地开发新逻辑时撰写对应测试。
6. **CI 配置**：仓库中新增了 `rust-ci.yml`，自动构建整个工作区、运行所有测试、扫描敏感字符串并执行 Clippy。
7. **打包与混淆**：对发布版本做简单 `strip` 和混淆，并将公钥植入客户端或通过配置下发。
8. **文档与发布**：完善 README、CHANGELOG，然后发布到 GitHub Releases、crate 或其他渠道。

按照这些步骤，你的商业软件授权流程会具备从签名生成到验证的完整闭环，且在 CI 中自动检测敏感信息，基本满足安全发布要求。

The binary will perform a license check and, upon success, execute `../scripts/demo.rhai` which uses `find_icon` and `move_mouse`.
更复杂的示例可以在 `scripts/automation.rhai` 找到，里面演示了：
- 鼠标移动/点击（`move_mouse`, `left_click`, `right_click`）
- 键盘输入与按键 (`type_text`, `press_key`)
- 延迟 (`sleep`)
- 截图 (`screenshot`)
- 查图并单击 (`find_and_click`)
- 文本读取/查找 (`read_text`, `find_text`)  **(需启用 feature="ocr" 并安装系统 tesseract)**

运行示例：
```bash
# 确保有 license.txt 或在开发模式下跳过验证
cd rpa-bin
cargo run
# 或手动调用脚本：
RHAI_DEBUG=1 cargo run --release -- "../scripts/automation.rhai"
```

> **OCR 注意**：如果要使用 `read_text`/`find_text`，须先在系统上安装 Tesseract。
> 在 Ubuntu/容器中，例如：
> ```sh
> sudo apt-get update && sudo apt-get install -y tesseract-ocr libtesseract-dev libleptonica-dev
> ```
> 然后通过 `--features ocr` 编译/测试仓库：
> ```sh
> cargo build --workspace --features ocr
> cargo test --workspace --features ocr
> ```
> CI 工作流已在安装依赖并使用该特性。


```
my-rust-vibe/
├── .gitignore      # core defense: hide private files
├── Cargo.toml      # workspace configuration
├── rpa-core/       # [Public] core engine code (Action defs, Rhai init)
│   └── src/
│       ├── lib.rs   # define Action logic
│       └── engine.rs # register Rhai functions
├── rpa-bin/        # [Public] final CLI or UI entry
│   └── src/main.rs # include hardware validation logic
├── scripts/        # [GitIgnore] store private scripts you write for clients
└── .env            # [GitIgnore] store private keys or server addresses
```
