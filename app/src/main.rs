async fn delay(duration: std::time::Duration) {
    let start = instant::Instant::now();
    loop {
        let now = instant::Instant::now();
        let duration_from_start = now.duration_since(start);
        if duration_from_start > duration {
            break;
        }
        runtime::next_frame().await;
    }
}

async fn loop_log(n: usize) {
    loop {
        for _ in 0..n {
            runtime::next_frame().await;
        }
        log::info!("loop_log: {n}");
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Start app!");

    runtime::spawn({
        async move {
            log::info!("Hello, world!");
            delay(std::time::Duration::from_secs(1)).await;
            log::info!("Hello, Wasm!");
            futures::join!(loop_log(60), loop_log(240));
        }
    });
}
