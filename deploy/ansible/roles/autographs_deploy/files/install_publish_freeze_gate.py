#!/usr/bin/env python3
"""Freeze the live Caddy publish routes without replacing its bind-mounted inode."""

import argparse
import os
from pathlib import Path


PROXY = "\thandle /admin/api/* {"
FULL = "handle /admin/api/publish/full {"
INCREMENTAL = "handle /admin/api/publish/incremental {"
RESPONSE = 'respond "Publishing temporarily unavailable" 503'
MARKER = "# BEGIN AUTOGRAPHS PUBLISH FREEZE"
GATE = '''# BEGIN AUTOGRAPHS PUBLISH FREEZE
handle /admin/api/publish/full {
  header Cache-Control "no-store"
  respond "Publishing temporarily unavailable" 503
}
handle /admin/api/publish/incremental {
  header Cache-Control "no-store"
  respond "Publishing temporarily unavailable" 503
}
# END AUTOGRAPHS PUBLISH FREEZE

'''


def install(path: Path) -> str:
    if path.is_symlink() or not path.is_file():
        raise ValueError("live Caddyfile must be a regular file")
    with path.open("r+", encoding="utf-8") as caddyfile:
        content = caddyfile.read()
        if content.count(PROXY) != 1:
            raise ValueError("live Caddyfile has no unique admin API proxy")
        counts = (content.count(FULL), content.count(INCREMENTAL))
        if counts == (1, 1):
            if content.count(RESPONSE) != 2 or content.count('header Cache-Control "no-store"') < 2:
                raise ValueError("live publish gate is incomplete")
            return "already_frozen"
        if counts != (0, 0) or MARKER in content or RESPONSE in content:
            raise ValueError("live publish gate is partial or ambiguous")
        caddyfile.seek(0)
        caddyfile.write(content.replace(PROXY, GATE + PROXY, 1))
        caddyfile.truncate()
        caddyfile.flush()
        os.fsync(caddyfile.fileno())
    return "installed"


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("caddyfile", type=Path)
    args = parser.parse_args()
    try:
        print(install(args.caddyfile))
    except ValueError as error:
        parser.exit(1, f"{error}\n")
