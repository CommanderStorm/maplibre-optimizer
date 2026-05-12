#!/usr/bin/env tsx
/**
 * Verify tile shaving effectiveness across multiple styles.
 *
 * For each OMT-compatible style, runs the full tile pruning pipeline
 * (optimize with --all + --stats + --advisory → advisory --format mvt)
 * and compares SUM(LENGTH(tile_data)) before vs after.
 *
 * Usage:
 *   npx tsx verify_tile_shave.ts
 *   npx tsx verify_tile_shave.ts --styles liberty,bright
 *   npx tsx verify_tile_shave.ts --mbtiles /path/to/tiles.mbtiles
 *   npx tsx verify_tile_shave.ts --out results/shave.jsonl
 */

import path from "node:path";
import fs from "node:fs";
import { execFileSync, execSync } from "node:child_process";
import Database from "better-sqlite3";

const __dirname = import.meta.dirname!;
const REPO_ROOT = path.resolve(__dirname, "../..");
const OPTIMIZER = path.join(REPO_ROOT, "target/release/maplibre-style-optimize");
const RESULTS_DIR = path.join(__dirname, "results");
const DEFAULT_MBTILES = path.join(REPO_ROOT, "data/output.mbtiles");

// ── styles (OMT-compatible only) ────────────────────────────────────────────

interface BenchStyle {
  id: string;
  url: string;
  cachePath: string;
}

const OMT_STYLES: BenchStyle[] = [
  { id: "liberty",         url: "https://tiles.openfreemap.org/styles/liberty",                                                    cachePath: path.join(RESULTS_DIR, "_cached_liberty.json") },
  { id: "bright",          url: "https://tiles.openfreemap.org/styles/bright",                                                     cachePath: path.join(RESULTS_DIR, "_cached_bright.json") },
  { id: "positron",        url: "https://tiles.openfreemap.org/styles/positron",                                                   cachePath: path.join(RESULTS_DIR, "_cached_positron.json") },
  { id: "fiord",           url: "https://tiles.openfreemap.org/styles/fiord",                                                      cachePath: path.join(RESULTS_DIR, "_cached_fiord.json") },
  { id: "dark-matter",     url: "https://cdn.jsdelivr.net/gh/openmaptiles/dark-matter-gl-style@v1.9/style.json",                   cachePath: path.join(RESULTS_DIR, "_cached_dark-matter.json") },
  { id: "osm-bright",      url: "https://cdn.jsdelivr.net/gh/openmaptiles/osm-bright-gl-style@v1.11/style.json",                   cachePath: path.join(RESULTS_DIR, "_cached_osm-bright.json") },
  { id: "klokan-basic",    url: "https://cdn.jsdelivr.net/gh/openmaptiles/klokantech-basic-gl-style@v1.10/style.json",             cachePath: path.join(RESULTS_DIR, "_cached_klokan-basic.json") },
  { id: "toner",           url: "https://cdn.jsdelivr.net/gh/openmaptiles/toner-gl-style@v1.0/style.json",                         cachePath: path.join(RESULTS_DIR, "_cached_toner.json") },
  { id: "osm-liberty",     url: "https://maputnik.github.io/osm-liberty/style.json",                                               cachePath: path.join(RESULTS_DIR, "_cached_osm-liberty.json") },
  { id: "americana",       url: "https://americanamap.org/style.json",                                                             cachePath: path.join(RESULTS_DIR, "_cached_americana.json") },
  { id: "stadia-outdoors", url: "https://tiles.stadiamaps.com/styles/outdoors.json",                                               cachePath: path.join(RESULTS_DIR, "_cached_stadia-outdoors.json") },
];

// ── CLI args ────────────────────────────────────────────────────────────────

const argv = process.argv.slice(2);
const stylesArg = argv.findIndex((a) => a === "--styles");
const mbtilesArg = argv.findIndex((a) => a === "--mbtiles");
const outArg = argv.findIndex((a) => a === "--out");

const styleFilter = stylesArg >= 0 ? argv[stylesArg + 1].split(",") : undefined;
const MBTILES = mbtilesArg >= 0 ? argv[mbtilesArg + 1] : DEFAULT_MBTILES;
const outPath = outArg >= 0
  ? argv[outArg + 1]
  : path.join(RESULTS_DIR, `verify-tile-shave-${new Date().toISOString().replace(/[:.]/g, "-")}.jsonl`);

const STYLES = styleFilter
  ? OMT_STYLES.filter((s) => styleFilter.includes(s.id))
  : OMT_STYLES;

// ── helpers ─────────────────────────────────────────────────────────────────

function buildOptimizer(): void {
  console.log("Building optimizer (release)…");
  execSync("cargo build --release --bin maplibre-style-optimize", {
    cwd: REPO_ROOT,
    stdio: "inherit",
  });
}

function totalTileDataSize(dbPath: string): { totalBytes: number; tileCount: number } {
  const db = new Database(dbPath, { readonly: true });
  const row = db.prepare("SELECT SUM(LENGTH(tile_data)) AS total, COUNT(*) AS cnt FROM tiles").get() as
    { total: number; cnt: number };
  db.close();
  return { totalBytes: row.total, tileCount: row.cnt };
}

async function fetchAndCacheStyle(style: BenchStyle): Promise<string> {
  if (fs.existsSync(style.cachePath)) {
    return fs.readFileSync(style.cachePath, "utf8");
  }
  console.log(`  Fetching ${style.id} from ${style.url}…`);
  const resp = await fetch(style.url);
  if (!resp.ok) throw new Error(`Failed to fetch ${style.id}: ${resp.status} ${resp.statusText}`);
  const text = await resp.text();
  JSON.parse(text); // validate JSON

  // Run gl-style-migrate to convert legacy property functions and filters
  // to expressions (same as run.ts) so the optimizer sees canonical syntax.
  const rawPath = style.cachePath + ".raw";
  fs.writeFileSync(rawPath, text);
  const migrated = execFileSync("gl-style-migrate", [rawPath], { encoding: "utf8", timeout: 10_000 });
  JSON.parse(migrated);
  fs.writeFileSync(style.cachePath, migrated);
  fs.unlinkSync(rawPath);
  console.log(`  Cached ${style.id} (${(text.length / 1024).toFixed(1)} KB raw → ${(migrated.length / 1024).toFixed(1)} KB migrated)`);
  return migrated;
}

function collectStats(mbtilesPath: string, outputPath: string): void {
  execFileSync(OPTIMIZER, [
    "stats", "--input", mbtilesPath, "--source-name", "openmaptiles", "--output", outputPath,
  ], { timeout: 300_000 });
}

function formatBytes(bytes: number): string {
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(2)} GB`;
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`;
  if (bytes >= 1e3) return `${(bytes / 1e3).toFixed(1)} KB`;
  return `${bytes} B`;
}

// ── main ────────────────────────────────────────────────────────────────────

interface ShaveResult {
  style_id: string;
  timestamp: string;
  original_bytes: number;
  pruned_bytes: number;
  reduction_bytes: number;
  reduction_pct: number;
  original_tile_count: number;
  pruned_tile_count: number;
  tiles_dropped: number;
}

async function main(): Promise<void> {
  buildOptimizer();
  fs.mkdirSync(RESULTS_DIR, { recursive: true });

  // Step 1: collect tile stats (once — style-independent)
  const statsPath = path.join(RESULTS_DIR, `_verify_stats_${process.pid}.json`);
  console.log(`Collecting tile stats from ${MBTILES}…`);
  collectStats(MBTILES, statsPath);
  console.log("  Stats collected.");

  // Step 2: query original tile data size
  console.log("Querying original tile data size…");
  const original = totalTileDataSize(MBTILES);
  console.log(`  Original: ${formatBytes(original.totalBytes)} across ${original.tileCount} tiles\n`);

  const timestamp = new Date().toISOString();
  const jsonlFd = fs.openSync(outPath, "w");
  const results: ShaveResult[] = [];

  const tmpStyleInput = path.join(RESULTS_DIR, `_verify_style_in_${process.pid}.json`);
  const tmpStyleOpt = path.join(RESULTS_DIR, `_verify_style_opt_${process.pid}.json`);
  const tmpAdvisory = path.join(RESULTS_DIR, `_verify_advisory_${process.pid}.json`);
  const tmpShaveDir = path.join(RESULTS_DIR, `_verify_shave_${process.pid}`);

  for (const style of STYLES) {
    console.log(`── ${style.id} ──`);

    // Fetch / cache the style
    const styleText = await fetchAndCacheStyle(style);
    fs.writeFileSync(tmpStyleInput, styleText);

    // Step 4a: optimize with --all --stats --advisory
    execFileSync(OPTIMIZER, [
      "optimize", "--input", tmpStyleInput, "--output", tmpStyleOpt,
      "--all",
      "--stats", statsPath,
      "--advisory", tmpAdvisory,
    ], { timeout: 60_000 });

    // Step 4b: advisory --format mvt
    console.log("  Running advisory (MVT)…");
    fs.mkdirSync(tmpShaveDir, { recursive: true });
    execFileSync(OPTIMIZER, [
      "advisory",
      "--advisory", tmpAdvisory,
      "--tiles", MBTILES,
      "--output", tmpShaveDir,
      "--format", "mvt",
    ], { timeout: 600_000 });

    // Step 4c: find pruned mbtiles and query size
    const shaveFiles = fs.readdirSync(tmpShaveDir);
    const prunedFile = shaveFiles.find((f) => f.endsWith(".mbtiles"))!;
    const prunedPath = path.join(tmpShaveDir, prunedFile);
    const pruned = totalTileDataSize(prunedPath);
    const dropped = original.tileCount - pruned.tileCount;
    const reductionBytes = original.totalBytes - pruned.totalBytes;
    const reductionPct = (reductionBytes / original.totalBytes) * 100;

    const result: ShaveResult = {
      style_id: style.id,
      timestamp,
      original_bytes: original.totalBytes,
      pruned_bytes: pruned.totalBytes,
      reduction_bytes: reductionBytes,
      reduction_pct: reductionPct,
      original_tile_count: original.tileCount,
      pruned_tile_count: pruned.tileCount,
      tiles_dropped: dropped,
    };
    results.push(result);
    fs.writeSync(jsonlFd, JSON.stringify(result) + "\n");

    console.log(`  ${formatBytes(original.totalBytes)} → ${formatBytes(pruned.totalBytes)} (${reductionPct.toFixed(1)}% reduction, ${dropped} tiles dropped)\n`);

    // Step 4d: clean up pruned mbtiles (~3 GB each)
    fs.rmSync(tmpShaveDir, { recursive: true, force: true });
    fs.unlinkSync(tmpAdvisory);
    fs.unlinkSync(tmpStyleOpt);
  }

  // Cleanup
  fs.unlinkSync(tmpStyleInput);
  fs.unlinkSync(statsPath);
  fs.closeSync(jsonlFd);

  // Print summary table
  console.log("═══════════════════════════════════════════════════════════════════════════════");
  console.log(
    "Style".padEnd(20) +
    "Original".padStart(12) +
    "Pruned".padStart(12) +
    "Reduction".padStart(12) +
    "  %" .padStart(8) +
    "Dropped".padStart(10),
  );
  console.log("─".repeat(74));
  for (const r of results) {
    console.log(
      r.style_id.padEnd(20) +
      formatBytes(r.original_bytes).padStart(12) +
      formatBytes(r.pruned_bytes).padStart(12) +
      formatBytes(r.reduction_bytes).padStart(12) +
      `${r.reduction_pct.toFixed(1)}%`.padStart(8) +
      String(r.tiles_dropped).padStart(10),
    );
  }
  console.log("═══════════════════════════════════════════════════════════════════════════════");

  console.log(`\nResults written to ${outPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
