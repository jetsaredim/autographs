#!/usr/bin/env python3
"""Freeze the live Caddy publish routes without replacing its bind-mounted inode."""

import argparse
import os
from pathlib import Path


PROXY = "\thandle /admin/api/* {"
FULL = "handle /admin/api/publish/full {"
INCREMENTAL = "handle /admin/api/publish/incremental {"
STATUS = "handle /admin/api/publish/status {"
FROZEN_HEADER = 'header X-Autographs-Publish-Frozen "true"'
RESPONSE = 'respond "Publishing temporarily unavailable" 503'
MARKER = "# BEGIN AUTOGRAPHS PUBLISH FREEZE"
END_MARKER = "# END AUTOGRAPHS PUBLISH FREEZE"
STATUS_GATE = '''handle /admin/api/publish/status {
  header Cache-Control "no-store"
  header X-Autographs-Publish-Frozen "true"
  reverse_proxy autographs-controller:8080
}
'''
GATE = MARKER + '\n' + STATUS_GATE + '''handle /admin/api/publish/full {
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
        counts = (content.count(FULL), content.count(INCREMENTAL), content.count(STATUS))
        marker_pair = content.count(MARKER) == 1 and content.count(END_MARKER) == 1
        gate_before_proxy = marker_pair and content.index(MARKER) < content.index(END_MARKER) < content.index(PROXY)
        gate_body = content[content.index(MARKER):content.index(END_MARKER)] if gate_before_proxy else ""
        if counts == (1, 1, 1):
            if (not gate_before_proxy or gate_body.count(RESPONSE) != 2 or gate_body.count(FROZEN_HEADER) != 1
                    or gate_body.count('header Cache-Control "no-store"') != 3
                    or not all(route in gate_body for route in (FULL, INCREMENTAL, STATUS))):
                raise ValueError("live publish gate is incomplete")
            return "already_frozen"
        if counts == (1, 1, 0):
            if (not gate_before_proxy or gate_body.count(RESPONSE) != 2
                    or gate_body.count(FROZEN_HEADER) != 0
                    or gate_body.count('header Cache-Control "no-store"') != 2
                    or not all(route in gate_body for route in (FULL, INCREMENTAL))):
                raise ValueError("live publish gate is partial or ambiguous")
            caddyfile.seek(0)
            caddyfile.write(content.replace(MARKER + '\n', MARKER + '\n' + STATUS_GATE, 1))
            caddyfile.truncate()
            caddyfile.flush()
            os.fsync(caddyfile.fileno())
            return "upgraded"
        if counts != (0, 0, 0) or MARKER in content or RESPONSE in content or FROZEN_HEADER in content:
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
