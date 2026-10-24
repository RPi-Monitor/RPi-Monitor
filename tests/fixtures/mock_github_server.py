#!/usr/bin/env python3
"""
Simple mock server for GitHub Releases API and distribution asset downloads.
Used for isolated unit and integration testing of install.sh.
"""

import http.server
import json
import os
import sys
import hashlib
from urllib.parse import urlparse

PORT = int(os.environ.get("MOCK_PORT", 8080))
ASSETS_DIR = os.environ.get("ASSETS_DIR", "/tmp/mock_assets")
DEFAULT_VERSION = os.environ.get("MOCK_VERSION", "2.14")
CORRUPT_CHECKSUM = os.environ.get("MOCK_CORRUPT_CHECKSUM", "0") == "1"
OMIT_ZIP_ASSET = os.environ.get("MOCK_OMIT_ZIP_ASSET", "0") == "1"
OMIT_SHA256_ASSET = os.environ.get("MOCK_OMIT_SHA256_ASSET", "0") == "1"
INVALID_ZIP = os.environ.get("MOCK_INVALID_ZIP", "0") == "1"
EMPTY_ZIP = os.environ.get("MOCK_EMPTY_ZIP", "0") == "1"

class GitHubMockHandler(http.server.BaseHTTPRequestHandler):
    def _send_json(self, status_code, data):
        body = json.dumps(data).encode("utf-8")
        self.send_response(status_code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _send_file(self, file_path, content_type="application/octet-stream"):
        if not os.path.exists(file_path):
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not Found")
            return

        with open(file_path, "rb") as f:
            content = f.read()

        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(content)))
        self.end_headers()
        self.wfile.write(content)

    def _get_release_dict(self, tag_name, version, is_prerelease=False,
                          published_at="2026-01-01T00:00:00Z"):
        base_url = f"http://{self.headers.get('Host', f'127.0.0.1:{PORT}')}"
        asset_name = f"rpimonitor-{version}.zip"
        sha_name = f"{asset_name}.sha256"

        assets = []
        if not OMIT_ZIP_ASSET:
            assets.append({
                "name": asset_name,
                "browser_download_url": f"{base_url}/download/{asset_name}"
            })
        if not OMIT_SHA256_ASSET:
            assets.append({
                "name": sha_name,
                "browser_download_url": f"{base_url}/download/{sha_name}"
            })

        return {
            "tag_name": tag_name,
            "draft": False,
            "prerelease": is_prerelease,
            "published_at": published_at,
            "assets": assets,
        }

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path

        # Healthcheck
        if path == "/health":
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b"OK")
            return

        # /releases/latest
        if path.endswith("/releases/latest"):
            version = DEFAULT_VERSION
            tag_name = f"v{version}"
            self._send_json(200, self._get_release_dict(tag_name, version, False))
            return

        # /releases?per_page=100 or /releases
        if "/releases" in path and not path.endswith("/latest") and "/tags/" not in path:
            version = DEFAULT_VERSION
            # Pre-release is newer by published_at so --pre-release selects it
            # deterministically over the stable release.
            releases = [
                self._get_release_dict(f"v{version}", version, False,
                                       "2026-01-01T00:00:00Z"),
                self._get_release_dict(f"v{version}-rc1", f"{version}-rc1", True,
                                       "2026-02-01T00:00:00Z"),
            ]
            self._send_json(200, releases)
            return

        # /releases/tags/<tag>
        if "/releases/tags/" in path:
            tag = path.split("/releases/tags/")[-1]
            version = tag.lstrip("v")
            self._send_json(200, self._get_release_dict(tag, version, "-rc" in tag))
            return

        # /download/<asset_name>
        if path.startswith("/download/"):
            filename = os.path.basename(path)
            file_path = os.path.join(ASSETS_DIR, filename)

            if filename.endswith(".sha256") and CORRUPT_CHECKSUM:
                # Return corrupted hash
                corrupt_body = f"0000000000000000000000000000000000000000000000000000000000000000  {filename[:-7]}\n".encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/plain")
                self.send_header("Content-Length", str(len(corrupt_body)))
                self.end_headers()
                self.wfile.write(corrupt_body)
                return

            if filename.endswith(".zip") and EMPTY_ZIP:
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Length", "0")
                self.end_headers()
                return

            if filename.endswith(".zip") and INVALID_ZIP:
                # Non-empty payload without ZIP magic bytes (PK\x03\x04)
                bad_body = b"NOT_A_VALID_ZIP_ARCHIVE"
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Length", str(len(bad_body)))
                self.end_headers()
                self.wfile.write(bad_body)
                return

            self._send_file(file_path, "application/octet-stream" if filename.endswith(".zip") else "text/plain")
            return

        self.send_response(404)
        self.end_headers()
        self.wfile.write(b"Endpoint Not Found")

    def log_message(self, format, *args):
        # Suppress noisy HTTP logs during test execution unless DEBUG is set
        if os.environ.get("DEBUG"):
            super().log_message(format, *args)

def run():
    server = http.server.HTTPServer(("0.0.0.0", PORT), GitHubMockHandler)
    sys.stderr.write(f"[MockServer] Listening on http://0.0.0.0:{PORT} (Serving assets from {ASSETS_DIR})\n")
    server.serve_forever()

if __name__ == "__main__":
    run()
