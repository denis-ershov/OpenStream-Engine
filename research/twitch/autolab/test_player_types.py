import socket
import httpx
import json
from urllib.parse import quote, urljoin
import re

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"

# Базовый шаблон запроса PlaybackAccessToken с динамической платформой
GQL_QUERY = """
query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!, $platform: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: $platform, playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
    value
    signature
  }
}
""".strip()

_original_getaddrinfo = socket.getaddrinfo

# DoH разрешение для gql.twitch.tv
def resolve_doh(host):
    try:
        r = httpx.get(
            "https://cloudflare-dns.com/dns-query",
            params={"name": host, "type": "A"},
            headers={"accept": "application/dns-json"}
        )
        for answer in r.json().get("Answer", []):
            if answer.get("type") == 1:
                return answer.get("data")
    except Exception:
        pass
    return None

gql_real_ip = resolve_doh("gql.twitch.tv")

# Список комбинаций для проверки
COMBOS = [
    # (playerType, platform, name)
    ("site", "web", "Standard Web"),
    ("embed", "web", "Embed Web"),
    ("embed", "iframe", "Embed iframe"),
    ("samsung_tv", "smarttv", "Samsung TV"),
    ("lg_tv", "smarttv", "LG TV"),
    ("android_tv", "android", "Android TV"),
    ("apple_tv", "ios", "Apple TV"),
    ("fire_tv", "amazon", "Fire TV"),
    ("twitch_luna", "luna", "Twitch Luna"),
]

def check_combo(player_type, platform, via_doh=False):
    # Устанавливаем DNS патч если нужно
    if via_doh and gql_real_ip:
        def custom_getaddrinfo(host, port, family=0, type=0, proto=0, flags=0):
            if host == "gql.twitch.tv":
                return _original_getaddrinfo(gql_real_ip, port, family, type, proto, flags)
            return _original_getaddrinfo(host, port, family, type, proto, flags)
        socket.getaddrinfo = custom_getaddrinfo
    else:
        socket.getaddrinfo = _original_getaddrinfo

    payload = {
        "operationName": "PlaybackAccessToken_Template",
        "query": GQL_QUERY,
        "variables": {
            "isLive": True,
            "login": "gaules",
            "playerType": player_type,
            "platform": platform,
        },
    }

    try:
        # Для ТВ платформ иногда нужен другой Client-ID или User-Agent, но сначала проверим со стандартным
        with httpx.Client(headers={"User-Agent": "Mozilla/5.0", "Client-ID": TWITCH_CLIENT_ID}, timeout=10) as client:
            r = client.post("https://gql.twitch.tv/gql", json=payload)
            if r.status_code != 200:
                return {"ok": False, "status": r.status_code}
            
            data = r.json()
            token_obj = (data.get("data") or {}).get("streamPlaybackAccessToken")
            if not token_obj:
                return {"ok": False, "error": "No token returned"}
            
            token_val = token_obj["value"]
            token_json = json.loads(token_val)
            
            return {
                "ok": True,
                "show_ads": token_json.get("show_ads"),
                "server_ads": token_json.get("server_ads"),
                "adblock": token_json.get("adblock"),
                "hide_ads": token_json.get("hide_ads"),
                "user_ip": token_json.get("user_ip"),
                "player_type": token_json.get("player_type"),
                "platform": token_json.get("platform"),
                "max_res": token_json.get("maximum_resolution"),
            }
    except Exception as e:
        return {"ok": False, "error": str(e)}

print("=== Testing GQL PlayerTypes via System DNS (EU/SmartDNS Proxy) ===")
for p_type, plat, desc in COMBOS:
    res = check_combo(p_type, plat, via_doh=False)
    if res.get("ok"):
        print(f"[{desc}] show_ads: {res['show_ads']} | server_ads: {res['server_ads']} | hide_ads: {res['hide_ads']} | max_res: {res['max_res']}")
    else:
        print(f"[{desc}] Failed: {res.get('error') or res.get('status')}")

print("\n=== Testing GQL PlayerTypes via DoH (Direct РФ IP) ===")
for p_type, plat, desc in COMBOS:
    res = check_combo(p_type, plat, via_doh=True)
    if res.get("ok"):
        print(f"[{desc}] show_ads: {res['show_ads']} | server_ads: {res['server_ads']} | hide_ads: {res['hide_ads']} | max_res: {res['max_res']}")
    else:
        print(f"[{desc}] Failed: {res.get('error') or res.get('status')}")

socket.getaddrinfo = _original_getaddrinfo
