// demo.rhai - simple automation example
// 找到屏幕上的 start 按钮并点击
let result = find_and_click("start_btn.png");
if result {
    sleep(500);
    print("成功找到并点击了 start_btn.png");
} else {
    print("未找到 start_btn.png");
}
