import json
import httpx
import re
from urllib.parse import quote

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"

def test_active_channels():
    with httpx.Client(headers={"Client-ID": TWITCH_CLIENT_ID, "User-Agent": "Mozilla/5.0"}, timeout=10.0) as client:
        # 1. Geo-IP
        try:
            ip_info = client.get("http://ip-api.com/json").json()
            print(f"[*] Current Public IP: {ip_info.get('query')} ({ip_info.get('country')}, {ip_info.get('city')}, ISP: {ip_info.get('isp')})")
        except Exception as e:
            print(f"[*] IP Info Error: {e}")

        # 2. Get top 5 live streams
        r = client.post("https://gql.twitch.tv/gql", json={"query": "query { streams(first: 5) { edges { node { broadcaster { login } viewersCount } } } }"})
        edges = r.json().get("data", {}).get("streams", {}).get("edges", [])
        
        print(f"\n[*] Found {len(edges)} top active streams:")
        for edge in edges:
            ch = edge["node"]["broadcaster"]["login"]
            viewers = edge["node"]["viewersCount"]
            print(f"\n--- Channel: {ch} ({viewers} viewers) ---")
            
            tok_r = client.post("https://gql.twitch.tv/gql", json={
                "operationName": "PlaybackAccessToken_Template",
                "query": """query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
                  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
                    value signature
                  }
                }""",
                "variables": {"isLive": True, "login": ch, "playerType": "site"}
            })
            tok_obj = tok_r.json().get("data", {}).get("streamPlaybackAccessToken", {})
            val = tok_obj.get("value")
            sig = tok_obj.get("signature")
            if not val:
                print("  [!] Failed to get token")
                continue
                
            tok_json = json.loads(val)
            print(f"  [GQL Token] show_ads: {tok_json.get('show_ads')} | server_ads: {tok_json.get('server_ads')} | hide_ads: {tok_json.get('hide_ads')}")
            
            master_url = f"https://usher.ttvnw.net/api/channel/hls/{ch}.m3u8?client_id={TWITCH_CLIENT_ID}&token={quote(val, safe='')}&sig={quote(sig, safe='')}&allow_source=true"
            master_r = client.get(master_url)
            if master_r.status_code != 200:
                print(f"  [Usher] HTTP {master_r.status_code}")
                continue
                
            media_urls = [line.strip() for line in master_r.text.splitlines() if line.startswith("http")]
            if media_urls:
                media_r = client.get(media_urls[0])
                has_ads = "twitch-stitched-ad" in media_r.text or "EXT-X-DATERANGE" in media_r.text
                print(f"  [Media Playlist] Ads Present (SSAI): {has_ads}")
                if has_ads:
                    for line in media_r.text.splitlines():
                        if "stitched-ad" in line:
                            print(f"    -> {line[:100]}...")

if __name__ == "__main__":
    test_active_channels()
