use yahoo_finance_api as yahoo;
use std::error::Error;

#[tokio::main] // 异步程序的入口
async fn main() -> Result<(), Box<dyn Error>> {
    // `new()` 返回 Result，因此使用 `?` 传播错误（主函数已返回 Result）
    let provider = yahoo::YahooConnector::new()?;
    // 允许通过命令行提供第一个 ticker，随后进入交互循环
    let mut initial_ticker: Option<String> = None;
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        initial_ticker = Some(args[1].clone());
    }

    // 用于存放当前查询的股票代码
    let mut ticker = String::new();

    loop {
        if let Some(t) = initial_ticker.take() {
            ticker = t;
        } else {
            print!("请输入股票代码（可用逗号或空格分隔多个），或输入 exit 退出: ");
            use std::io::{self, Write};
            io::stdout().flush()?;
            ticker.clear();
            io::stdin().read_line(&mut ticker)?;
            ticker = ticker.trim().to_string();
        }

        // 检查是否退出
        if ticker.eq_ignore_ascii_case("exit") {
            println!("退出程序");
            break;
        }

        // 将输入拆分成多个代码
        let symbols: Vec<&str> = ticker
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect();

        for sym in symbols {
            println!("正在查询 {} 的股价...", sym);

            let response_result = provider.get_latest_quotes(sym, "1d").await;
            let response = match response_result {
                Ok(r) => r,
                Err(e) => {
                    println!("查询 {} 时出错：{}，请检查股票代码或稍后再试。", sym, e);
                    println!("");
                    continue;
                }
            };

            if let Ok(quote) = response.last_quote() {
                println!("--------------------------------");
                println!("股票代码: {}", sym);
                println!("最新收盘价: ${:.2}", quote.close);
                println!("成交量: {}", quote.volume);
                println!("--------------------------------");
            } else {
                println!("未找到 {} 对应的股票数据，请检查代码是否正确。", sym);
            }

            println!("");
        }
        // 下一次循环会等待输入
    }

    Ok(())
}
