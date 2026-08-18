import json
import httpx
import re
from urllib.parse import quote

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"

GQL_TOKEN_QUERY = """
query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
    value
    signature
  }
}
"""

def test_stream_probe(channel="tarik"):
    print(f"=== STREAM EXPERIMENT: CHANNEL '{channel}' ===")
    with httpx.Client(headers={"Client-ID": TWITCH_CLIENT_ID, "User-Agent": "Mozilla/5.0"}, timeout=10.0) as client:
        # 1. Check our public IP
        ip_r = client.get("https://api.ipify.org?format=json")
        my_ip = ip_r.json().get("ip")
        print(f"[1] Current Egress IP: {my_ip}")
        
        # 2. Check Geo-IP location
        try:
            geo_r = client.get(f"https://ipapi.co/{my_ip}/json/")
            geo = geo_r.json()
            print(f"[2] Geo-IP: {geo.get('country_name')} ({geo.get('country_code')}), City: {geo.get('city')}, Org: {geo.get('org')}")
        except Exception as e:
            print(f"[2] Geo-IP error: {e}")

        # 3. Request GQL token with different player types
        player_types = ["site", "embed", "samsung_tv", "twitch_luna"]
        for pt in player_types:
            r = client.post("https://gql.twitch.tv/gql", json={
                "operationName": "PlaybackAccessToken_Template",
                "query": GQL_TOKEN_QUERY,
                "variables": {"isLive": True, "login": channel, "playerType": pt}
            })
            tok_obj = r.json().get("data", {}).get("streamPlaybackAccessToken", {})
            val = tok_obj.get("value")
            if val:
                val_json = json.loads(val)
                print(f"    PlayerType '{pt}': show_ads={val_json.get('show_ads')}, server_ads={val_json.get('server_ads')}, hide_ads={val_json.get('hide_ads')}")

        # 4. Use standard token to query Usher
        r = client.post("https://gql.twitch.tv/gql", json={
            "operationName": "PlaybackAccessToken_Template",
            "query": GQL_TOKEN_QUERY,
            "variables": {"isLive": True, "login": channel, "playerType": "site"}
        })
        tok_obj = r.json().get("data", {}).get("streamPlaybackAccessToken", {})
        val = tok_obj.get("value")
        sig = tok_obj.get("signature")

        master_url = f"https://usher.ttvnw.net/api/channel/hls/{channel}.m3u8?client_id={TWITCH_CLIENT_ID}&token={quote(val, safe='')}&sig={quote(sig, safe='')}&allow_source=true"
        master_r = client.get(master_url)
        print(f"\n[3] Usher master playlist response: HTTP {master_r.status_code}")
        
        media_urls = [line.strip() for line in master_r.text.splitlines() if line.startswith("http")]
        if media_urls:
            print(f"[4] First media playlist URL: {media_urls[0][:70]}...")
            media_r = client.get(media_urls[0])
            has_ad = "twitch-stitched-ad" in media_r.text or "EXT-X-DATERANGE" in media_r.text
            print(f"[5] SSAI Ad markers found in media playlist: {has_ad}")
            
            # Print ad tags if found
            if has_ad:
                for line in media_r.text.splitlines():
                    if "stitched-ad" in line or "EXTINF" in line and "Amazon" in line:
                        print(f"    -> {line[:120]}")

if __name__ == "__main__":
    # Test on active big channels
    test_stream_probe("tarik")
