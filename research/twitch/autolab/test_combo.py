import socket
import httpx
import json
from urllib.parse import quote, urljoin, urlparse
import re

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"
GQL_QUERY = """
query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
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
        data = r.json()
        for answer in data.get("Answer", []):
            if answer.get("type") == 1: # A record
                return answer.get("data")
    except Exception as e:
        print("DoH failed:", e)
    return None

gql_real_ip = resolve_doh("gql.twitch.tv")
print(f"Real IP for gql.twitch.tv from DoH: {gql_real_ip}")

# Патчим socket.getaddrinfo для перенаправления gql.twitch.tv на реальный IP
def custom_getaddrinfo(host, port, family=0, type=0, proto=0, flags=0):
    if host == "gql.twitch.tv" and gql_real_ip:
        # Возвращаем адресную информацию для реального IP, но для оригинального хоста
        res = _original_getaddrinfo(gql_real_ip, port, family, type, proto, flags)
        return res
    return _original_getaddrinfo(host, port, family, type, proto, flags)

socket.getaddrinfo = custom_getaddrinfo

# 1. Шаг 1: Получаем токен напрямую (через реальный IP / ISP РФ)
print("\n--- Step 1: Fetching PlaybackAccessToken (Direct via Real IP) ---")
payload = {
    "operationName": "PlaybackAccessToken_Template",
    "query": GQL_QUERY,
    "variables": {
        "isLive": True,
        "login": "gaules",
        "playerType": "site",
    },
}

with httpx.Client(headers={"User-Agent": "Mozilla/5.0", "Client-ID": TWITCH_CLIENT_ID}) as client:
    r = client.post("https://gql.twitch.tv/gql", json=payload)
    print("GQL Status:", r.status_code)
    gql_data = r.json()
    token_obj = (gql_data.get("data") or {}).get("streamPlaybackAccessToken")
    if not token_obj:
        print("Failed to get token!")
        print(gql_data)
        exit(1)
    
    token = token_obj["value"]
    sig = token_obj["signature"]
    
    # Расшифруем токен для логов
    token_json = json.loads(token)
    print("Token show_ads:", token_json.get("show_ads"))
    print("Token server_ads:", token_json.get("server_ads"))
    print("Token user_ip (how Twitch sees us):", token_json.get("user_ip"))

# Отключаем патч для запроса к usher (чтобы он шел через Fake-IP / прокси)
socket.getaddrinfo = _original_getaddrinfo

# 2. Шаг 2: Получаем master-плейлист через системный DNS (Fake-IP / прокси в Европе)
print("\n--- Step 2: Fetching master playlist (via System DNS / Proxy) ---")
path = (
    f"/api/channel/hls/gaules.m3u8"
    f"?client_id={TWITCH_CLIENT_ID}"
    f"&token={quote(token, safe='')}"
    f"&sig={quote(sig, safe='')}"
    f"&allow_source=true&allow_audio_only=true"
)
url = f"https://usher.ttvnw.net{path}"

with httpx.Client(headers={"User-Agent": "Mozilla/5.0", "Client-ID": TWITCH_CLIENT_ID}) as client:
    r = client.get(url)
    print("Usher Status:", r.status_code)
    master_body = r.text
    print(f"Master playlist size: {len(master_body)} bytes")
    
    # Ищем доступные разрешения
    resolutions = re.findall(r"RESOLUTION=(\d+x\d+)", master_body)
    print("Available resolutions in master:", resolutions)

    # Ищем ссылку на медиа-плейлист качеств
    lines = master_body.splitlines()
    media_url = None
    for line in lines:
        if line and not line.startswith("#"):
            media_url = line.strip()
            break
            
    if media_url:
        print("\n--- Step 3: Fetching media playlist (Direct / System DNS) ---")
        # Делаем запрос к медиа-плейлисту напрямую
        r_media = client.get(media_url)
        print("Media Status:", r_media.status_code)
        media_body = r_media.text
        
        # Ищем рекламу
        ads_found = []
        for marker in ("stitched", "EXT-X-DATERANGE", "X-TV-TWITCH-AD", "Amazon"):
            if marker.lower() in media_body.lower():
                ads_found.append(marker)
        
        if ads_found:
            print("ADS FOUND:", ads_found)
            # Выведем кусок плейлиста с рекламой для наглядности
            print("\nPreview of media playlist:")
            print("\n".join(media_body.splitlines()[:25]))
        else:
            print("NO ADS FOUND! SUCCESS!")
            print("\nPreview of media playlist:")
            print("\n".join(media_body.splitlines()[:25]))
