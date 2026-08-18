import json
import httpx
import re
from urllib.parse import quote

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"

# GQL to find top live channels
QUERY_TOP = """
query {
  streams(first: 10) {
    edges {
      node {
        broadcaster {
          login
          displayName
        }
        viewersCount
        game {
          name
        }
      }
    }
  }
}
"""

GQL_TOKEN_QUERY = """
query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
    value
    signature
  }
}
"""

def probe():
    with httpx.Client(headers={"Client-ID": TWITCH_CLIENT_ID, "User-Agent": "Mozilla/5.0"}, timeout=10.0) as client:
        # Get live streams
        r = client.post("https://gql.twitch.tv/gql", json={"query": QUERY_TOP})
        data = r.json()
        streams = data.get("data", {}).get("streams", {}).get("edges", [])
        
        print(f"Found {len(streams)} top live streams:")
        for edge in streams:
            node = edge.get("node", {})
            channel = node.get("broadcaster", {}).get("login")
            viewers = node.get("viewersCount")
            game = (node.get("game") or {}).get("name")
            print(f" - {channel} ({viewers} viewers, {game})")
            
        print("\n" + "="*50)
        
        # Test the top 3 channels
        for edge in streams[:3]:
            channel = edge.get("node", {}).get("broadcaster", {}).get("login")
            print(f"\n>>> TESTING CHANNEL: {channel} <<<")
            
            # 1. GQL Token
            tok_r = client.post("https://gql.twitch.tv/gql", json={
                "operationName": "PlaybackAccessToken_Template",
                "query": GQL_TOKEN_QUERY,
                "variables": {"isLive": True, "login": channel, "playerType": "site"}
            })
            tok_data = tok_r.json().get("data", {}).get("streamPlaybackAccessToken", {})
            val = tok_data.get("value")
            sig = tok_data.get("signature")
            if not val:
                print("Failed to get token")
                continue
                
            tok_json = json.loads(val)
            print(f"Token: show_ads={tok_json.get('show_ads')}, server_ads={tok_json.get('server_ads')}, hide_ads={tok_json.get('hide_ads')}, ip={tok_json.get('user_ip')}")
            
            # 2. Usher Master Playlist
            master_url = f"https://usher.ttvnw.net/api/channel/hls/{channel}.m3u8?client_id={TWITCH_CLIENT_ID}&token={quote(val, safe='')}&sig={quote(sig, safe='')}&allow_source=true"
            master_r = client.get(master_url)
            print(f"Master playlist status: {master_r.status_code}")
            if master_r.status_code != 200:
                print("Master body:", master_r.text[:200])
                continue
                
            media_urls = [line.strip() for line in master_r.text.splitlines() if line.startswith("http")]
            print(f"Variants in master: {len(media_urls)}")
            
            # Check qualities
            qualities = re.findall(r"NAME=\"([^\"]+)\"", master_r.text)
            print(f"Qualities: {qualities}")
            
            # 3. Media Playlist
            if media_urls:
                media_r = client.get(media_urls[0])
                media_body = media_r.text
                has_ads = any(m in media_body.lower() for m in ["stitched", "daterange", "x-tv-twitch-ad", "amazon"])
                print(f"Media playlist has ads: {has_ads}")
                
                # Print sample of media playlist
                lines = media_body.splitlines()
                print("--- Media Playlist Sample ---")
                for line in lines[:20]:
                    print(line)
                if has_ads:
                    print("--- Ad Lines Found ---")
                    for line in lines:
                        if any(m in line.lower() for m in ["stitched", "daterange", "x-tv-twitch-ad", "amazon", "prefetch"]):
                            print(line)

if __name__ == "__main__":
    probe()
