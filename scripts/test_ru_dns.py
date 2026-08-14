import socket
import time
import urllib.request
import json

print("===================================================================")
print("=== ТЕСТИРОВАНИЕ 5 РОССИЙСКИХ DNS-СЕРВИСОВ ДЛЯ OPENSTREAM / TWITCH ===")
print("===================================================================\n")

dns_list = {
    "Яндекс DNS (Базовый)": ["77.88.8.8", "77.88.8.1"],
    "Яндекс DNS (Безопасный)": ["77.88.8.88", "77.88.8.2"],
    "Яндекс DNS (Семейный)": ["77.88.8.7", "77.88.8.3"],
    "MSK-IX DNS": ["62.76.76.62", "62.76.62.76"],
    "SkyDNS": ["193.58.251.251"],
    "Comss.one DNS (SmartDNS)": ["83.220.169.155", "212.109.195.93"],
    "НСДИ (Гос. резерв)": ["195.208.4.1", "195.208.5.1"],
    "Cloudflare (Контрольный EU)": ["1.1.1.1"]
}

domains = [
    ("gql.twitch.tv", "🔑 Токен авторизации"),
    ("usher.ttvnw.net", "📺 Мастер-плейлист"),
    ("edge.ads.twitch.tv", "🚫 Сервер рекламы"),
    ("video-weaver.fra02.hls.ttvnw.net", "🚀 Видеопоток CDN")
]

def query_dns_udp(domain, dns_ip, timeout=2.0):
    import random
    tx_id = random.randint(0, 65535)
    header = tx_id.to_bytes(2, 'big') + (0x0100).to_bytes(2, 'big') + (1).to_bytes(2, 'big') + (0).to_bytes(6, 'big')
    qname = b''.join(len(p).to_bytes(1, 'big') + p.encode('ascii') for p in domain.split('.')) + b'\x00'
    packet = header + qname + (1).to_bytes(2, 'big') + (1).to_bytes(2, 'big')
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(timeout)
    t0 = time.perf_counter()
    try:
        sock.sendto(packet, (dns_ip, 53))
        data, _ = sock.recvfrom(1024)
        rtt = (time.perf_counter() - t0) * 1000
        sock.close()
        
        ips = []
        offset = 12 + len(qname) + 4
        while offset < len(data):
            if offset + 12 > len(data): break
            if (data[offset] & 0xc0) == 0xc0: offset += 2
            else:
                while offset < len(data) and data[offset] != 0: offset += 1 + data[offset]
                offset += 1
            rtype = int.from_bytes(data[offset:offset+2], 'big')
            rdlength = int.from_bytes(data[offset+8:offset+10], 'big')
            offset += 10
            if rtype == 1 and rdlength == 4:
                ips.append(f"{data[offset]}.{data[offset+1]}.{data[offset+2]}.{data[offset+3]}")
            offset += rdlength
        return ips, round(rtt, 1), None
    except Exception as e:
        return [], 0, str(e)

results_table = {}

for svc_name, ips in dns_list.items():
    primary_ip = ips[0]
    print(f"\n[*] DNS: {svc_name} [{primary_ip}]")
    results_table[svc_name] = {}
    
    for d, desc in domains:
        resolved_ips, rtt, err = query_dns_udp(d, primary_ip)
        if err:
            print(f"  {d:35s}: ERROR ({err})")
            results_table[svc_name][d] = {"status": "FAIL", "error": err}
        else:
            ip_str = ", ".join(resolved_ips[:3]) if resolved_ips else "NXDOMAIN / 0.0.0.0"
            print(f"  {d:35s}: {ip_str} [{rtt} ms]")
            results_table[svc_name][d] = {"status": "OK", "ips": resolved_ips, "rtt": rtt}

print("\n===================================================================")
print("=== COMPARATIVE ANALYSIS ===")
print("===================================================================")

# Проверяем Comss.one на SmartDNS подмену
comss_res = results_table.get("Comss.one DNS (SmartDNS)", {})
yandex_res = results_table.get("Яндекс DNS (Базовый)", {})
cf_res = results_table.get("Cloudflare (Контрольный EU)", {})

print("1. Comss.one DNS vs Yandex DNS Analysis:")
for d, desc in domains:
    c_ips = comss_res.get(d, {}).get("ips", [])
    y_ips = yandex_res.get(d, {}).get("ips", [])
    print(f"  - {d}:")
    print(f"      Comss.one -> {c_ips}")
    print(f"      Yandex    -> {y_ips}")

