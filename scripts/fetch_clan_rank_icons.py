# /// script
# requires-python = ">=3.10"
# dependencies = ["pillow"]
# ///
"""Downloads the OSRS clan rank icons from the OSRS Wiki into
trackscape-discord-api/assets/clan_rank_icons/, named after the Discord emoji
they are uploaded as (see trackscape-discord-api/src/rank_emojis.rs).

The originals are 13x13, so each is scaled up with nearest-neighbour to stay
crisp at Discord's emoji size. Re-run when the game adds rank titles:

    uv run scripts/fetch_clan_rank_icons.py
"""

import io
import json
import re
import time
import urllib.parse
import urllib.request
from pathlib import Path

from PIL import Image

WIKI = "https://oldschool.runescape.wiki"
USER_AGENT = "trackscape-discord rank icon fetch (github.com/fatfingers23/trackscape-discord)"
OUT_DIR = Path(__file__).parent.parent / "trackscape-discord-api/assets/clan_rank_icons"
RANK_PREFIX = "Clan_icon_-_"
# The icon the game shows for the clan's own broadcasts
CLAN_ICON = "Your_Clan_icon.png"
# Discord recommends 128x128 for emoji
MAX_SIZE = 128


def get(url: str) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request) as response:
        return response.read()


def emoji_name(rank: str) -> str:
    """Must match rank_emoji_name in rank_emojis.rs"""
    return "rank_" + re.sub(r"[^a-z0-9]", "_", rank.lower())


def save(wiki_file: str, name: str) -> None:
    image = Image.open(io.BytesIO(get(f"{WIKI}/images/{urllib.parse.quote(wiki_file)}")))
    image = image.convert("RGBA")
    scale = max(1, MAX_SIZE // max(image.size))
    image = image.resize((image.width * scale, image.height * scale), Image.NEAREST)
    image.save(OUT_DIR / f"{name}.png", optimize=True)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    page = json.loads(
        get(f"{WIKI}/api.php?action=parse&page=Clan_Settings&prop=images&format=json")
    )
    rank_files = [f for f in page["parse"]["images"] if f.startswith(RANK_PREFIX)]

    for wiki_file in rank_files:
        rank = wiki_file[len(RANK_PREFIX) : -len(".png")]
        save(wiki_file, emoji_name(rank))
        time.sleep(0.1)
    save(CLAN_ICON, "clan_icon")
    print(f"Saved {len(rank_files) + 1} icons to {OUT_DIR}")


if __name__ == "__main__":
    main()
