import socket
import urllib.request
import json
from urllib.parse import quote

print("=========================================================")
print("=== ТЕСТИРОВАНИЕ СЦЕНАРИЕВ БЕЗ VPN: DNS И ТОКЕНЫ TWITCH ===")
print("=========================================================\n")

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"
GQL_QUERY = """
query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
    value
    signature
  }
}
""".strip()

def query_dns(domain, dns_ip):
    import random
    tx_id = random.randint(0, 65535)
    header = tx_id.to_bytes(2, 'big') + (0x0100).to_bytes(2, 'big') + (1).to_bytes(2, 'big') + (0).to_bytes(6, 'big')
    qname = b''.join(len(p).to_bytes(1, 'big') + p.encode('ascii') for p in domain.split('.')) + b'\x00'
    packet = header + qname + (1).to_bytes(2, 'big') + (1).to_bytes(2, 'big')
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(3.0)
    try:
        sock.sendto(packet, (dns_ip, 53))
        data, _ = sock.recvfrom(1024)
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
        return ips
    except Exception as e:
        return []

# Тестируем DNS-серверы
dns_tests = {
    "Яндекс DNS (РФ Anycast)": "77.88.8.8",
    "Cloudflare DNS (EU/Global)": "1.1.1.1",
    "Google DNS (Global)": "8.8.8.8",
    "Quad9 DNS (Швейцария/EU)": "9.9.9.9"
}

print("1. Резолв доменов через разные DNS-провайдеры:")
resolved = {}
for name, ip in dns_tests.items():
    gql_ips = query_dns("gql.twitch.tv", ip)
    usher_ips = query_dns("usher.ttvnw.net", ip)
    resolved[name] = {"gql": gql_ips, "usher": usher_ips}
    print(f"  [{name} {ip}]")
    print(f"    gql.twitch.tv   -> {', '.join(gql_ips[:2])}")
    print(f"    usher.ttvnw.net -> {', '.join(usher_ips[:2])}")

# 2. Получение токена Twitch
print("\n2. Анализ токена Twitch (PlaybackAccessToken):")

def fetch_token(channel="dota2ti_ru", target_ip=None):
    url = "https://gql.twitch.tv/gql"
    payload = {
        "operationName": "PlaybackAccessToken_Template",
        "query": GQL_QUERY,
        "variables": {
            "isLive": True,
            "login": channel,
            "playerType": "site"
        }
    }
    
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode('utf-8'),
        headers={
            "Client-ID": TWITCH_CLIENT_ID,
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Content-Type": "application/json"
        }
    )
    
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            data = json.loads(resp.read().decode('utf-8'))
            tok_obj = data.get("data", {}).get("streamPlaybackAccessToken", {})
            raw_val = tok_obj.get("value", "")
            if raw_val:
                t_json = json.loads(raw_val)
                return {
                    "ok": True,
                    "channel": channel,
                    "user_ip": t_json.get("user_ip"),
                    "has_preroll": t_json.get("has_preroll"),
                    "geo": t_json.get("geo"),
                    "ad_properties": t_json.get("ad_properties"),
                    "auth": t_json.get("authorization"),
                    "signature": tok_obj.get("signature", "")[:16] + "..."
                }
            return {"ok": False, "raw": data}
    except Exception as e:
        return {"ok": False, "error": str(e)}

res = fetch_token("dota2ti_ru")
print(json.dumps(res, indent=2, ensure_ascii=False))

# 3. Тестирование запроса мастер-плейлиста с usher.ttvnw.net
if res.get("ok"):
    print("\n3. Запрос мастер-плейлиста с usher.ttvnw.net:")
    token_str = json.dumps(res) # or fetch raw value
