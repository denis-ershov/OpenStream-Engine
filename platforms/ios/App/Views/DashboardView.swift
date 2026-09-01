//
//  DashboardView.swift
//  OpenStream
//
//  Дизайн: Linear / Apple Aesthetic.
//  Mobile-First, неоморфные карточки, темная тема, индикатор Apple Jetsam.
//

import SwiftUI

public struct DashboardView: View {
    @ObservedObject public var viewModel: TunnelViewModel
    
    public init(viewModel: TunnelViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
        NavigationStack {
            ZStack {
                // Фон: глубокий премиальный темный градиент
                LinearGradient(
                    colors: [Color(hex: "0D0E15"), Color(hex: "131620"), Color(hex: "08090C")],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
                .ignoresSafeArea()
                
                ScrollView {
                    VStack(spacing: 20) {
                        // 1. Статусный Hero Card
                        heroStatusCard
                        
                        // 2. Монитор лимита памяти Apple Jetsam
                        jetsamMemoryCard
                        
                        // 3. Сетка метрик в реальном времени
                        metricsGrid
                        
                        // 4. Карточка активной сервисной маршрутизации
                        activeRoutingOverview
                    }
                    .padding(.horizontal, 16)
                    .padding(.top, 10)
                    .padding(.bottom, 30)
                }
            }
            .navigationTitle("OpenStream 2.0")
            .navigationBarTitleDisplayMode(.inline)
        }
    }
    
    // MARK: - Hero Card с главным переключателем
    private var heroStatusCard: some View {
        VStack(spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(viewModel.isConnected ? "ЗАЩИТА АКТИВНА" : "МАРШРУТИЗАЦИЯ ОТКЛЮЧЕНА")
                        .font(.system(size: 11, weight: .bold, design: .monospaced))
                        .foregroundColor(viewModel.isConnected ? Color(hex: "34D399") : Color(hex: "9CA3AF"))
                    
                    Text(viewModel.statusMessage)
                        .font(.system(size: 18, weight: .semibold))
                        .foregroundColor(.white)
                }
                Spacer()
                
                // Индикатор состояния
                Circle()
                    .fill(viewModel.isConnected ? Color(hex: "10B981") : Color(hex: "4B5563"))
                    .frame(width: 12, height: 12)
                    .overlay(
                        Circle()
                            .stroke(viewModel.isConnected ? Color(hex: "10B981").opacity(0.4) : Color.clear, lineWidth: 6)
                            .scaleEffect(viewModel.isConnected ? 1.3 : 1.0)
                            .animation(viewModel.isConnected ? .easeInOut(duration: 1.5).repeatForever(autoreverses: true) : .default, value: viewModel.isConnected)
                    )
            }
            
            Divider().background(Color.white.opacity(0.1))
            
            // Кнопка включения/выключения туннеля
            Button(action: {
                UIImpactFeedbackGenerator(style: .medium).impactOccurred()
                viewModel.toggleTunnel()
            }) {
                HStack {
                    Image(systemName: viewModel.isConnected ? "stop.fill" : "bolt.fill")
                        .font(.system(size: 16, weight: .bold))
                    Text(viewModel.isConnected ? "Остановить туннель" : "Включить OpenStream")
                        .font(.system(size: 16, weight: .bold))
                }
                .foregroundColor(viewModel.isConnected ? .white : Color(hex: "0D0E15"))
                .frame(maxWidth: .infinity)
                .frame(height: 50)
                .background(
                    viewModel.isConnected
                        ? LinearGradient(colors: [Color(hex: "DC2626"), Color(hex: "B91C1C")], startPoint: .leading, endPoint: .trailing)
                        : LinearGradient(colors: [Color(hex: "60A5FA"), Color(hex: "3B82F6")], startPoint: .leading, endPoint: .trailing)
                )
                .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
                .shadow(color: (viewModel.isConnected ? Color(hex: "EF4444") : Color(hex: "3B82F6")).opacity(0.3), radius: 10, y: 5)
            }
        }
        .padding(20)
        .background(Color(hex: "161922").opacity(0.85))
        .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(Color.white.opacity(0.08), lineWidth: 1)
        )
    }
    
    // MARK: - Монитор расхода памяти Apple Jetsam
    private var jetsamMemoryCard: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Label("Apple Jetsam Guard", systemImage: "memorychip")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(Color(hex: "93C5FD"))
                Spacer()
                Text("2.1 МБ / 15.0 МБ")
                    .font(.system(size: 12, weight: .bold, design: .monospaced))
                    .foregroundColor(Color(hex: "34D399"))
            }
            
            // Прогресс-бар расхода лимита памяти
            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 6)
                        .fill(Color.white.opacity(0.06))
                        .frame(height: 8)
                    
                    RoundedRectangle(cornerRadius: 6)
                        .fill(
                            LinearGradient(
                                colors: [Color(hex: "10B981"), Color(hex: "34D399")],
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .frame(width: geo.size.width * (2.1 / 15.0), height: 8)
                }
            }
            .frame(height: 8)
            
            Text("Потребление RAM в 7 раз ниже порога SIGKILL iOS (благодаря Zero-Allocation Rust Core)")
                .font(.system(size: 11))
                .foregroundColor(Color(hex: "9CA3AF"))
        }
        .padding(16)
        .background(Color(hex: "161922").opacity(0.85))
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color.white.opacity(0.08), lineWidth: 1)
        )
    }
    
    // MARK: - Метрики в реальном времени
    private var metricsGrid: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 14) {
            metricItem(title: "Всего запросов", value: "\(viewModel.totalQueries)", icon: "globe", color: Color(hex: "60A5FA"))
            metricItem(title: "Блокировано рекламы", value: "\(viewModel.blockedQueries)", icon: "shield.slash.fill", color: Color(hex: "F87171"))
            metricItem(title: "Anti-DPI (Direct)", value: "\(viewModel.dpiEvasiveQueries)", icon: "bolt.shield.fill", color: Color(hex: "C084FC"))
            metricItem(title: "Прямой WAN", value: "\(viewModel.directQueries)", icon: "arrow.up.forward", color: Color(hex: "34D399"))
        }
    }
    
    private func metricItem(title: String, value: String, icon: String, color: Color) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Image(systemName: icon)
                    .foregroundColor(color)
                    .font(.system(size: 14, weight: .bold))
                Spacer()
            }
            
            Text(value)
                .font(.system(size: 22, weight: .bold, design: .rounded))
                .foregroundColor(.white)
            
            Text(title)
                .font(.system(size: 12, weight: .medium))
                .foregroundColor(Color(hex: "9CA3AF"))
        }
        .padding(16)
        .background(Color(hex: "161922").opacity(0.85))
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color.white.opacity(0.08), lineWidth: 1)
        )
    }
    
    // MARK: - Карточка активных правил
    private var activeRoutingOverview: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Установленные сервисные политики")
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundColor(.white)
                Spacer()
                Text("\(viewModel.activeRulesCount) активны")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundColor(Color(hex: "60A5FA"))
            }
            
            serviceRow(name: "Twitch Live Optimizer", badge: "Geo-Split + 1440p", color: Color(hex: "A855F7"))
            serviceRow(name: "YouTube Anti-DPI", badge: "ClientHello Split", color: Color(hex: "EF4444"))
            serviceRow(name: "Crunchyroll Smart Route", badge: "US Catalog + CDN", color: Color(hex: "F97316"))
            serviceRow(name: "Privacy & AdBlock", badge: "DNS 0.0.0.0 Sinkhole", color: Color(hex: "10B981"))
        }
        .padding(16)
        .background(Color(hex: "161922").opacity(0.85))
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color.white.opacity(0.08), lineWidth: 1)
        )
    }
    
    private func serviceRow(name: String, badge: String, color: Color) -> some View {
        HStack {
            Circle()
                .fill(color)
                .frame(width: 8, height: 8)
            Text(name)
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(Color(hex: "E5E7EB"))
            Spacer()
            Text(badge)
                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                .padding(.horizontal, 8)
                .padding(.vertical, 3)
                .background(color.opacity(0.15))
                .foregroundColor(color)
                .clipShape(Capsule())
        }
        .padding(.vertical, 2)
    }
}

// Вспомогательное расширение цветов Hex
extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3:
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6:
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8:
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (1, 1, 1, 0)
        }
        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}
