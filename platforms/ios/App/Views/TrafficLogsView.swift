//
//  TrafficLogsView.swift
//  OpenStream
//
//  Журнал перехваченных сетевых запросов и примененных вердиктов в реальном времени.
//

import SwiftUI

public struct TrafficLogEntry: Identifiable, Sendable {
    public let id = UUID()
    public let timestamp: Date
    public let domain: String
    public let verdictTitle: String
    public let verdictType: VerdictLogType
    public let latencyMs: Int
}

public enum VerdictLogType: Sendable {
    case direct
    case block
    case antiDpi
    case proxy
    
    var color: Color {
        switch self {
        case .direct: return Color(hex: "34D399")
        case .block: return Color(hex: "F87171")
        case .antiDpi: return Color(hex: "C084FC")
        case .proxy: return Color(hex: "60A5FA")
        }
    }
}

public struct TrafficLogsView: View {
    @State private var filter: String = "All"
    @State private var searchText: String = ""
    
    @State private var sampleLogs: [TrafficLogEntry] = [
        TrafficLogEntry(timestamp: Date().addingTimeInterval(-2), domain: "gql.twitch.tv", verdictTitle: "PROXY:UA", verdictType: .proxy, latencyMs: 24),
        TrafficLogEntry(timestamp: Date().addingTimeInterval(-5), domain: "rr1---sn-4g5ednks.googlevideo.com", verdictTitle: "ANTI-DPI SPLIT", verdictType: .antiDpi, latencyMs: 8),
        TrafficLogEntry(timestamp: Date().addingTimeInterval(-10), domain: "edge.ads.twitch.tv", verdictTitle: "SINKHOLE 0.0.0.0", verdictType: .block, latencyMs: 1),
        TrafficLogEntry(timestamp: Date().addingTimeInterval(-15), domain: "video-edge-1.live-video.net", verdictTitle: "DIRECT WAN", verdictType: .direct, latencyMs: 12),
        TrafficLogEntry(timestamp: Date().addingTimeInterval(-25), domain: "usher.ttvnw.net", verdictTitle: "SMARTDNS 1440P", verdictType: .proxy, latencyMs: 31),
        TrafficLogEntry(timestamp: Date().addingTimeInterval(-35), domain: "telemetry.sdk.split.io", verdictTitle: "SINKHOLE 0.0.0.0", verdictType: .block, latencyMs: 1)
    ]
    
    public init() {}
    
    public var body: some View {
        NavigationStack {
            ZStack {
                Color(hex: "0D0E15").ignoresSafeArea()
                
                VStack(spacing: 12) {
                    // Селектор фильтра
                    filterPicker
                        .padding(.horizontal, 16)
                        .padding(.top, 8)
                    
                    ScrollView {
                        LazyVStack(spacing: 10) {
                            ForEach(filteredLogs) { log in
                                logRow(log: log)
                            }
                        }
                        .padding(.horizontal, 16)
                        .padding(.vertical, 8)
                    }
                }
            }
            .navigationTitle("Журнал трафика")
            .navigationBarTitleDisplayMode(.inline)
            .searchable(text: $searchText, prompt: "Поиск по домену...")
        }
    }
    
    private var filterPicker: some View {
        HStack(spacing: 8) {
            filterButton(title: "Все", tag: "All")
            filterButton(title: "Блокировка", tag: "Block")
            filterButton(title: "Anti-DPI", tag: "AntiDpi")
            filterButton(title: "Прокси", tag: "Proxy")
        }
    }
    
    private func filterButton(title: String, tag: String) -> some View {
        Button(action: {
            filter = tag
        }) {
            Text(title)
                .font(.system(size: 12, weight: .medium))
                .padding(.horizontal, 12)
                .padding(.vertical, 6)
                .background(filter == tag ? Color(hex: "3B82F6") : Color(hex: "1F2430"))
                .foregroundColor(filter == tag ? .white : Color(hex: "9CA3AF"))
                .clipShape(Capsule())
        }
    }
    
    private var filteredLogs: [TrafficLogEntry] {
        sampleLogs.filter { item in
            let matchesSearch = searchText.isEmpty || item.domain.localizedCaseInsensitiveContains(searchText)
            let matchesFilter: Bool
            switch filter {
            case "Block": matchesFilter = item.verdictType == .block
            case "AntiDpi": matchesFilter = item.verdictType == .antiDpi
            case "Proxy": matchesFilter = item.verdictType == .proxy
            default: matchesFilter = true
            }
            return matchesSearch && matchesFilter
        }
    }
    
    private func logRow(log: TrafficLogEntry) -> some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(log.domain)
                    .font(.system(size: 13, weight: .semibold, design: .monospaced))
                    .foregroundColor(.white)
                    .lineLimit(1)
                
                Text(log.timestamp.formatted(date: .omitted, time: .standard))
                    .font(.system(size: 11))
                    .foregroundColor(Color(hex: "6B7280"))
            }
            
            Spacer()
            
            VStack(alignment: .trailing, spacing: 4) {
                Text(log.verdictTitle)
                    .font(.system(size: 10, weight: .bold, design: .monospaced))
                    .padding(.horizontal, 8)
                    .padding(.vertical, 3)
                    .background(log.verdictType.color.opacity(0.15))
                    .foregroundColor(log.verdictType.color)
                    .clipShape(Capsule())
                
                Text("\(log.latencyMs) мс")
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundColor(Color(hex: "9CA3AF"))
            }
        }
        .padding(14)
        .background(Color(hex: "161922").opacity(0.85))
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
    }
}
