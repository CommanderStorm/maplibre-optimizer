#!/usr/bin/env tsx
/**
 * Cross-style generalization benchmark.
 *
 * Fetches open-source MapLibre styles from the Maputnik catalog, runs the
 * optimizer on each, and records size/complexity metrics. No rendering needed —
 * this is a pure static analysis benchmark.
 *
 * Usage:
 *   npx tsx cross_style.ts
 *   npx tsx cross_style.ts --out results/cross-style.jsonl
 */

import path from "node:path";
import fs from "node:fs";
import { execFileSync, execSync } from "node:child_process";

const __dirname = import.meta.dirname!;
const REPO_ROOT = path.resolve(__dirname, "../..");
const OPTIMIZER = path.join(REPO_ROOT, "target/release/maplibre-style-optimize");
const RESULTS_DIR = path.join(__dirname, "results");
const CACHE_DIR = path.join(RESULTS_DIR, "_catalog_cache");
const CATALOG_URL = "https://raw.githubusercontent.com/maplibre/maputnik/main/src/config/styles.json";

// ── CLI args ─────────────────────────────────────────��───────────────────────

const argv = process.argv.slice(2);
const outArg = argv.findIndex((a) => a === "--out");
const outPath = outArg >= 0
  ? argv[outArg + 1]
  : path.join(RESULTS_DIR, `cross-style-${new Date().toISOString().replace(/[:.]/g, "-")}.jsonl`);

// ── build optimizer ──────────────────────────────────────────────────────────

function buildOptimizer(): void {
  console.log("Building optimizer (release)…");
  execSync("cargo build --release --bin maplibre-style-optimize", {
    cwd: REPO_ROOT,
    stdio: "inherit",
  });
}

// ── fetch catalog ──────────────────────────��─────────────────────────────────

interface CatalogEntry {
  id: string;
  url: string;
  title?: string;
}

async function fetchCatalog(): Promise<CatalogEntry[]> {
  console.log(`Fetching catalog from ${CATALOG_URL}…`);
  const resp = await fetch(CATALOG_URL);
  if (!resp.ok) throw new Error(`Failed to fetch catalog: ${resp.status}`);
  const data = await resp.json() as CatalogEntry[];
  console.log(`  ${data.length} styles in catalog`);
  return data;
}

async function fetchAndCacheStyle(entry: CatalogEntry): Promise<string | null> {
  const cacheFile = path.join(CACHE_DIR, `${entry.id}.json`);
  if (fs.existsSync(cacheFile)) {
    return fs.readFileSync(cacheFile, "utf8");
  }
  try {
    const resp = await fetch(entry.url);
    if (!resp.ok) {
      console.log(`  SKIP ${entry.id}: HTTP ${resp.status}`);
      return null;
    }
    const text = await resp.text();
    // Validate it's JSON
    JSON.parse(text);
    fs.writeFileSync(cacheFile, text);
    return text;
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    console.log(`  SKIP ${entry.id}: ${msg}`);
    return null;
  }
}

// ── optimize + complexity ───────────────────��────────────────────────────────

interface ComplexityReport {
  ast_nodes: number;
  max_depth: number;
  layer_count: number;
  filter_count: number;
  expression_types: Record<string, number>;
}

function getComplexity(stylePath: string): ComplexityReport | null {
  try {
    const output = execFileSync(OPTIMIZER, ["complexity", "--input", stylePath], {
      timeout: 10_000,
      encoding: "utf8",
    });
    return JSON.parse(output) as ComplexityReport;
  } catch {
    return null;
  }
}

function tryOptimize(inputPath: string, outputPath: string): boolean {
  try {
    // Use --all minus --selectivity-reorder (no tile data available)
    execFileSync(OPTIMIZER, [
      "optimize", "--input", inputPath, "--output", outputPath,
      "--simplify-unary", "--expression-kind", "--constant-fold",
      "--simplify-expressions", "--strip-defaults", "--minify-colors",
      "--strip-metadata", "--dead-elimination", "--metadata-refinement",
      "--cleanup", "--layer-merge",
    ], { timeout: 30_000 });
    return true;
  } catch {
    return false;
  }
}

function gzipSize(data: string): number {
  const zlib = require("node:zlib");
  return zlib.gzipSync(Buffer.from(data, "utf8"), { level: 9 }).length;
}

function brotliSize(data: string): number {
  const zlib = require("node:zlib");
  return zlib.brotliCompressSync(Buffer.from(data, "utf8"), {
    params: { [zlib.constants.BROTLI_PARAM_QUALITY]: zlib.constants.BROTLI_MAX_QUALITY },
  }).length;
}

// ── main ─────��────────────────────────���────────────────────────���─────────────

async function main(): Promise<void> {
  buildOptimizer();
  fs.mkdirSync(RESULTS_DIR, { recursive: true });
  fs.mkdirSync(CACHE_DIR, { recursive: true });

  const catalog = await fetchCatalog();
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const jsonlFd = fs.openSync(outPath, "w");

  let successes = 0;
  let skips = 0;
  let failures = 0;

  const tmpInput = path.join(RESULTS_DIR, "_cross_input.json");
  const tmpOutput = path.join(RESULTS_DIR, "_cross_output.json");

  for (const entry of catalog) {
    const styleText = await fetchAndCacheStyle(entry);
    if (!styleText) {
      skips++;
      continue;
    }

    fs.writeFileSync(tmpInput, styleText);
    const originalBytes = Buffer.byteLength(styleText, "utf8");
    const originalGzip = gzipSize(styleText);
    const originalBrotli = brotliSize(styleText);
    const originalComplexity = getComplexity(tmpInput);

    if (!tryOptimize(tmpInput, tmpOutput)) {
      console.log(`  FAIL ${entry.id}: optimizer error`);
      failures++;
      continue;
    }

    const optimizedText = fs.readFileSync(tmpOutput, "utf8");
    const optimizedBytes = Buffer.byteLength(optimizedText, "utf8");
    const optimizedGzip = gzipSize(optimizedText);
    const optimizedBrotli = brotliSize(optimizedText);

    fs.writeFileSync(tmpInput, optimizedText);
    const optimizedComplexity = getComplexity(tmpInput);

    const record: Record<string, unknown> = {
      style_id: entry.id,
      style_title: entry.title ?? entry.id,
      timestamp,
      original_bytes: originalBytes,
      optimized_bytes: optimizedBytes,
      original_gzip_bytes: originalGzip,
      optimized_gzip_bytes: optimizedGzip,
      original_brotli_bytes: originalBrotli,
      optimized_brotli_bytes: optimizedBrotli,
      reduction_pct: ((1 - optimizedBytes / originalBytes) * 100),
      gzip_reduction_pct: ((1 - optimizedGzip / originalGzip) * 100),
      brotli_reduction_pct: ((1 - optimizedBrotli / originalBrotli) * 100),
    };

    if (originalComplexity) {
      record.original_ast_nodes = originalComplexity.ast_nodes;
      record.original_max_depth = originalComplexity.max_depth;
      record.original_layer_count = originalComplexity.layer_count;
      record.original_filter_count = originalComplexity.filter_count;
    }
    if (optimizedComplexity) {
      record.optimized_ast_nodes = optimizedComplexity.ast_nodes;
      record.optimized_max_depth = optimizedComplexity.max_depth;
      record.optimized_layer_count = optimizedComplexity.layer_count;
      record.optimized_filter_count = optimizedComplexity.filter_count;
    }

    fs.writeSync(jsonlFd, JSON.stringify(record) + "\n");
    const pct = record.reduction_pct as number;
    console.log(`  OK ${entry.id}: ${(originalBytes / 1024).toFixed(1)} KB → ${(optimizedBytes / 1024).toFixed(1)} KB (${pct.toFixed(1)}% smaller)`);
    successes++;
  }

  // Cleanup
  try { fs.unlinkSync(tmpInput); } catch {}
  try { fs.unlinkSync(tmpOutput); } catch {}
  fs.closeSync(jsonlFd);

  console.log(`\nDone: ${successes} OK, ${skips} skipped, ${failures} failed`);
  console.log(`Results written to ${outPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
