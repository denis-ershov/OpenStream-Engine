//
//  SharedConfiguration.swift
//  OpenStream
//
//  Clean Architecture: Модуль синхронизации конфигураций между
//  основным приложением SwiftUI и процессом расширения NetworkExtension (PacketTunnel).
//

import Foundation

public final class SharedConfiguration: Sendable {
    public static let shared = SharedConfiguration()
    
    public static let appGroupIdentifier = "group.org.openstream.engine"
    private let suiteName = "group.org.openstream.engine"
    
    private var sharedDefaults: UserDefaults? {
        UserDefaults(suiteName: suiteName)
    }
    
    private init() {}
    
    /// Общий каталог контейнера App Group
    public func sharedContainerURL() -> URL? {
        FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: SharedConfiguration.appGroupIdentifier)
    }
    
    /// Каталог хранения манифестов сервисных правил (.osrule.yaml)
    public func rulesDirectoryURL() -> URL? {
        guard let container = sharedContainerURL() else { return nil }
        let rulesDir = container.appendingPathComponent("Rules", isDirectory: true)
        if !FileManager.default.fileExists(atPath: rulesDir.path) {
            try? FileManager.default.createDirectory(at: rulesDir, withIntermediateDirectories: true)
        }
        return rulesDir
    }
    
    /// Проверка активности конкретного правила
    public func isRuleEnabled(ruleId: String) -> Bool {
        // По умолчанию включены все предустановленные правила
        guard let defaults = sharedDefaults else { return true }
        if defaults.object(forKey: "rule_\(ruleId)") == nil {
            return true
        }
        return defaults.bool(forKey: "rule_\(ruleId)")
    }
    
    /// Изменение активности правила
    public func setRuleEnabled(ruleId: String, enabled: Bool) {
        sharedDefaults?.set(enabled, forKey: "rule_\(ruleId)")
    }
    
    /// Сохранение YAML-манифеста в общий контейнер
    public func saveRuleManifest(filename: String, yamlContent: String) throws {
        guard let rulesDir = rulesDirectoryURL() else {
            throw NSError(domain: "OpenStream", code: -1, userInfo: [NSLocalizedDescriptionKey: "App Group container unavailable"])
        }
        let fileURL = rulesDir.appendingPathComponent(filename)
        try yamlContent.write(to: fileURL, atomically: true, encoding: .utf8)
    }
}
