from __future__ import annotations

import argparse
import json
import sys
from http import HTTPStatus
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from v1_frames import FrameParams, PARAM_LIMITS, coerce_params, generate_payload

CACHE: dict[str, dict] = {}


def parse_args():
    parser = argparse.ArgumentParser(description="Serve the Bressloff V1 browser viewer with on-demand model runs.")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8892)
    return parser.parse_args()


def first_values(query: dict[str, list[str]]) -> dict[str, str]:
    return {key: values[0] for key, values in query.items() if values}


def cache_key(params: FrameParams) -> str:
    return json.dumps(params.__dict__, sort_keys=True, separators=(",", ":"))


class V1ViewerHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def json_response(self, status: HTTPStatus, payload: dict):
        body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        parsed = urlparse(self.path)
        if parsed.path == "/api/defaults":
            defaults = FrameParams()
            self.json_response(
                HTTPStatus.OK,
                {
                    "defaults": defaults.__dict__,
                    "limits": PARAM_LIMITS,
                    "resolution_options": [32, 48, 64, 80, 96],
                    "orientation_options": [4, 8, 12, 16, 24],
                    "solver_options": ["preview", "accurate"],
                    "colormaps": ["twilight", "viridis", "magma", "inferno", "turbo", "gray"],
                },
            )
            return

        if parsed.path == "/api/generate":
            try:
                raw = first_values(parse_qs(parsed.query))
                params = coerce_params(raw)
                key = cache_key(params)
                payload = CACHE.get(key)
                if payload is None:
                    payload = generate_payload(params)
                    CACHE[key] = payload
                self.json_response(HTTPStatus.OK, payload)
            except Exception as error:
                self.json_response(HTTPStatus.INTERNAL_SERVER_ERROR, {"error": str(error)})
            return

        super().do_GET()


def main():
    args = parse_args()
    server = ThreadingHTTPServer((args.host, args.port), V1ViewerHandler)
    print(f"Serving Bressloff V1 viewer on http://{args.host}:{args.port}/viewer/index.html", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
