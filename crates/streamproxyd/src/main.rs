use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use ose_config::{Config, ProxyMode};
use ose_plugin::PluginManager;
use ose_proxy::{build_state, run, PluginReloader};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "streamproxyd", about = "OpenStream Engine HLS/DASH proxy daemon")]
struct Args {
    #[arg(short, long, default_value = "config.example.yaml")]
    config: PathBuf,
    #[arg(long)]
    mitm: Option<bool>,
}

fn build_plugins(config: &Config) -> Result<PluginManager> {
    let mut plugins: Vec<Arc<dyn ose_plugin::Plugin>> = Vec::new();

    #[cfg(feature = "plugin-twitch")]
    if config.twitch.enabled {
        use ose_detector::Rule;
        use ose_plugin_twitch::TwitchPlugin;

        let mut rules = Vec::new();
        if config.twitch.detect_stitched {
            rules.push(Rule::contains("stitched"));
        }
        if config.twitch.detect_daterange {
            rules.push(Rule::date_range_stitched());
        }
        rules.push(Rule::extinf_not_live());
        if config.twitch.detect_regex {
            rules.push(Rule::regex(
                r"(?i)stitched|twitch-stitched-ad|X-TV-TWITCH-AD",
            ));
        }
        let plugin = TwitchPlugin::new(rules)
            .with_debug(config.twitch.debug)
            .with_max_wait(config.twitch.max_wait_secs)
            .with_backup_seamless(config.twitch.backup_seamless);
        plugins.push(Arc::new(plugin));
    }

    #[cfg(feature = "plugin-hls")]
    {
        use ose_plugin_hls::RulesHlsPlugin;
        use ose_rules::RulesFile;

        if config.kick.enabled {
            plugins.push(Arc::new(
                RulesHlsPlugin::kick().with_debug(config.kick.debug),
            ));
        }
        if config.trovo.enabled {
            plugins.push(Arc::new(
                RulesHlsPlugin::trovo().with_debug(config.trovo.debug),
            ));
        }
        if config.youtube.enabled {
            plugins.push(Arc::new(
                RulesHlsPlugin::youtube().with_debug(config.youtube.debug),
            ));
        }

        if let Some(ref path) = config.rules_file {
            match RulesFile::load(path) {
                Ok(file) => {
                    for rs in file.rulesets {
                        if rs.enabled {
                            let name = rs.name.clone();
                            plugins.push(Arc::new(
                                RulesHlsPlugin::from_ruleset(rs).with_debug(config.twitch.debug),
                            ));
                            info!(ruleset = %name, "loaded ruleset from rules_file");
                        }
                    }
                }
                Err(e) => warn!(path = %path, error = %e, "failed to load rules_file"),
            }
        }
    }

    #[cfg(feature = "plugin-dash")]
    if config.dash.enabled {
        use ose_plugin_dash::DashPlugin;
        plugins.push(Arc::new(
            DashPlugin::universal().with_debug(config.dash.debug),
        ));
    }

    #[cfg(not(feature = "plugin-twitch"))]
    if config.twitch.enabled {
        warn!("twitch.enabled but binary built without feature plugin-twitch");
    }
    #[cfg(not(feature = "plugin-hls"))]
    {
        if config.kick.enabled || config.trovo.enabled || config.youtube.enabled {
            warn!("hls service enabled but binary built without feature plugin-hls");
        }
        if config.rules_file.is_some() {
            warn!("rules_file set but binary built without feature plugin-hls");
        }
    }
    #[cfg(not(feature = "plugin-dash"))]
    if config.dash.enabled {
        warn!("dash.enabled but binary built without feature plugin-dash");
    }

    Ok(PluginManager::new(plugins))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config_path = args.config.clone();
    let config = if config_path.exists() {
        Config::load(&config_path).context("load config")?
    } else {
        info!("config not found, using defaults");
        Config::default()
    };

    let filter = if config.twitch.debug
        || config.kick.debug
        || config.trovo.debug
        || config.youtube.debug
        || config.dash.debug
    {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into())
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into())
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!(
        features = ?compiled_features(),
        "build features"
    );

    let manager = build_plugins(&config)?;
    info!(
        plugins = ?manager.plugin_names(),
        mode = ?config.mode,
        max_manifest_bytes = config.max_manifest_bytes,
        "starting"
    );

    if matches!(config.mode, ProxyMode::Off) {
        info!("mode=off — proxy disabled, API only");
    }

    let enable_mitm = args.mitm.unwrap_or(config.mitm);
    let path_for_reload = if config_path.exists() {
        Some(config_path.clone())
    } else {
        None
    };

    let reloader: Option<PluginReloader> = path_for_reload.as_ref().map(|p| {
        let p = p.clone();
        Arc::new(move || {
            let cfg = Config::load(&p)?;
            build_plugins(&cfg)
        }) as PluginReloader
    });

    let state = Arc::new(build_state(
        config,
        manager,
        enable_mitm,
        path_for_reload,
        reloader,
    )?);

    #[cfg(unix)]
    {
        let state_sighup = state.clone();
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};
            let mut stream = match signal(SignalKind::hangup()) {
                Ok(s) => s,
                Err(e) => {
                    warn!(error = %e, "SIGHUP handler unavailable");
                    return;
                }
            };
            while stream.recv().await.is_some() {
                info!("SIGHUP received — reloading");
                if let Some(path) = &state_sighup.config_path {
                    if let Ok(cfg) = Config::load(path) {
                        state_sighup.replace_config(cfg);
                    }
                }
                if let Some(reload) = &state_sighup.reload_plugins {
                    match reload() {
                        Ok(pm) => {
                            state_sighup.replace_plugins(pm);
                            info!("plugins rebuilt via SIGHUP");
                        }
                        Err(e) => warn!(error = %e, "SIGHUP plugin reload failed"),
                    }
                }
            }
        });
    }

    run(state).await
}

fn compiled_features() -> Vec<&'static str> {
    #[allow(clippy::vec_init_then_push)]
    {
        let mut f = Vec::new();
        #[cfg(feature = "plugin-twitch")]
        f.push("plugin-twitch");
        #[cfg(feature = "plugin-hls")]
        f.push("plugin-hls");
        #[cfg(feature = "plugin-dash")]
        f.push("plugin-dash");
        f
    }
}
