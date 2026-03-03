// 示例 Rhai 脚本，展示 RPA API 用法
// 1. 移动鼠标
print("步骤 1: 移动鼠标到 (100, 100)");
move_mouse(100, 100);
// 2. 左键点击
print("步骤 2: 左键点击");
left_click();
// 3. 输入文字
print("步骤 3: 睡眠 500ms");
sleep(500);
print("步骤 4: 移动鼠标到 (200, 200)");
move_mouse(200,200);
print("步骤 5: 左键点击");
left_click();
print("步骤 6: 输入文本 'Hello from Rhai!'");
type_text("Hello from Rhai!");
// 4. 按键
print("步骤 7: 按 Enter 键");
press_key("enter");
print("步骤 8: 查找并点击图标");
let r = find_and_click("../icons/ok_button.png");
if r {
    print("  ✓ 成功找到并点击图标");
    sleep(200);
    type_text("Found and clicked!");
} else {
    print("  ✗ 未找到图标");
}
// 6. 截图保存
print("步骤 9: 保存屏幕截图到 demo_capture.png");
screenshot("demo_capture.png");

// 7. OCR/文本检测示例
// 只有在启用了 feature="ocr" 并安装了系统 tesseract 时才有效
print("步骤 10: 进行 OCR 文本识别");
let text = read_text(0,0,0,0);
print("识别到的文本：" + text);
if (find_text("登录")) {
    print("✓ 屏幕中出现了'登录'，可以自动处理");
} else {
    print("✗ 未找到'登录'文本");
}
print("脚本执行完成！");
