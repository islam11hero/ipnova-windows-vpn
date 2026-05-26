//! sing-box config shaping for system-proxy mode.

use serde_json::Value;

use super::constants::SYSTEM_PROXY_PORT;
use crate::wininet_registry::proxy_server_value;

pub fn prepare_system_proxy_config(config: &mut Value) {
    let obj = match config.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    let _proxy_server = proxy_server_value(SYSTEM_PROXY_PORT);

    obj.insert(
        "inbounds".to_string(),
        serde_json::json!([{
            "type": "mixed",
            "tag": "mixed-in",
            "listen": "127.0.0.1",
            "listen_port": SYSTEM_PROXY_PORT,
            "sniff": true,
            "set_system_proxy": false
        }]),
    );

    let route = obj
        .entry("route".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Some(route_obj) = route.as_object_mut() {
        route_obj
            .entry("auto_detect_interface".to_string())
            .or_insert(serde_json::json!(true));

        let sniff_rule = serde_json::json!({ "action": "sniff" });
        match route_obj.get_mut("rules") {
            Some(Value::Array(rules)) => {
                let has_sniff = rules.iter().any(|r| {
                    r.get("action")
                        .and_then(|a| a.as_str())
                        .is_some_and(|a| a == "sniff")
                });
                if !has_sniff {
                    rules.insert(0, sniff_rule);
                }
            }
            _ => {
                route_obj.insert("rules".to_string(), serde_json::json!([sniff_rule]));
            }
        }

        let private_rule = serde_json::json!({
            "ip_is_private": true,
            "outbound": "direct"
        });
        if let Some(Value::Array(rules)) = route_obj.get_mut("rules") {
            let has_private = rules
                .iter()
                .any(|r| r.get("ip_is_private").and_then(|v| v.as_bool()) == Some(true));
            if !has_private {
                rules.push(private_rule);
            }
        }
    }

    harden_singbox_config(config);
}

/// DNS via VPN outbound + QUIC reject (helps TUN; encourages TCP in proxied clients).
pub fn harden_singbox_config(config: &mut Value) {
    let obj = match config.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    if !obj.contains_key("dns") {
        obj.insert(
            "dns".to_string(),
            serde_json::json!({
                "servers": [
                    {
                        "tag": "dns-remote",
                        "address": "8.8.8.8",
                        "detour": "proxy"
                    },
                    {
                        "tag": "dns-local",
                        "address": "local",
                        "detour": "direct"
                    }
                ],
                "rules": [
                    { "outbound": "any", "server": "dns-remote" }
                ],
                "final": "dns-remote"
            }),
        );
    }

    let route = obj
        .entry("route".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Some(route_obj) = route.as_object_mut() {
        let quic_reject = serde_json::json!({ "protocol": "quic", "action": "reject" });
        match route_obj.get_mut("rules") {
            Some(Value::Array(rules)) => {
                let has_quic = rules.iter().any(|r| {
                    r.get("protocol")
                        .and_then(|p| p.as_str())
                        .is_some_and(|p| p == "quic")
                });
                if !has_quic {
                    rules.push(quic_reject);
                }
            }
            _ => {
                route_obj.insert("rules".to_string(), serde_json::json!([quic_reject]));
            }
        }
    }
}
