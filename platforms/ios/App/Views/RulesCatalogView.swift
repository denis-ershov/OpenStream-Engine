//
//  RulesCatalogView.swift
//  OpenStream
//
//  Управление каталогом декларативных правил (.osrule.yaml).
//  Согласовано с мобильными стандартами UI/UX (без таблиц, адаптивные карточки, переключатели).
//

import SwiftUI

public struct RuleItemModel: Identifiable, Sendable {
    public let id: String
    public let name: String
    public let version: String
    public let category: String
    public let description: String
    public let strategySummary: String
    public let accentColor: Color
    public var isEnabled: Bool
}

public struct RulesCatalogView: View {
    @State private var rules: [RuleItemModel] = [
        RuleItemModel(
            id: "org.openstream.rules.exclusions",
            name: "Исключения Bypass (Direct WAN)",
            version: "2.1.0",
            category: "Network",
            description: "Принудительный пропуск доменов и подсетей напрямую к провайдеру в обход Zapret2 и VPN-шлюзов.",
            strategySummary: "Bypass Return (Direct WAN)",
            accentColor: Color(hex: "06B6D4"),
            isEnabled: SharedConfiguration.shared.isRuleEnabled(ruleId: "org.openstream.rules.exclusions")
        ),
        RuleItemModel(
            id: "org.openstream.rules.twitch",
            name: "Twitch Live Optimizer",
            version: "2.0.0",
            category: "Streaming",
            description: "Раздельная маршрутизация токенов через Ad-Free VPN, 1440p через SmartDNS, прямой CDN без задержек.",
            strategySummary: "Geo-Split (UA/AL) + Direct Video",
            accentColor: Color(hex: "A855F7"),
            isEnabled: SharedConfiguration.shared.isRuleEnabled(ruleId: "org.openstream.rules.twitch")
        ),
        RuleItemModel(
            id: "org.openstream.rules.youtube",
            name: "YouTube Anti-DPI & Clean",
            version: "2.0.0",
            category: "Streaming",
            description: "Локальное разделение TLS ClientHello на границе SNI для CDN googlevideo без ограничений VPN и серверов.",
            strategySummary: "ClientHello Split + Direct 1 Gbps",
            accentColor: Color(hex: "EF4444"),
            isEnabled: SharedConfiguration.shared.isRuleEnabled(ruleId: "org.openstream.rules.youtube")
        ),
        RuleItemModel(
            id: "org.openstream.rules.crunchyroll",
            name: "Crunchyroll Smart Route",
            version: "2.0.0",
            category: "Streaming",
            description: "Разблокировка библиотек США через US Proxy с прямой загрузкой медиапотока на максимальной скорости.",
            strategySummary: "US Egress + Direct CDN",
            accentColor: Color(hex: "F97316"),
            isEnabled: SharedConfiguration.shared.isRuleEnabled(ruleId: "org.openstream.rules.crunchyroll")
        ),
        RuleItemModel(
            id: "org.openstream.rules.adblock",
            name: "Privacy & AdBlock Sinkhole",
            version: "2.0.0",
            category: "Privacy",
            description: "Защита конфиденциальности: блокировка рекламных сетей, трекеров и телеметрии на уровне DNS (0.0.0.0).",
            strategySummary: "DNS Sinkhole (0.0.0.0 / ::)",
            accentColor: Color(hex: "10B981"),
            isEnabled: SharedConfiguration.shared.isRuleEnabled(ruleId: "org.openstream.rules.adblock")
        )
    ]
    
    public init() {}
    
    public var body: some View {
        NavigationStack {
            ZStack {
                Color(hex: "0D0E15").ignoresSafeArea()
                
                ScrollView {
                    VStack(spacing: 16) {
                        headerSection
                        
                        ForEach($rules) { $rule in
                            ruleCard(rule: $rule)
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.top, 10)
                    .padding(.bottom, 30)
                }
            }
            .navigationTitle("Каталог правил")
            .navigationBarTitleDisplayMode(.inline)
        }
    }
    
    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("One Rule. Every Platform.")
                .font(.system(size: 12, weight: .bold, design: .monospaced))
                .foregroundColor(Color(hex: "60A5FA"))
            
            Text("Декларативные пакеты (.osrule.yaml)")
                .font(.system(size: 16, weight: .semibold))
                .foregroundColor(.white)
            
            Text("Каждое правило детерминированно управляет FQDN, стратегиями туннелирования и Anti-DPI.")
                .font(.system(size: 12))
                .foregroundColor(Color(hex: "9CA3AF"))
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, 6)
    }
    
    private func ruleCard(rule: Binding<RuleItemModel>) -> some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 8) {
                        Text(rule.wrappedValue.name)
                            .font(.system(size: 16, weight: .bold))
                            .foregroundColor(.white)
                        
                        Text("v\(rule.wrappedValue.version)")
                            .font(.system(size: 10, weight: .semibold, design: .monospaced))
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.white.opacity(0.1))
                            .foregroundColor(Color(hex: "D1D5DB"))
                            .clipShape(Capsule())
                    }
                    
                    Text(rule.wrappedValue.id)
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundColor(Color(hex: "6B7280"))
                }
                
                Spacer()
                
                // Переключатель активности правила
                Toggle("", isOn: rule.isEnabled)
                    .labelsHidden()
                    .tint(rule.wrappedValue.accentColor)
                    .onChange(of: rule.wrappedValue.isEnabled) { _, newValue in
                        UIImpactFeedbackGenerator(style: .light).impactOccurred()
                        SharedConfiguration.shared.setRuleEnabled(ruleId: rule.wrappedValue.id, enabled: newValue)
                    }
            }
            
            Text(rule.wrappedValue.description)
                .font(.system(size: 13))
                .foregroundColor(Color(hex: "9CA3AF"))
                .lineSpacing(3)
            
            HStack {
                HStack(spacing: 6) {
                    Image(systemName: "arrow.triangle.branch")
                        .font(.system(size: 11))
                    Text(rule.wrappedValue.strategySummary)
                        .font(.system(size: 11, weight: .medium))
                }
                .foregroundColor(rule.wrappedValue.accentColor)
                .padding(.horizontal, 10)
                .padding(.vertical, 5)
                .background(rule.wrappedValue.accentColor.opacity(0.12))
                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                
                Spacer()
                
                Text(rule.wrappedValue.category)
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(Color(hex: "9CA3AF"))
            }
        }
        .padding(18)
        .background(Color(hex: "161922").opacity(0.85))
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .stroke(rule.wrappedValue.isEnabled ? rule.wrappedValue.accentColor.opacity(0.3) : Color.white.opacity(0.06), lineWidth: 1)
        )
    }
}
