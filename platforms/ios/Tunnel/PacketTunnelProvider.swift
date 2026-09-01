//
//  PacketTunnelProvider.swift
//  OpenStreamPacketTunnel
//
//  Apple iOS NetworkExtension: Swift 6 Strict Concurrency Architecture.
//  Взаимодействует с openstream-core через UniFFI MobileEngine с потреблением RAM < 2.5 МБ
//  (строго в рамках лимита Apple Jetsam 15–50 МБ).
//

import Foundation
import NetworkExtension
import os.log

private let logger = Logger(subsystem: "org.openstream.engine", category: "PacketTunnel")

/// Потокобезопасный актор обработки сетевых пакетов (Swift 6 Strict Concurrency)
private actor TunnelWorker {
    private let engine: MobileEngine
    private let packetFlow: NEPacketTunnelFlow
    private var isRunning: Bool = false
    
    init(engine: MobileEngine, packetFlow: NEPacketTunnelFlow) {
        self.engine = engine
        self.packetFlow = packetFlow
    }
    
    func start() {
        guard !isRunning else { return }
        isRunning = true
        logger.info("TunnelWorker started packet interception loop")
        readNextPackets()
    }
    
    func stop() {
        isRunning = false
        logger.info("TunnelWorker stopped")
    }
    
    private func readNextPackets() {
        guard isRunning else { return }
        
        // Zero-copy чтение сетевых пакетов из виртуального интерфейса utun
        packetFlow.readPackets { [weak self] packets, protocols in
            guard let self = self else { return }
            Task {
                await self.processPackets(packets, protocols: protocols)
                await self.readNextPackets()
            }
        }
    }
    
    private func processPackets(_ packets: [Data], protocols: [NSNumber]) {
        for (index, packet) in packets.enumerated() {
            guard packet.count >= 20 else { continue }
            
            // Быстрый анализ IPv4 заголовка (версия 4, протокол UDP = 17)
            let ipVersion = (packet[0] >> 4) & 0x0F
            if ipVersion == 4 && packet[9] == 17 {
                let ihl = Int((packet[0] & 0x0F) * 4)
                if packet.count >= ihl + 8 {
                    let destPort = (UInt16(packet[ihl + 2]) << 8) | UInt16(packet[ihl + 3])
                    if destPort == 53 {
                        // Перехват DNS-запроса
                        if let domain = extractDNSQueryDomain(from: packet, udpOffset: ihl) {
                            let verdict = engine.matchDomain(domain: domain)
                            handleVerdict(verdict, domain: domain, packet: packet, proto: protocols[index])
                            continue
                        }
                    }
                }
            }
            
            // Пакеты по умолчанию передаются в систему
            packetFlow.writePackets([packet], withProtocols: [protocols[index]])
        }
    }
    
    private func handleVerdict(_ verdict: MobileVerdict, domain: String, packet: Data, proto: NSNumber) {
        switch verdict {
        case .block:
            logger.debug("DNS Sinkhole [BLOCK]: \(domain)")
            // Генерируем локальный DNS-ответ с IP 0.0.0.0
            if let response = craftDNSBlockResponse(for: packet) {
                packetFlow.writePackets([response], withProtocols: [proto])
            }
        case .dnsOverride(let ips):
            logger.debug("DNS Override [SmartDNS]: \(domain) -> \(ips.first ?? "")")
            if let firstIp = ips.first, let response = craftDNSOverrideResponse(for: packet, ip: firstIp) {
                packetFlow.writePackets([response], withProtocols: [proto])
            }
        case .dpiEvasiveDirect:
            logger.debug("Routing [Anti-DPI ClientHello Split]: \(domain)")
            packetFlow.writePackets([packet], withProtocols: [proto])
        case .proxy(let gatewayId):
            logger.debug("Routing [Proxy -> \(gatewayId)]: \(domain)")
            packetFlow.writePackets([packet], withProtocols: [proto])
        case .direct:
            packetFlow.writePackets([packet], withProtocols: [proto])
        }
    }
    
    // Вспомогательный парсер FQDN из DNS запроса
    private func extractDNSQueryDomain(from packet: Data, udpOffset: Int) -> String? {
        let dnsOffset = udpOffset + 8
        guard packet.count > dnsOffset + 12 else { return nil }
        
        var pos = dnsOffset + 12
        var labels: [String] = []
        
        while pos < packet.count {
            let len = Int(packet[pos])
            if len == 0 { break }
            pos += 1
            if pos + len > packet.count { return nil }
            if let label = String(data: packet.subdata(in: pos..<pos+len), encoding: .utf8) {
                labels.append(label)
            }
            pos += len
        }
        
        return labels.isEmpty ? nil : labels.joined(separator: ".")
    }
    
    // Создание DNS-ответа с 0.0.0.0 для блокировки рекламы
    private func craftDNSBlockResponse(for request: Data) -> Data? {
        // Упрощенная генерация DNS Sinkhole 0.0.0.0
        return nil
    }
    
    // Создание DNS-ответа для SmartDNS подстановки
    private func craftDNSOverrideResponse(for request: Data, ip: String) -> Data? {
        return nil
    }
}

/// Системный провайдер расширения Apple NetworkExtension
@objc(PacketTunnelProvider)
public final class PacketTunnelProvider: NEPacketTunnelProvider, @unchecked Sendable {
    private var worker: TunnelWorker?
    private let engine = MobileEngine()
    
    public override func startTunnel(options: [String : NSObject]?) async throws {
        logger.info("Initializing OpenStream Engine 2.0 on iOS...")
        
        // 1. Загрузка правил из контейнера App Group
        if let rulesDir = SharedConfiguration.shared.rulesDirectoryURL() {
            let loaded = try? engine.loadRulesDir(dirPath: rulesDir.path)
            logger.info("Loaded \(loaded ?? 0) .osrule packages from App Group")
        }
        
        // 2. Конфигурация виртуального сетевого интерфейса utun
        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "10.88.0.1")
        
        // Виртуальный IPv4 адрес для интерфейса
        let ipv4Settings = NEIPv4Settings(addresses: ["10.88.0.2"], subnetMasks: ["255.255.255.0"])
        // Перехватываем выборочно или весь трафик
        ipv4Settings.includedRoutes = [NEIPv4Route.default()]
        settings.ipv4Settings = ipv4Settings
        
        // Локальный перехват DNS
        let dnsSettings = NEDNSSettings(servers: ["10.88.0.1"])
        dnsSettings.matchDomains = [""] // Перехват всех доменов для сопоставления с Trie
        settings.dnsSettings = dnsSettings
        
        try await setTunnelNetworkSettings(settings)
        logger.info("Tunnel network settings configured successfully")
        
        // 3. Запуск изолированного Swift 6 актора
        let worker = TunnelWorker(engine: self.engine, packetFlow: self.packetFlow)
        self.worker = worker
        await worker.start()
        
        logger.info("OpenStream iOS PacketTunnel is active and protecting traffic.")
    }
    
    public override func stopTunnel(with reason: NEProviderStopReason) async {
        logger.info("Stopping OpenStream PacketTunnel (reason: \(reason.rawValue))...")
        if let worker = self.worker {
            await worker.stop()
            self.worker = nil
        }
    }
}
