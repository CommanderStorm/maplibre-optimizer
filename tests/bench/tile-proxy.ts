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

function cachePath(key: string): string {
  // Use 2-level directory structure to avoid too many files in one dir
  return path.join(CACHE_DIR, key.slice(0, 2), key.slice(2, 4), key);
}

function metaPath(key: string): string {
  return cachePath(key) + ".meta";
}

let hits = 0;
let misses = 0;

const server = http.createServer(async (req, res) => {
  const upstreamUrl = UPSTREAM + (req.url ?? "/");
  const key = upstreamUrl.replace("https://tiles.openfreemap.org/", "");
  const body = cachePath(key);
  const meta = metaPath(key);

  // Serve from cache
  if (fs.existsSync(body) && fs.existsSync(meta)) {
    hits++;
    const { status, headers } = JSON.parse(fs.readFileSync(meta, "utf8"));

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

    // Write to cache
    const dir = path.dirname(body);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(body, respBody);
    fs.writeFileSync(
      meta,
      JSON.stringify({ status: upstream.status, headers }),
    );

    res.writeHead(upstream.status, headers);
    res.end(respBody);
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    res.writeHead(502);
    res.end(`upstream error: ${msg}`);
  }
});

server.listen(PORT, () => {
  console.log(`Tile caching proxy listening on http://localhost:${PORT}`);
  console.log(`Cache directory: ${CACHE_DIR}`);
  console.log(`Upstream: ${UPSTREAM}`);
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
