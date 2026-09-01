//
//  OpenStreamApp.swift
//  OpenStream
//
//  Точка входа iOS-приложения OpenStream Engine 2.0.
//  Mobile-First дизайн, вкладки, инициализация эталонных правил в контейнере App Group.
//

import SwiftUI

@main
struct OpenStreamApp: App {
    @StateObject private var tunnelViewModel = TunnelViewModel()
    
    init() {
        // Установка премиального темного стиля для навигационных панелей
        let appearance = UINavigationBarAppearance()
        appearance.configureWithOpaqueBackground()
        appearance.backgroundColor = UIColor(Color(hex: "0D0E15"))
        appearance.titleTextAttributes = [.foregroundColor: UIColor.white]
        appearance.largeTitleTextAttributes = [.foregroundColor: UIColor.white]
        
        UINavigationBar.appearance().standardAppearance = appearance
        UINavigationBar.appearance().scrollEdgeAppearance = appearance
        
        // Синхронизация эталонных правил в контейнер App Group
        deployReferenceRulesIfNeeded()
    }
    
    var body: some Scene {
        WindowGroup {
            TabView {
                DashboardView(viewModel: tunnelViewModel)
                    .tabItem {
                        Label("Статус", systemImage: "bolt.shield.fill")
                    }
                
                RulesCatalogView()
                    .tabItem {
                        Label("Правила", systemImage: "square.stack.3d.up.fill")
                    }
                
                TrafficLogsView()
                    .tabItem {
                        Label("Журнал", systemImage: "list.bullet.rectangle.fill")
                    }
            }
            .tint(Color(hex: "3B82F6"))
            .preferredColorScheme(.dark)
        }
    }
    
    /// Копирование встроенных правил (.osrule.yaml) в общую память App Group
    private func deployReferenceRulesIfNeeded() {
        let twitchYaml = """
schema_version: "2.0"
id: "org.openstream.rules.twitch"
name: "Twitch Live Optimizer"
version: "2.0.0"

matches:
  - group: "auth_token"
    domains: ["gql.twitch.tv"]
    strategy: "adfree_egress"
  - group: "master_playlist"
    domains: ["usher.ttvnw.net"]
    strategy: "quality_unlock"
  - group: "video_cdn"
    domains: ["*.live-video.net", "*.ttvnw.net"]
    action: "direct"
  - group: "ads"
    domains: ["edge.ads.twitch.tv"]
    action: "block"

strategies:
  adfree_egress:
    preference: ["geo:al", "geo:ua", "direct"]
  quality_unlock:
    preference: ["smartdns:eu", "direct"]
"""
        let youtubeYaml = """
schema_version: "2.0"
id: "org.openstream.rules.youtube"
name: "YouTube Anti-DPI & Clean"
version: "2.0.0"

matches:
  - group: "video_playback"
    domains: ["*.googlevideo.com", "*.ytimg.com"]
    action: "dpi_evasive_direct"
  - group: "ads"
    domains: ["youtubeads.g.doubleclick.net"]
    action: "block"
"""
        _ = try? SharedConfiguration.shared.saveRuleManifest(filename: "twitch.osrule.yaml", yamlContent: twitchYaml)
        _ = try? SharedConfiguration.shared.saveRuleManifest(filename: "youtube.osrule.yaml", yamlContent: youtubeYaml)
    }
}
