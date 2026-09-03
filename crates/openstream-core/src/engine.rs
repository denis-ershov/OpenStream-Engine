use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;
use parking_lot::RwLock;

use openstream_rule::schema::{
    DetailedAction, NestedAction, RoutingAction, RuleManifest, SimpleAction, TaggedAction,
};
use crate::backend::{CompiledRuleSet, EgressGateway};
use crate::ip_tree::IpTable;
use crate::trie::DomainTrie;

/// Решение о маршрутизации пакета/соединения
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RoutingVerdict {
    #[default]
    Direct,
    Bypass, // Принудительный пропуск без обработки (Direct WAN)
    DpiEvasiveDirect,
    Zapret2 { preset: String, custom_args: Option<String> },
    StreamProxy { mode: String },
    Proxy { gateway_id: String },
    DnsOverride { ips: Vec<IpAddr> },
    Block { reason: Option<String> },
    StripPayload { plugin_id: String },
}

impl From<RoutingAction> for RoutingVerdict {
    fn from(action: RoutingAction) -> Self {
        match action {
            RoutingAction::Simple(s) => match s {
                SimpleAction::Direct => RoutingVerdict::Direct,
                SimpleAction::Bypass | SimpleAction::Pass => RoutingVerdict::Bypass,
                SimpleAction::DpiEvasiveDirect => RoutingVerdict::DpiEvasiveDirect,
                SimpleAction::Zapret2 => RoutingVerdict::Zapret2 {
                    preset: "general".to_string(),
                    custom_args: None,
                },
                SimpleAction::StreamProxy => RoutingVerdict::StreamProxy {
                    mode: "default".to_string(),
                },
                SimpleAction::Block => RoutingVerdict::Block { reason: None },
            },
            RoutingAction::Detailed(d) => match d {
                DetailedAction::Tagged(t) => match t {
                    TaggedAction::Direct => RoutingVerdict::Direct,
                    TaggedAction::Bypass | TaggedAction::Pass => RoutingVerdict::Bypass,
                    TaggedAction::DpiEvasiveDirect => RoutingVerdict::DpiEvasiveDirect,
                    TaggedAction::Zapret2 { preset, custom_args } => {
                        RoutingVerdict::Zapret2 { preset, custom_args }
                    }
                    TaggedAction::StreamProxy { mode } => RoutingVerdict::StreamProxy { mode },
                    TaggedAction::Block => RoutingVerdict::Block { reason: None },
                    TaggedAction::Proxy { gateway_id } => RoutingVerdict::Proxy { gateway_id },
                    TaggedAction::DnsOverride { ips } => RoutingVerdict::DnsOverride { ips },
                    TaggedAction::StripPayload { plugin_id } => {
                        RoutingVerdict::StripPayload { plugin_id }
                    }
                },
                DetailedAction::Nested(n) => match n {
                    NestedAction::Zapret2 { preset, custom_args } => {
                        RoutingVerdict::Zapret2 { preset, custom_args }
                    }
                    NestedAction::StreamProxy { mode } => RoutingVerdict::StreamProxy { mode },
                    NestedAction::Proxy { gateway_id } => RoutingVerdict::Proxy { gateway_id },
                    NestedAction::DnsOverride { ips } => RoutingVerdict::DnsOverride { ips },
                    NestedAction::StripPayload { plugin_id } => {
                        RoutingVerdict::StripPayload { plugin_id }
                    }
                },
            },
        }
    }
}

/// Внутреннее состояние скомпилированных правил в памяти
#[derive(Default)]
struct CompiledState {
    domain_trie: DomainTrie<RoutingVerdict>,
    ip_table: IpTable<RoutingVerdict>,
}

/// Центральный движок политик маршрутизации трафика (PolicyEngine)
pub struct PolicyEngine {
    manifests: RwLock<HashMap<String, RuleManifest>>,
    gateways: RwLock<HashMap<String, EgressGateway>>,
    state: RwLock<CompiledState>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            manifests: RwLock::new(HashMap::new()),
            gateways: RwLock::new(HashMap::new()),
            state: RwLock::new(CompiledState::default()),
        }
    }

    /// Добавить или обновить манифест правила
    pub fn add_rule(&self, manifest: RuleManifest) {
        let mut manifests = self.manifests.write();
        manifests.insert(manifest.id.clone(), manifest);
        drop(manifests);
        self.recompile();
    }

    /// Удалить манифест правила по идентификатору
    pub fn remove_rule(&self, id: &str) {
        let mut manifests = self.manifests.write();
        manifests.remove(id);
        drop(manifests);
        self.recompile();
    }

    /// Обновить перечень доступных шлюзов выхода (туннелей)
    pub fn update_gateways(&self, gateways: Vec<EgressGateway>) {
        let mut gw_lock = self.gateways.write();
        gw_lock.clear();
        for gw in gateways {
            gw_lock.insert(gw.id.clone(), gw);
        }
        drop(gw_lock);
        self.recompile();
    }

    /// Сопоставление домена / SNI хоста с политиками маршрутизации
    pub fn resolve_domain(&self, domain: &str, _app_id: Option<&str>) -> RoutingVerdict {
        let state = self.state.read();
        state
            .domain_trie
            .find(domain)
            .cloned()
            .unwrap_or(RoutingVerdict::Direct)
    }

    /// Сопоставление IP-адреса назначения с политиками маршрутизации
    pub fn resolve_ip(&self, ip: IpAddr, _port: u16) -> RoutingVerdict {
        let state = self.state.read();
        state
            .ip_table
            .find(ip)
            .cloned()
            .unwrap_or(RoutingVerdict::Direct)
    }

    /// Перекомпиляция всех зарегистрированных правил с учетом доступных шлюзов
    pub fn recompile(&self) {
        let manifests = self.manifests.read();
        let gateways = self.gateways.read();

        let mut trie = DomainTrie::new();
        let mut iptable = IpTable::new();

        for manifest in manifests.values() {
            if !manifest.enabled {
                continue;
            }

            for rule in &manifest.matches {
                let verdict = self.resolve_rule_action(rule, &manifest.strategies, &gateways);

                for domain in &rule.domains {
                    trie.insert(domain, verdict.clone());
                }

                for cidr in &rule.cidrs {
                    iptable.insert(*cidr, verdict.clone());
                }
            }
        }

        let mut state = self.state.write();
        state.domain_trie = trie;
        state.ip_table = iptable;
    }

    fn resolve_rule_action(
        &self,
        rule: &openstream_rule::schema::MatchRule,
        strategies: &BTreeMap<String, openstream_rule::schema::StrategyDef>,
        gateways: &HashMap<String, EgressGateway>,
    ) -> RoutingVerdict {
        if let Some(ref action) = rule.action {
            return action.clone().into();
        }

        if let Some(ref strat_name) = rule.strategy {
            if let Some(strategy) = strategies.get(strat_name) {
                // Перебор цепочки предпочтений
                for pref in &strategy.preference {
                    match pref.as_str() {
                        "direct" => return RoutingVerdict::Direct,
                        "dpi_evasion" | "dpi_evasive_direct" => {
                            return RoutingVerdict::DpiEvasiveDirect;
                        }
                        tag => {
                            // Поиск шлюза с соответствующим тегом или ID
                            for gw in gateways.values() {
                                if gw.healthy && (gw.id == tag || gw.tag == tag) {
                                    return RoutingVerdict::Proxy {
                                        gateway_id: gw.id.clone(),
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        RoutingVerdict::Direct
    }

    /// Экспорт скомпилированных наборов для аппаратного бэкенда (nftables / NetworkExtension)
    pub fn compile_for_backend(&self) -> CompiledRuleSet {
        let manifests = self.manifests.read();
        let gateways = self.gateways.read();
        let mut compiled = CompiledRuleSet::default();

        for manifest in manifests.values() {
            if !manifest.enabled {
                continue;
            }

            for rule in &manifest.matches {
                let verdict = self.resolve_rule_action(rule, &manifest.strategies, &gateways);

                if let Some(ref bypass) = rule.bypass {
                    compiled.bypass_cidrs.extend(bypass.bypass_cidrs.clone());
                    if bypass.bypass_p2p {
                        compiled.bypass_p2p = true;
                    }
                    if bypass.disable_quic {
                        compiled.disable_quic = true;
                    }
                    if bypass.block_doh {
                        compiled.block_doh = true;
                    }
                    if bypass.exclude_ntp {
                        compiled.exclude_ntp = true;
                    }
                }

                for domain in &rule.domains {
                    match &verdict {
                        RoutingVerdict::Direct => compiled.direct_domains.push(domain.clone()),
                        RoutingVerdict::Bypass => compiled.bypass_domains.push(domain.clone()),
                        RoutingVerdict::DpiEvasiveDirect => {
                            compiled.dpi_evasive_domains.push(domain.clone());
                        }
                        RoutingVerdict::Zapret2 { preset, .. } => {
                            compiled
                                .zapret2_domains
                                .push((domain.clone(), preset.clone()));
                        }
                        RoutingVerdict::StreamProxy { .. } => {
                            compiled.streamproxy_domains.push(domain.clone());
                        }
                        RoutingVerdict::Proxy { gateway_id } => {
                            compiled
                                .proxy_domains
                                .push((domain.clone(), gateway_id.clone()));
                        }
                        RoutingVerdict::Block { .. } => {
                            compiled.blocked_domains.push(domain.clone());
                        }
                        RoutingVerdict::DnsOverride { ips } => {
                            compiled.dns_overrides.push((domain.clone(), ips.clone()));
                        }
                        RoutingVerdict::StripPayload { .. } => {}
                    }
                }
            }
        }

        compiled
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openstream_rule::schema::{MatchRule, StrategyDef};

    #[test]
    fn test_policy_engine_routing() {
        let engine = PolicyEngine::new();

        // Добавляем доступный шлюз (шлюз в Албании)
        engine.update_gateways(vec![EgressGateway {
            id: "gw_al".into(),
            tag: "geo:al".into(),
            protocol: "wireguard".into(),
            endpoint: "1.2.3.4:51820".into(),
            healthy: true,
        }]);

        let mut strategies = BTreeMap::new();
        strategies.insert(
            "adfree".into(),
            StrategyDef {
                preference: vec!["geo:al".into(), "direct".into()],
            },
        );

        let manifest = RuleManifest {
            schema_version: "2.0".into(),
            id: "test.twitch".into(),
            name: "Twitch".into(),
            version: "1.0.0".into(),
            author: None,
            description: None,
            enabled: true,
            capabilities_required: vec![],
            allow_sensitive: false,
            matches: vec![
                MatchRule {
                    group: Some("token".into()),
                    clients: None,
                    bypass: None,
                    domains: vec!["gql.twitch.tv".into()],
                    cidrs: vec![],
                    ports: vec![],
                    app_ids: vec![],
                    strategy: Some("adfree".into()),
                    action: None,
                },
                MatchRule {
                    group: Some("video".into()),
                    clients: None,
                    bypass: None,
                    domains: vec!["*.live-video.net".into()],
                    cidrs: vec![],
                    ports: vec![],
                    app_ids: vec![],
                    strategy: None,
                    action: Some(RoutingAction::Simple(SimpleAction::Direct)),
                },
                MatchRule {
                    group: Some("ads".into()),
                    clients: None,
                    bypass: None,
                    domains: vec!["edge.ads.twitch.tv".into()],
                    cidrs: vec![],
                    ports: vec![],
                    app_ids: vec![],
                    strategy: None,
                    action: Some(RoutingAction::Simple(SimpleAction::Block)),
                },
            ],
            strategies,
        };

        engine.add_rule(manifest);

        // Проверка резолва
        assert_eq!(
            engine.resolve_domain("gql.twitch.tv", None),
            RoutingVerdict::Proxy {
                gateway_id: "gw_al".into()
            }
        );

        assert_eq!(
            engine.resolve_domain("video-edge-1.live-video.net", None),
            RoutingVerdict::Direct
        );

        assert_eq!(
            engine.resolve_domain("edge.ads.twitch.tv", None),
            RoutingVerdict::Block { reason: None }
        );

        assert_eq!(
            engine.resolve_domain("unrelated.org", None),
            RoutingVerdict::Direct
        );
    }
}
