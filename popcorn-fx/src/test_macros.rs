use std::sync::Once;

pub static INIT: Once = Once::new();

/// Initializes the logger with the specified log level.
macro_rules! init_logger {
    () => {{
        init_logger!(log::LevelFilter::Trace)
    }};
    ($level:expr) => {{
        use log4rs::Config;
        use log4rs::append::console::ConsoleAppender;
        use log4rs::config::Appender;
        use log4rs::config::Root;
        use log4rs::config::runtime::Logger;
        use log4rs::encode::pattern::PatternEncoder;
        use log::LevelFilter;

        let level: LevelFilter = $level;

        crate::test_macros::INIT.call_once(|| {
            log4rs::init_config(Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder()
                    .encoder(Box::new(PatternEncoder::new("\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<60.60}\x1B[0m \x1B[37m:\x1B[0m {m}{n}")))
                    .build())))
                .logger(Logger::builder().build("async_io", LevelFilter::Info))
                .logger(Logger::builder().build("fx_callback", LevelFilter::Info))
                .logger(Logger::builder().build("fx_torrent", LevelFilter::Info))
                .logger(Logger::builder().build("h2", LevelFilter::Info))
                .logger(Logger::builder().build("httpmock::server", LevelFilter::Debug))
                .logger(Logger::builder().build("hyper", LevelFilter::Info))
                .logger(Logger::builder().build("hyper_util", LevelFilter::Info))
                .logger(Logger::builder().build("mdns_sd", LevelFilter::Info))
                .logger(Logger::builder().build("mio", LevelFilter::Info))
                .logger(Logger::builder().build("neli", LevelFilter::Info))
                .logger(Logger::builder().build("polling", LevelFilter::Info))
                .logger(Logger::builder().build("popcorn_fx_players", LevelFilter::Debug))
                .logger(Logger::builder().build("reqwest", LevelFilter::Info))
                .logger(Logger::builder().build("rustls", LevelFilter::Info))
                .logger(Logger::builder().build("serde_xml_rs", LevelFilter::Info))
                .logger(Logger::builder().build("tracing", LevelFilter::Info))
                .logger(Logger::builder().build("want", LevelFilter::Info))
                .build(Root::builder().appender("stdout").build(level))
                .unwrap())
                .unwrap();
        })
    }};
}

/// A macro wrapper for [`tokio::time::timeout`] that awaits a future with a timeout duration.
macro_rules! timeout {
    ($future:expr, $duration:expr) => {{
        timeout!($future, $duration, "operation timed-out")
    }};
    ($future:expr, $duration:expr, $message:expr) => {{
        use std::io;
        use std::time::Duration;
        use tokio::time::timeout;

        let future = $future;
        let duration: Duration = $duration;

        timeout(duration, future)
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("after {}.{:03}s", duration.as_secs(), duration.as_millis()),
                )
            })
            .expect($message)
    }};
}
