use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong, jlongArray, jstring};
use jni::JNIEnv;

use openstream_core::engine::RoutingVerdict;
use openstream_core::PolicyEngine;
use openstream_rule::parse_rule_yaml;

struct EngineState {
    engine: PolicyEngine,
    total_queries: AtomicU64,
    blocked_queries: AtomicU64,
    proxied_queries: AtomicU64,
    dpi_evasive_queries: AtomicU64,
    direct_queries: AtomicU64,
}

static STATE: OnceLock<EngineState> = OnceLock::new();

fn get_state() -> &'static EngineState {
    STATE.get_or_init(|| EngineState {
        engine: PolicyEngine::new(),
        total_queries: AtomicU64::new(0),
        blocked_queries: AtomicU64::new(0),
        proxied_queries: AtomicU64::new(0),
        dpi_evasive_queries: AtomicU64::new(0),
        direct_queries: AtomicU64::new(0),
    })
}

#[no_mangle]
pub extern "system" fn Java_org_openstream_engine_OpenStreamCore_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    rules_dir: JString,
) {
    let state = get_state();
    if let Ok(dir_str) = env.get_string(&rules_dir) {
        let dir_string = dir_str.to_string_lossy().to_string();
        let path = Path::new(&dir_string);
        if path.exists() && path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let sub_path = entry.path();
                    if sub_path.is_file() && sub_path.to_string_lossy().ends_with(".osrule.yaml") {
                        if let Ok(content) = fs::read_to_string(&sub_path) {
                            if let Ok(rule) = parse_rule_yaml(&content) {
                                state.engine.add_rule(rule);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_openstream_engine_OpenStreamCore_nativeMatchDomain(
    mut env: JNIEnv,
    _class: JClass,
    domain: JString,
) -> jint {
    let state = get_state();
    state.total_queries.fetch_add(1, Ordering::Relaxed);

    let domain_str = match env.get_string(&domain) {
        Ok(s) => s.to_string_lossy().to_string(),
        Err(_) => return 0, // Direct
    };

    let verdict = state.engine.resolve_domain(&domain_str, None);

    match verdict {
        RoutingVerdict::Direct => {
            state.direct_queries.fetch_add(1, Ordering::Relaxed);
            0
        }
        RoutingVerdict::DpiEvasiveDirect => {
            state.dpi_evasive_queries.fetch_add(1, Ordering::Relaxed);
            1
        }
        RoutingVerdict::Proxy { .. } => {
            state.proxied_queries.fetch_add(1, Ordering::Relaxed);
            2
        }
        RoutingVerdict::DnsOverride { .. } => {
            3
        }
        RoutingVerdict::Block { .. } => {
            state.blocked_queries.fetch_add(1, Ordering::Relaxed);
            4
        }
        RoutingVerdict::Zapret2 { .. } => {
            state.dpi_evasive_queries.fetch_add(1, Ordering::Relaxed);
            1
        }
        RoutingVerdict::StreamProxy { .. } => {
            5
        }
        RoutingVerdict::StripPayload { .. } => 0,
    }
}

#[no_mangle]
pub extern "system" fn Java_org_openstream_engine_OpenStreamCore_nativeLoadRule(
    mut env: JNIEnv,
    _class: JClass,
    yaml: JString,
) -> jstring {
    let state = get_state();

    let yaml_str = match env.get_string(&yaml) {
        Ok(s) => s.to_string_lossy().to_string(),
        Err(_) => return env.new_string("").unwrap().into_raw(),
    };

    match parse_rule_yaml(&yaml_str) {
        Ok(manifest) => {
            let id = manifest.id.clone();
            state.engine.add_rule(manifest);
            env.new_string(id).unwrap().into_raw()
        }
        Err(e) => {
            let err_msg = format!("ERR: {}", e);
            env.new_string(err_msg).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_openstream_engine_OpenStreamCore_nativeGetMetrics(
    env: JNIEnv,
    _class: JClass,
) -> jlongArray {
    let state = get_state();

    let metrics: [jlong; 6] = [
        state.total_queries.load(Ordering::Relaxed) as jlong,
        state.blocked_queries.load(Ordering::Relaxed) as jlong,
        state.proxied_queries.load(Ordering::Relaxed) as jlong,
        state.dpi_evasive_queries.load(Ordering::Relaxed) as jlong,
        state.direct_queries.load(Ordering::Relaxed) as jlong,
        (1024 * 1024 * 2) as jlong, // ~2 MB memory
    ];

    let long_array = match env.new_long_array(6) {
        Ok(a) => a,
        Err(_) => return std::ptr::null_mut(),
    };

    if env.set_long_array_region(&long_array, 0, &metrics).is_ok() {
        long_array.into_raw()
    } else {
        std::ptr::null_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_state_basic() {
        let state = get_state();
        let yaml = r#"
schema_version: "2.0"
id: "org.openstream.android.test"
name: "Android Test Rule"
version: "1.0.0"

matches:
  - domains: ["*.ads.android.com"]
    action: "block"
  - domains: ["video.android.tv"]
    action: "direct"
"#;
        let rule = parse_rule_yaml(yaml).expect("Rule should parse");
        state.engine.add_rule(rule);

        let v1 = state.engine.resolve_domain("edge.ads.android.com", None);
        assert!(matches!(v1, RoutingVerdict::Block { .. }));

        let v2 = state.engine.resolve_domain("video.android.tv", None);
        assert_eq!(v2, RoutingVerdict::Direct);
    }
}
