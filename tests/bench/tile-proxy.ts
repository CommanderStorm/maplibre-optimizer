#!/usr/bin/env tsx
/**
 * Caching reverse proxy for tile.openfreemap.org.
 *
 * First request for a URL goes upstream; subsequent requests are served
 * from an on-disk cache. Runs as a standalone process so the benchmark
 * harness talks to it over plain HTTP — same codepath as production.
 *
 * Usage:
 *   npx tsx tile-proxy.ts                    # start on port 8765, no throttling
 *   npx tsx tile-proxy.ts --bandwidth 10     # simulate 10 Mbps (fast 4G)
 *   npx tsx tile-proxy.ts --bandwidth 1.5    # simulate 1.5 Mbps (3G)
 *   npx tsx tile-proxy.ts --port 9999        # custom port
 */

import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import Database from "better-sqlite3";
const __dirname = import.meta.dirname!;
const CACHE_DIR = path.join(__dirname, "results", "_tile_cache");
const UPSTREAM = "https://tiles.openfreemap.org";

const argv = process.argv.slice(2);
const portIdx = argv.findIndex((a) => a === "--port");
const PORT = portIdx >= 0 ? parseInt(argv[portIdx + 1], 10) : 8765;
const bwIdx = argv.findIndex((a) => a === "--bandwidth");
/** Simulated bandwidth in megabits per second. 0 = unlimited (no throttling). */
const BANDWIDTH_MBPS = bwIdx >= 0 ? parseFloat(argv[bwIdx + 1]) : 0;
const BYTES_PER_SEC = BANDWIDTH_MBPS > 0 ? (BANDWIDTH_MBPS * 1_000_000) / 8 : 0;

fs.mkdirSync(CACHE_DIR, { recursive: true });

// ── mbtiles serving ──────────────────────────────────────────────────────────

let mbtilesDb: InstanceType<typeof Database> | null = null;
let mbtilesTileStmt: ReturnType<InstanceType<typeof Database>["prepare"]> | null = null;

function loadMbtiles(filePath: string): void {
  closeMbtiles();
  mbtilesDb = new Database(filePath, { readonly: true });
  mbtilesTileStmt = mbtilesDb.prepare(
    "SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?",
  );
  console.log(`Loaded mbtiles: ${filePath}`);
}

function closeMbtiles(): void {
  if (mbtilesDb) {
    mbtilesDb.close();
    mbtilesDb = null;
    mbtilesTileStmt = null;
    console.log("Unloaded mbtiles");
  }
}

// Load initial mbtiles if provided via CLI
const mbtilesIdx = argv.findIndex((a) => a === "--mbtiles");
if (mbtilesIdx >= 0) {
  loadMbtiles(argv[mbtilesIdx + 1]);
}

function cachePath(key: string): string {
  // Use 2-level directory structure to avoid too many files in one dir
  return path.join(CACHE_DIR, key.slice(0, 2), key.slice(2, 4), key);
}

function metaPath(key: string): string {
  return cachePath(key) + ".meta";
}

let hits = 0;
let misses = 0;
let cachedTileJson: string | null = null;

const server = http.createServer(async (req, res) => {
  const url = req.url ?? "/";

  // ── control endpoints ────────────────────────────────────────────────────
  if (url === "/control/load-mbtiles" && req.method === "POST") {
    const chunks: Buffer[] = [];
    for await (const chunk of req) chunks.push(chunk as Buffer);
    try {
      const { path: mbPath } = JSON.parse(Buffer.concat(chunks).toString("utf8"));
      loadMbtiles(mbPath);
      res.writeHead(200);
      res.end("ok");
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      res.writeHead(400);
      res.end(msg);
    }
    return;
  }

  if (url === "/control/unload-mbtiles" && req.method === "POST") {
    closeMbtiles();
    res.writeHead(200);
    res.end("ok");
    return;
  }

  // ── synthetic TileJSON at /sources/openmaptiles ─────────────────────────
  // OpenFreeMap serves TileJSON at `/planet`, not `/sources/openmaptiles`.
  // Fetch once, rewrite tile URLs to route through this proxy, cache in memory.
  if (url === "/sources/openmaptiles") {
    if (!cachedTileJson) {
      try {
        const upstream = await fetch(`${UPSTREAM}/planet`, {
          headers: {
            "User-Agent":
              "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/131 Safari/537.36",
            "Accept": "application/json",
          },
        });
        if (!upstream.ok) {
          res.writeHead(upstream.status, {
            "content-type": "text/plain",
            "access-control-allow-origin": "*",
          });
          res.end(`upstream error: ${upstream.status}`);
          return;
        }
        const json = (await upstream.json()) as {
          tiles?: string[];
          [k: string]: unknown;
        };
        if (Array.isArray(json.tiles)) {
          json.tiles = json.tiles.map((t) =>
            t.replace(/^https?:\/\/tiles\.openfreemap\.org/, `http://localhost:${PORT}`),
          );
        }
        cachedTileJson = JSON.stringify(json);
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        res.writeHead(502, {
          "content-type": "text/plain",
          "access-control-allow-origin": "*",
        });
        res.end(`upstream error: ${msg}`);
        return;
      }
    }
    hits++;
    res.writeHead(200, {
      "content-type": "application/json",
      "content-length": String(Buffer.byteLength(cachedTileJson)),
      "access-control-allow-origin": "*",
    });
    res.end(cachedTileJson);
    return;
  }

  // ── mbtiles tile serving: /mbtiles/{z}/{x}/{y} ──────────────────────────
  const mbtilesMatch = url.match(/^\/mbtiles\/(\d+)\/(\d+)\/(\d+)$/);
  if (mbtilesMatch) {
    if (!mbtilesTileStmt) {
      res.writeHead(404);
      res.end("no mbtiles loaded");
      return;
    }
    const z = parseInt(mbtilesMatch[1], 10);
    const x = parseInt(mbtilesMatch[2], 10);
    const xyzY = parseInt(mbtilesMatch[3], 10);
    // MBTiles uses TMS y-coordinate convention
    const tmsY = (1 << z) - 1 - xyzY;

    const row = mbtilesTileStmt.get(z, x, tmsY) as { tile_data: Buffer } | undefined;
    if (!row) {
      res.writeHead(404);
      res.end("tile not found");
      return;
    }

    const tileData = row.tile_data;

    if (BYTES_PER_SEC > 0) {
      const delayMs = (tileData.length / BYTES_PER_SEC) * 1000;
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }

    hits++;
    res.writeHead(200, {
      "content-type": "application/octet-stream",
      "content-length": String(tileData.length),
      "access-control-allow-origin": "*",
    });
    res.end(tileData);
    return;
  }

  // ── upstream proxy with caching ──────────────────────────────────────────
  const upstreamUrl = UPSTREAM + url;
  const key = upstreamUrl.replace("https://tiles.openfreemap.org/", "");
  const body = cachePath(key);
  const meta = metaPath(key);

  // Serve from cache
  if (fs.existsSync(body) && fs.existsSync(meta)) {
    hits++;
    const { status, headers } = JSON.parse(fs.readFileSync(meta, "utf8"));
    // Always advertise CORS regardless of what upstream sent — MapLibre runs
    // under `file://` (Puppeteer's `about:blank`) and needs `Access-Control-
    // Allow-Origin: *` on every response or the fetch is rejected.
    headers["access-control-allow-origin"] = "*";

    if (BYTES_PER_SEC > 0) {
      // Simulate network transfer time proportional to response size
      const size = fs.statSync(body).size;
      const delayMs = (size / BYTES_PER_SEC) * 1000;
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }

    res.writeHead(status, headers);
    fs.createReadStream(body).pipe(res);
    return;
  }

  // Fetch upstream, cache, and serve
  misses++;
  try {
    const upstream = await fetch(upstreamUrl, {
      headers: {
        "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/131 Safari/537.36",
        "Accept": req.headers.accept ?? "*/*",
      },
    });
    const respBody = Buffer.from(await upstream.arrayBuffer());
    const headers: Record<string, string> = {};
    upstream.headers.forEach((v, k) => {
      headers[k] = v;
    });
    // Remove headers that shouldn't be cached/forwarded
    delete headers["transfer-encoding"];
    delete headers["content-encoding"];
    headers["content-length"] = String(respBody.length);
    headers["access-control-allow-origin"] = "*";

    // Only cache successful responses. Caching a 403/404/5xx would pin the
    // proxy to that error indefinitely (we hit this when openfreemap briefly
    // rejected `/sources/openmaptiles` — the bad response was served for
    // weeks afterwards, silently invalidating every benchmark run).
    if (upstream.status >= 200 && upstream.status < 300) {
      const dir = path.dirname(body);
      fs.mkdirSync(dir, { recursive: true });
      fs.writeFileSync(body, respBody);
      fs.writeFileSync(
        meta,
        JSON.stringify({ status: upstream.status, headers }),
      );
    }

    res.writeHead(upstream.status, headers);
    res.end(respBody);
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    res.writeHead(502, { "access-control-allow-origin": "*" });
    res.end(`upstream error: ${msg}`);
  }
});

server.listen(PORT, () => {
  console.log(`Tile caching proxy listening on http://localhost:${PORT}`);
  console.log(`Cache directory: ${CACHE_DIR}`);
  console.log(`Upstream: ${UPSTREAM}`);
  if (mbtilesDb) {
    console.log(`MBTiles: loaded (serving on /mbtiles/{z}/{x}/{y})`);
  } else {
    console.log("MBTiles: none (use --mbtiles <path> or POST /control/load-mbtiles)");
  }
  if (BANDWIDTH_MBPS > 0) {
    console.log(`Bandwidth throttle: ${BANDWIDTH_MBPS} Mbps (${(BYTES_PER_SEC / 1024).toFixed(0)} KB/s)`);
  } else {
    console.log("Bandwidth throttle: off (use --bandwidth <Mbps> to simulate network)");
  }
  console.log("Press Ctrl+C to stop\n");
});

// Print stats periodically
setInterval(() => {
  if (hits + misses > 0) {
    console.log(
      `  cache stats: ${hits} hits, ${misses} misses (${((hits / (hits + misses)) * 100).toFixed(0)}% hit rate)`,
    );
  }
}, 10_000);
