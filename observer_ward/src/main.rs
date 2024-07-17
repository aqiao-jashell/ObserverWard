use console::{style, Emoji};
use log::{error, info};
use observer_ward::api::api_server;
#[cfg(not(target_os = "windows"))]
use observer_ward::api::background;
use observer_ward::cli::ObserverWardConfig;
use observer_ward::cluster_templates;
use observer_ward::helper::Helper;
use observer_ward::output::Output;
use observer_ward::scan;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
  let config = ObserverWardConfig::default();
  if config.debug {
    std::env::set_var("RUST_LOG", "observer_ward=debug,actix_web=debug");
  } else if config.silent {
    std::env::set_var("RUST_LOG", "observer_ward=off,actix_web=off");
  } else {
    std::env::set_var("RUST_LOG", "observer_ward=info,actix_web=info");
  }
  if config.no_color {
    console::set_colors_enabled(false);
    std::env::set_var("RUST_LOG_STYLE", "never");
  }
  // 自定义日志输出
  env_logger::builder()
    .format_target(false)
    // .format_level(false)
    .format_timestamp(None)
    .init();
  if let Some(address) = config.api_server {
    #[cfg(not(target_os = "windows"))]
    if config.daemon {
      background();
    }
    api_server(address, config.clone())
      .map_err(|err| error!("start api server err:{}", err))
      .unwrap_or_default();
    std::process::exit(0);
  }
  let helper = Helper::new(&config);
  helper.run();
  let templates = config.templates();
  info!(
    "{}probes loaded: {}",
    Emoji("📇", ""),
    style(templates.len()).blue()
  );
  let cl = cluster_templates(&templates);
  info!(
    "{}optimized probes: {}",
    Emoji("🚀", ""),
    style(cl.len()).blue()
  );
  let (tx, rx) = channel();
  let output_config = config.clone();
  thread::spawn(move || {
    scan(&config, cl, tx);
  });
  let mut output = Output::new(&output_config);
  for result in rx {
    output.save_and_print(result.clone());
    output.webhook_results(vec![result]);
  }
}
