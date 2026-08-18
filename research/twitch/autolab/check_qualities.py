import json
import httpx
import re
from urllib.parse import quote

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"

def test_master_qualities():
    with httpx.Client(headers={"Client-ID": TWITCH_CLIENT_ID, "User-Agent": "Mozilla/5.0"}, timeout=10.0) as client:
        # Get live streams
        r = client.post("https://gql.twitch.tv/gql", json={"query": "query { streams(first: 5) { edges { node { broadcaster { login displayName } viewersCount } } } }"})
        edges = r.json().get("data", {}).get("streams", {}).get("edges", [])
        
        for edge in edges[:3]:
            ch = edge["node"]["broadcaster"]["login"]
            print(f"\n==========================================")
            print(f"CHANNEL: {ch}")
            print(f"==========================================")
            
            # GQL Token
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
            
            val_json = json.loads(val)
            print(f"Token max_res: {val_json.get('maximum_resolution')}")
            
            # Query Usher with allow_source=true
            usher_url = (
                f"https://usher.ttvnw.net/api/channel/hls/{ch}.m3u8"
                f"?client_id={TWITCH_CLIENT_ID}"
                f"&token={quote(val, safe='')}"
                f"&sig={quote(sig, safe='')}"
                f"&allow_source=true"
                f"&allow_audio_only=true"
                f"&fast_bread=true"
            )
            
            master_r = client.get(usher_url)
            print(f"Master status: {master_r.status_code}")
            
            # Parse EXT-X-STREAM-INF and media streams
            lines = master_r.text.splitlines()
            current_info = {}
            for line in lines:
                if line.startswith("#EXT-X-STREAM-INF:"):
                    # Extract bandwidth, resolution, framerate, video
                    bw = re.search(r"BANDWIDTH=(\d+)", line)
                    res = re.search(r"RESOLUTION=(\d+x\d+)", line)
                    fps = re.search(r"FRAME-RATE=([0-9.]+)", line)
                    name = re.search(r"VIDEO=\"([^\"]+)\"", line)
                    current_info = {
                        "name": name.group(1) if name else "Unknown",
                        "resolution": res.group(1) if res else "Unknown",
                        "fps": round(float(fps.group(1))) if fps else "Unknown",
                        "bitrate_kbps": round(int(bw.group(1))/1000) if bw else "Unknown",
                    }
                elif line.startswith("http") and current_info:
                    print(f" -> Quality: {current_info['name']:<12} | Resolution: {current_info['resolution']:<10} | FPS: {current_info['fps']} | Bitrate: {current_info['bitrate_kbps']} kbps")
                    current_info = {}

if __name__ == "__main__":
    test_master_qualities()
