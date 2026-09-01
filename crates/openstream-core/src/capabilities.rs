bitflags::bitflags! {
    /// Матрица физических возможностей платформенного сетевого бэкенда
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct PlatformCapabilities: u32 {
        /// Возможность перехвата и подмены DNS-запросов (53/UDP, DoH/DoT)
        const DNS_INTERCEPTION   = 1 << 0;
        /// Маршрутизация по доменному имени / SNI хоста
        const DOMAIN_ROUTING     = 1 << 1;
        /// Маршрутизация по IP-адресам / CIDR подсетям
        const IP_ROUTING         = 1 << 2;
        /// Маршрутизация по имени процесса / идентификатору приложения (package_name)
        const PROCESS_ROUTING    = 1 << 3;
        /// Локальная модификация тела ответов (HLS/DASH manifest stripping)
        const L7_PAYLOAD_STRIP   = 1 << 4;
        /// Низкоуровневая десинхронизация TCP/TLS пакетов (Anti-DPI / ClientHello split)
        const L4_PACKET_DESYNC   = 1 << 5;
        /// Полный захват L3 пакетов через виртуальный интерфейс TUN
        const FULL_TUNNEL        = 1 << 6;
        /// Аппаратная/ядерная фильтрация без подъема в userspace (nftables Zero-Copy)
        const ZERO_COPY_KERNEL   = 1 << 7;
    }
}
