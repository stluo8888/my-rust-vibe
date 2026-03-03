use rhai::Engine;
use enigo::{Enigo, MouseControllable, KeyboardControllable, Key};
use std::{io, thread, time};

/// 初始化并返回一个只包含安全 API 的 Rhai 引擎。
///
/// 这样即使别人拿到了 rpa-core 的代码，没有你定义的 Action 库，
/// 也无法操作网页或 App，只能调用你显式注册的函数。
pub fn init_engine() -> Engine {
    // 使用 new_raw 创建完全真空的引擎
    let mut engine = Engine::new_raw();

    // headless 模式（例如 CI 环境），避免调用 enigo/scrap
    let headless = std::env::var("RPA_HEADLESS").is_ok();

    // 注入你自己的“工具箱”
    if headless {
        engine.register_fn("move_mouse", |_: i64, _: i64| {});
        engine.register_fn("left_click", || {});
        engine.register_fn("right_click", || {});
        engine.register_fn("type_text", |_: &str| {});
        engine.register_fn("press_key", |_: &str| {});
    } else {
        engine.register_fn("move_mouse", |x: i64, y: i64| {
            let mut enigo = Enigo::new();
            enigo.mouse_move_to(x as i32, y as i32);
        });

        engine.register_fn("left_click", || {
            let mut enigo = Enigo::new();
            enigo.mouse_click(enigo::MouseButton::Left);
        });

        engine.register_fn("right_click", || {
            let mut enigo = Enigo::new();
            enigo.mouse_click(enigo::MouseButton::Right);
        });

        engine.register_fn("type_text", |s: &str| {
            let mut enigo = Enigo::new();
            enigo.key_sequence(s);
        });

        engine.register_fn("press_key", |k: &str| {
            let key = match k.to_lowercase().as_str() {
                "enter" => Key::Return,
                "space" => Key::Space,
                "tab" => Key::Tab,
                other => Key::Layout(other.chars().next().unwrap_or('\0')),
            };
            let mut enigo = Enigo::new();
            enigo.key_click(key);
        });
    }

    // 其它函数与头less无关
    engine.register_fn("sleep", |ms: i64| {
        thread::sleep(time::Duration::from_millis(ms as u64));
    });

    engine.register_fn("print", |msg: &str| {
        println!("{}", msg);
    });

    engine.register_fn("screenshot", |path: &str| {
        capture_screen().save(path).is_ok()
    });

    // 图像匹配相关功能
    engine.register_fn("find_icon", |template_path: &str| {
        if let Some((x, y)) = find_icon_impl(template_path) {
            (x, y)
        } else {
            (-1, -1)
        }
    });

    // 便捷函数来获取元组的第一个和第二个元素
    // 注：register_fn 会自动展开元组参数，所以我们需要用另一种方式
    // 改为注册一个 Position 类型会更干净，但为了简便直接修改脚本使用 let

    // 组合示例：查找图标然后单击
    engine.register_fn("find_and_click", |template_path: &str| {
        if let Some((x, y)) = find_icon_impl(template_path) {
            let mut enigo = Enigo::new();
            enigo.mouse_move_to(x as i32, y as i32);
            enigo.mouse_click(enigo::MouseButton::Left);
            true
        } else {
            false
        }
    });

    // OCR functions are optional and require the "ocr" feature
    #[cfg(feature = "ocr")]
    {
        if headless {
            engine.register_fn("read_text", |_: i64, _: i64, _: i64, _: i64| -> String {
                String::new()
            });
            engine.register_fn("find_text", |_: &str| -> bool { false });
        } else {
            engine.register_fn("read_text", |x: i64, y: i64, w: i64, h: i64| {
                read_text_impl(x, y, w, h)
            });
            engine.register_fn("find_text", |query: &str| read_text_impl(0, 0, 0, 0).contains(query));
        }
    }

    engine
}

/// 获取当前屏幕像素并返回为 image::RgbaImage
fn capture_screen() -> image::RgbaImage {
    // 使用 scrap 捕获主显示器的截图。如果没有显示器（headless CI），返回 1x1 空图。
    if let Ok(display) = scrap::Display::primary() {
        let (w, h) = (display.width(), display.height());
        let mut capturer = scrap::Capturer::new(display).expect("failed to create capturer");
        let one_frame = loop {
            match capturer.frame() {
                Ok(frame) => break frame.to_vec(),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => panic!("scrap error: {}", e),
            }
        };
        image::ImageBuffer::from_raw(w as u32, h as u32, one_frame).expect("buffer raw")
    } else {
        // 无显示时返回 1x1 黑图
        image::ImageBuffer::new(1, 1)
    }
}

/// 在大图中查找模板，返回坐标
fn find_template(screen: &image::RgbaImage, template: &image::RgbaImage) -> Option<(i64, i64)> {
    // 简单的暴力像素比较
    let (sw, sh) = screen.dimensions();
    let (tw, th) = template.dimensions();
    for y in 0..=(sh - th) {
        for x in 0..=(sw - tw) {
            let mut match_found = true;
            for ty in 0..th {
                for tx in 0..tw {
                    if screen.get_pixel(x+tx, y+ty) != template.get_pixel(tx, ty) {
                        match_found = false;
                        break;
                    }
                }
                if !match_found { break; }
            }
            if match_found {
                return Some((x as i64, y as i64));
            }
        }
    }
    None
}

/// 内部实现，用于注册函数
fn find_icon_impl(template_path: &str) -> Option<(i64, i64)> {
    let screen = capture_screen();
    if let Ok(template) = image::open(template_path) {
        let template = template.to_rgba8();
        find_template(&screen, &template)
    } else {
        None
    }
}

// region crop helper used by OCR
#[cfg(feature = "ocr")]
fn crop_region(screen: &image::RgbaImage, x: i64, y: i64, w: i64, h: i64) -> image::RgbaImage {
    let (sw, sh) = screen.dimensions();
    let x = x.max(0) as u32;
    let y = y.max(0) as u32;
    let w = if w <= 0 { sw } else { w as u32 };
    let h = if h <= 0 { sh } else { h as u32 };
    image::imageops::crop_imm(screen, x, y, w.min(sw - x), h.min(sh - y)).to_image()
}

#[cfg(feature = "ocr")]
fn read_text_impl(x: i64, y: i64, w: i64, h: i64) -> String {
    let screen = capture_screen();
    let region = crop_region(&screen, x, y, w, h);

    // convert to grayscale bytes
    // convert to grayscale and obtain raw bytes
    let gray = image::imageops::grayscale(&region);
    let bytes = gray.into_raw();

    let mut lt = leptess::LepTess::new(None, "eng").unwrap();
    lt.set_image_from_mem(&bytes).unwrap();
    lt.get_utf8_text().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_has_basic_ops() {
        unsafe { std::env::set_var("RPA_HEADLESS", "1"); }
        let engine = init_engine();
        // move mouse should not error (noop in headless)
        assert!(engine.eval::<()>("move_mouse(0,0);").is_ok());
        assert!(engine.eval::<()>("sleep(1);").is_ok());
        assert!(engine.eval::<()>("press_key(\"a\");").is_ok());
    }

    #[test]
    fn find_icon_returns_tuple() {
        unsafe { std::env::set_var("RPA_HEADLESS", "1"); }
        let engine = init_engine();
        // nonexistent path returns -1,-1; capture_screen now safe in headless
        let result: rhai::Dynamic = engine.eval("find_icon(\"foo\");").unwrap();
        let tup = result.cast::<(i64,i64)>();
        assert_eq!(tup, (-1, -1));
    }

    #[test]
    fn find_and_click_is_boolean() {
        unsafe { std::env::set_var("RPA_HEADLESS", "1"); }
        let engine = init_engine();
        let result: bool = engine.eval("find_and_click(\"foo\");").unwrap();
        assert!(!result);
    }

    #[cfg(feature = "ocr")]
    #[test]
    fn ocr_read_text_headless_returns_empty() {
        unsafe { std::env::set_var("RPA_HEADLESS", "1"); }
        let engine = init_engine();
        let text: String = engine.eval("read_text(0,0,0,0);").unwrap();
        assert_eq!(text, "");
    }

    #[cfg(feature = "ocr")]
    #[test]
    fn ocr_find_text_headless_returns_false() {
        unsafe { std::env::set_var("RPA_HEADLESS", "1"); }
        let engine = init_engine();
        let result: bool = engine.eval("find_text(\"anything\");").unwrap();
        assert!(!result);
    }
}
