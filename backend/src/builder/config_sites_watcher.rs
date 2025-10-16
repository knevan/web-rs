use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use notify::Error;
use notify_debouncer_full::notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebouncedEvent, new_debouncer};

use crate::scraping::model::SitesConfig;

pub async fn config_sites_watcher(
    config_path: String,
    config_swap: Arc<ArcSwap<SitesConfig>>,
) {
    println!("[CONFIG-WATCHER] Watch file {}", config_path);

    let path = Path::new(&config_path);
    let clone_config_path = config_path.clone();

    let mut debouncer = match new_debouncer(
        Duration::from_secs(5),
        None,
        move |res: Result<Vec<DebouncedEvent>, Vec<Error>>| match res {
            Ok(events) => {
                if events
                    .iter()
                    .any(|event| matches!(event.kind, EventKind::Modify(_)))
                {
                    println!(
                        "[CONFIG-WATCHER] Detected changes in config file, attempting to reload."
                    );

                    match SitesConfig::load(&clone_config_path) {
                        Ok(new_config) => {
                            config_swap.store(Arc::new(new_config));
                            println!(
                                "[CONFIG-WATCHER] Config store and reloaded successfully"
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[CONFIG-WATCHER] Config reload failed: {:?}. Keep the old version",
                                e
                            );
                        }
                    }
                }
            }
            Err(errors) => errors.iter().for_each(|error| {
                eprintln!("[CONFIG-WATCHER] File watcher error: {}", error);
            }),
        },
    ) {
        Ok(debouncer) => debouncer,
        Err(e) => {
            eprintln!(
                "[CONFIG-WATCHER] Could not start file watcher: {}. Hot-reloading will be disabled",
                e
            );
            return;
        }
    };

    if let Err(e) = debouncer.watch(path, RecursiveMode::NonRecursive) {
        eprintln!(
            "[CONFIG-WATCHER] Could not watch config file at '{}': {}. Hot-reloading will be disabled",
            path.display(),
            e
        );
    }

    std::future::pending::<()>().await;
}
