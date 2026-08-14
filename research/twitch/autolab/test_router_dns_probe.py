import unittest

from router_dns_probe import ad_markers, parse_master_variants


class RouterDnsProbeTests(unittest.TestCase):
    def test_detects_stitched_ad_only(self) -> None:
        playlist = '#EXTM3U\n#EXT-X-DATERANGE:CLASS="twitch-stitched-ad"\n'
        self.assertIn("twitch-stitched-ad", ad_markers(playlist))
        self.assertIn("EXT-X-DATERANGE:twitch-stitched-ad", ad_markers(playlist))

    def test_ignores_non_ad_daterange(self) -> None:
        self.assertEqual([], ad_markers('#EXTM3U\n#EXT-X-DATERANGE:CLASS="twitch-session"\n'))

    def test_parses_master_variants(self) -> None:
        master = '#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=1280x720\n720.m3u8\n'
        result = parse_master_variants(master)
        self.assertEqual("1280x720", result[0]["resolution"])
        self.assertEqual("720.m3u8", result[0]["uri"])
