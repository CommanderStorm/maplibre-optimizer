#!/usr/bin/env tsx
/**
 * Benchmark harness for maplibre-style-optimizer.
 *
 * Compares rendering performance of original vs optimized MapLibre GL styles
 * across diverse geographic locations and camera animations using headless Chrome.
 *
 * Usage:
 *   just bench                                      # run all scenarios, 15 runs each (original + optimized)
 *   just bench --runs 1 munich-zigzag               # single quick scenario
 *   just bench --mbtiles /path/to/tiles.mbtiles     # enable stats & rewritten variants
 *   just bench-debug tokyo                          # with browser console output
 */

import path from "node:path";
import fs from "node:fs";
import { execFileSync, execSync } from "node:child_process";
import puppeteer, { type Browser, type Page } from "puppeteer";
import { getAllScenarios, filterScenarios, type Scenario, type Keyframe } from "./scenarios.js";

// ── paths ────────────────────────────────────────────────────────────────────

const __dirname = import.meta.dirname!;
const REPO_ROOT = path.resolve(__dirname, "../..");
const MAPLIBRE_JS = path.resolve(
  __dirname,
  "node_modules/maplibre-gl/dist/maplibre-gl-dev.js",
);
const MAPLIBRE_CSS = path.resolve(
  __dirname,
  "node_modules/maplibre-gl/dist/maplibre-gl.css",
);
const OPTIMIZER = path.join(REPO_ROOT, "target/release/maplibre-style-optimize");
const RESULTS_DIR = path.join(__dirname, "results");
const CACHED_STYLE = path.join(RESULTS_DIR, "_cached_style.json");
const STYLE_URL = "https://tiles.openfreemap.org/styles/liberty";

const TILE_PROXY_PORT = 8765;
const TILE_PROXY_URL = `http://localhost:${TILE_PROXY_PORT}`;

// ── CLI args ─────────────────────────────────────────────────────────────────

const argv = process.argv.slice(2);
const debug = argv.includes("--debug");
const runsArg = argv.findIndex((a) => a === "--runs");
const warmupArg = argv.findIndex((a) => a === "--warmup");
const mbtilesArg = argv.findIndex((a) => a === "--mbtiles");
const RUNS = runsArg >= 0 ? parseInt(argv[runsArg + 1], 10) : 15;
const WARMUP = warmupArg >= 0 ? parseInt(argv[warmupArg + 1], 10) : 1;
const MBTILES = mbtilesArg >= 0 ? argv[mbtilesArg + 1] : undefined;
const filters = argv.filter(
  (a, i) =>
    !a.startsWith("--") &&
    i !== runsArg + 1 &&
    i !== warmupArg + 1 &&
    i !== mbtilesArg + 1,
);

const RUN_TIMEOUT = 120_000;

// ── types ────────────────────────────────────────────────────────────────────

interface RunMetrics {
  loadMs: number;
  idleMs: number;
  fps: number;
  p50FrameMs: number;
  p95FrameMs: number;
  p99FrameMs: number;
  jankCount: number;
  animationMs: number;
}

interface AggregatedMetrics {
  runs: RunMetrics[];
  median: RunMetrics;
  stddev: RunMetrics;
}

interface Variant {
  id: string;
  label: string;
  styleJson: string;
}

interface ScenarioResult {
  scenarioId: string;
  variants: Record<string, AggregatedMetrics>;
  deltas: Record<string, Record<keyof RunMetrics, number>>;
  scenario: Scenario;
}

interface BenchResult {
  meta: {
    timestamp: string;
    style: string;
    runsPerScenario: number;
    warmupRuns: number;
    maplibreVersion: string;
    renderer: string;
    optimizerFlags: string;
    variants: { id: string; label: string; sizeBytes: number }[];
  };
  scenarios: ScenarioResult[];
  summary: Record<string, {
    avgLoadDelta: number;
    avgIdleDelta: number;
    avgFpsDelta: number;
    avgP95FrameDelta: number;
  }>;
}

// ── Puppeteer args (same SwiftShader setup as render tests) ──────────────────

const PUPPETEER_ARGS = [
  "--disable-gpu",
  "--enable-features=AllowSwiftShaderFallback,AllowSoftwareGLFallbackDueToCrashes",
  "--enable-unsafe-swiftshader",
];

// ── build optimizer ──────────────────────────────────────────────────────────

function buildOptimizer(): void {
  console.log("Building optimizer (release)…");
  execSync("cargo build --release --bin maplibre-style-optimize", {
    cwd: REPO_ROOT,
    stdio: "inherit",
  });
  if (!fs.existsSync(OPTIMIZER)) {
    throw new Error(`Optimizer binary not found at ${OPTIMIZER}`);
  }
}

// ── fetch and cache the OpenFreeMap liberty style ────────────────────────────

async function fetchStyle(): Promise<string> {
  if (fs.existsSync(CACHED_STYLE)) {
    console.log("Using cached style from", CACHED_STYLE);
    return fs.readFileSync(CACHED_STYLE, "utf8");
  }
  console.log(`Fetching style from ${STYLE_URL}…`);
  const resp = await fetch(STYLE_URL);
  if (!resp.ok) throw new Error(`Failed to fetch style: ${resp.status} ${resp.statusText}`);
  const text = await resp.text();
  // Validate it's JSON
  JSON.parse(text);
  fs.writeFileSync(CACHED_STYLE, text);
  console.log(`Cached style (${(text.length / 1024).toFixed(1)} KB)`);
  return text;
}

// ── optimize style via Rust binary ───────────────────────────────────────────

function optimizeStyle(styleJson: string, extraArgs: string[] = []): string {
  const tmpIn = path.join(RESULTS_DIR, "_bench_input.json");
  const tmpOut = path.join(RESULTS_DIR, "_bench_output.json");
  fs.writeFileSync(tmpIn, styleJson);
  try {
    execFileSync(OPTIMIZER, ["optimize", "--input", tmpIn, "--output", tmpOut, "--all", ...extraArgs], {
      timeout: 30_000,
    });
    return fs.readFileSync(tmpOut, "utf8");
  } finally {
    try { fs.unlinkSync(tmpIn); } catch {}
    try { fs.unlinkSync(tmpOut); } catch {}
  }
}

function collectStats(mbtilesPath: string, outputPath: string): void {
  execFileSync(OPTIMIZER, [
    "stats", "--input", mbtilesPath, "--source-name", "openmaptiles", "--output", outputPath,
  ], { timeout: 120_000 });
}

function optimizeWithStats(styleJson: string, statsPath: string, advisoryPath?: string): string {
  const args = ["--stats", statsPath];
  if (advisoryPath) args.push("--advisory", advisoryPath);
  return optimizeStyle(styleJson, args);
}

function applyAdvisory(advisoryPath: string, mbtilesPath: string, stylePath: string, outputDir: string): { style: string; tiles: string } | null {
  try {
    fs.mkdirSync(outputDir, { recursive: true });
    execFileSync(OPTIMIZER, [
      "advisory", "--advisory", advisoryPath, "--tiles", mbtilesPath, "--style", stylePath, "--output", outputDir,
    ], { timeout: 120_000 });
    // Read the rewritten style from the output directory
    const rewrittenStyle = path.join(outputDir, path.basename(stylePath));
    if (fs.existsSync(rewrittenStyle)) {
      return { style: fs.readFileSync(rewrittenStyle, "utf8"), tiles: path.join(outputDir, path.basename(mbtilesPath)) };
    }
    return null;
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    console.warn(`  Advisory command failed (may not be implemented yet): ${msg}`);
    return null;
  }
}

function formatKB(json: string): string {
  return `${(Buffer.byteLength(json, "utf8") / 1024).toFixed(1)} KB`;
}

function buildVariants(originalStyleJson: string): Variant[] {
  const variants: Variant[] = [
    { id: "original", label: "Original", styleJson: originalStyleJson },
  ];

  // Optimized (no stats)
  console.log("Optimizing style (no stats)…");
  const optimizedJson = optimizeStyle(originalStyleJson);
  variants.push({ id: "optimized", label: "Optimized", styleJson: optimizedJson });
  console.log(`  Optimized: ${formatKB(optimizedJson)}`);

  if (MBTILES) {
    // Collect stats
    const statsPath = path.join(RESULTS_DIR, "_bench_stats.json");
    console.log(`Collecting tile statistics from ${MBTILES}…`);
    collectStats(MBTILES, statsPath);

    // Optimized with stats
    console.log("Optimizing style (with stats)…");
    const advisoryPath = path.join(RESULTS_DIR, "_bench_advisory.json");
    const optimizedStatsJson = optimizeWithStats(originalStyleJson, statsPath, advisoryPath);
    variants.push({ id: "optimized-stats", label: "Optimized+Stats", styleJson: optimizedStatsJson });
    console.log(`  Optimized+Stats: ${formatKB(optimizedStatsJson)}`);

    // Advisory rewrite
    if (fs.existsSync(advisoryPath)) {
      console.log("Applying advisory rewrite…");
      const tmpStylePath = path.join(RESULTS_DIR, "_bench_stats_style.json");
      fs.writeFileSync(tmpStylePath, optimizedStatsJson);
      const advisoryOutputDir = path.join(RESULTS_DIR, "_bench_advisory_output");
      const result = applyAdvisory(advisoryPath, MBTILES, tmpStylePath, advisoryOutputDir);
      if (result) {
        variants.push({ id: "optimized-rewritten", label: "Optimized+Rewritten", styleJson: result.style });
        console.log(`  Optimized+Rewritten: ${formatKB(result.style)}`);
      } else {
        console.warn("  Skipping optimized-rewritten variant (advisory not available)");
      }
      try { fs.unlinkSync(tmpStylePath); } catch {}
    }

    try { fs.unlinkSync(statsPath); } catch {}
  }

  return variants;
}

// ── browser-side benchmark harness (plain JS, no tsx transforms) ─────────────

const BENCH_HARNESS = `
window.__runBenchmark = function(styleJSON, keyframes, locationCenter, locationZoom) {
  return new Promise(function(resolve, reject) {
    var timeout = setTimeout(function() {
      reject(new Error("benchmark run timeout"));
    }, ${RUN_TIMEOUT});

    try {
      var style = JSON.parse(styleJSON);
      var startTime = performance.now();
      var frameTimestamps = [];

      var map = new maplibregl.Map({
        container: "map",
        style: style,
        center: locationCenter,
        zoom: locationZoom,
        interactive: false,
        attributionControl: false,
        canvasContextAttributes: { preserveDrawingBuffer: true, powerPreference: "default" },
        fadeDuration: 0,
        maxCanvasSize: [8192, 8192],
      });

      map.on("render", function() {
        frameTimestamps.push(performance.now());
      });

      var loadMs, idleMs;

      map.once("load", function() {
        loadMs = performance.now() - startTime;

        map.once("idle", function() {
          idleMs = performance.now() - startTime;

          // Run camera animation keyframes sequentially
          var animStart = performance.now();
          var chain = Promise.resolve();

          keyframes.forEach(function(kf) {
            chain = chain.then(function() {
              return new Promise(function(res) {
                var opts = { duration: kf.duration, easing: function(t) { return t; }, essential: true };
                if (kf.center) opts.center = kf.center;
                if (kf.zoom !== undefined) opts.zoom = kf.zoom;
                if (kf.bearing !== undefined) opts.bearing = kf.bearing;
                if (kf.pitch !== undefined) opts.pitch = kf.pitch;
                map.easeTo(opts);
                map.once("moveend", function() { res(); });
              });
            });
          });

          chain.then(function() {
            // Wait for final idle after all animations
            map.once("idle", function() {
              var animationMs = performance.now() - animStart;
              clearTimeout(timeout);
              map.remove();
              resolve({
                loadMs: loadMs,
                idleMs: idleMs,
                animationMs: animationMs,
                frames: frameTimestamps,
              });
            });
          });
        });
      });
    } catch(e) {
      clearTimeout(timeout);
      reject(e);
    }
  });
};
`;

// ── tile proxy ──────────────────────────────────────────────────────────────

/**
 * Check that the local nginx caching proxy is running.
 * The proxy caches tiles from tiles.openfreemap.org on first access,
 * then serves them from disk — eliminating network variability.
 */
async function checkTileProxy(): Promise<void> {
  try {
    const resp = await fetch(`${TILE_PROXY_URL}/styles/liberty`);
    if (!resp.ok) throw new Error(`proxy returned ${resp.status}`);
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    console.error(`\nTile proxy is not running at ${TILE_PROXY_URL} — ${msg}`);
    console.error("Start it with: just bench-proxy");
    process.exit(1);
  }
  console.log(`Tile proxy OK at ${TILE_PROXY_URL}`);
}

/**
 * Rewrite a style JSON so all tile/sprite/glyph URLs point at the local
 * caching proxy instead of remote OpenFreeMap.
 */
function rewriteStyleForProxy(styleJson: string): string {
  return styleJson.replaceAll(
    "https://tiles.openfreemap.org",
    TILE_PROXY_URL,
  );
}

// ── run a single benchmark in the browser ────────────────────────────────────

// Read CSS once at module level — avoid re-reading the same file on every benchmark run
const MAPLIBRE_CSS_TEXT = fs.readFileSync(MAPLIBRE_CSS, "utf8");

async function runBenchmarkInBrowser(
  page: Page,
  styleJson: string,
  scenario: Scenario,
): Promise<RunMetrics> {
  await page.setViewport({ width: 1024, height: 768, deviceScaleFactor: 1 });
  await page.setContent(`<!DOCTYPE html>
<html><head><meta charset="utf-8">
<style>
body { margin: 0; }
#map { width: 1024px; height: 768px; }
${MAPLIBRE_CSS_TEXT}
</style>
</head><body><div id="map"></div></body></html>`);

  await page.addScriptTag({ path: MAPLIBRE_JS });
  await page.addScriptTag({ content: BENCH_HARNESS });

  const raw = await page.evaluate(
    (sJson, kfs, center, zoom) =>
      (window as any).__runBenchmark(sJson, kfs, center, zoom),
    styleJson,
    scenario.keyframes as any,
    scenario.location.center as any,
    scenario.location.zoom,
  ) as { loadMs: number; idleMs: number; animationMs: number; frames: number[] };

  return computeMetrics(raw);
}

// ── compute per-run metrics from raw frame data ──────────────────────────────

function computeMetrics(raw: {
  loadMs: number;
  idleMs: number;
  animationMs: number;
  frames: number[];
}): RunMetrics {
  const { loadMs, idleMs, animationMs, frames } = raw;

  // Compute frame deltas
  const deltas: number[] = [];
  for (let i = 1; i < frames.length; i++) {
    deltas.push(frames[i] - frames[i - 1]);
  }

  if (deltas.length === 0) {
    return {
      loadMs, idleMs, animationMs,
      fps: 0, p50FrameMs: 0, p95FrameMs: 0, p99FrameMs: 0, jankCount: 0,
    };
  }

  const sorted = [...deltas].sort((a, b) => a - b);
  const fps = deltas.length / (animationMs / 1000);
  const p50FrameMs = percentile(sorted, 0.5);
  const p95FrameMs = percentile(sorted, 0.95);
  const p99FrameMs = percentile(sorted, 0.99);

  // Jank: frames with delta > 2x median
  const median = p50FrameMs;
  const jankCount = deltas.filter((d) => d > 2 * median).length;

  return { loadMs, idleMs, fps, p50FrameMs, p95FrameMs, p99FrameMs, jankCount, animationMs };
}

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const idx = Math.max(0, Math.ceil(sorted.length * p) - 1);
  return sorted[idx];
}

// ── aggregation across runs ──────────────────────────────────────────────────

function aggregate(runs: RunMetrics[]): AggregatedMetrics {
  const keys: (keyof RunMetrics)[] = [
    "loadMs", "idleMs", "fps", "p50FrameMs", "p95FrameMs", "p99FrameMs", "jankCount", "animationMs",
  ];

  const medianMetrics = {} as RunMetrics;
  const stddevMetrics = {} as RunMetrics;

  for (const key of keys) {
    const vals = runs.map((r) => r[key]).sort((a, b) => a - b);
    medianMetrics[key] = percentile(vals, 0.5);

    const mean = vals.reduce((a, b) => a + b, 0) / vals.length;
    const variance = vals.reduce((sum, v) => sum + (v - mean) ** 2, 0) / vals.length;
    stddevMetrics[key] = Math.sqrt(variance);
  }

  return { runs, median: medianMetrics, stddev: stddevMetrics };
}

function computeDeltas(
  orig: AggregatedMetrics,
  opt: AggregatedMetrics,
): Record<keyof RunMetrics, number> {
  const keys: (keyof RunMetrics)[] = [
    "loadMs", "idleMs", "fps", "p50FrameMs", "p95FrameMs", "p99FrameMs", "jankCount", "animationMs",
  ];
  const delta = {} as Record<keyof RunMetrics, number>;
  for (const key of keys) {
    const origVal = orig.median[key];
    delta[key] = origVal === 0 ? 0 : ((opt.median[key] - origVal) / origVal) * 100;
  }
  return delta;
}

// ── console output formatting ────────────────────────────────────────────────

const TABLE_COLUMNS: { suffix: string; key: keyof RunMetrics; lowerIsBetter: boolean }[] = [
  { suffix: "ΔLoad", key: "loadMs", lowerIsBetter: true },
  { suffix: "ΔFPS", key: "fps", lowerIsBetter: false },
  { suffix: "Δp95", key: "p95FrameMs", lowerIsBetter: true },
];

function printTable(results: ScenarioResult[], nonOrigVariants: Variant[]): void {
  const headerParts = ["Scenario".padEnd(26)];
  for (const v of nonOrigVariants) {
    for (const col of TABLE_COLUMNS) {
      headerParts.push(`${v.label} ${col.suffix}`.padEnd(10));
    }
  }
  const header = headerParts.join("│ ");
  const sep = "─".repeat(header.length);

  console.log("\n" + sep);
  console.log(header);
  console.log(sep);

  for (const r of results) {
    const rowParts = [r.scenarioId.padEnd(26)];
    for (const v of nonOrigVariants) {
      const d = r.deltas[v.id];
      for (const col of TABLE_COLUMNS) {
        rowParts.push(d ? colorDelta(d[col.key], col.lowerIsBetter).padEnd(10) : "n/a".padEnd(10));
      }
    }
    console.log(rowParts.join("│ "));
  }
  console.log(sep);
}

/** Color a delta percentage. For "lower is better" metrics, negative is green. */
function colorDelta(pct: number, lowerIsBetter: boolean): string {
  const str = `${pct >= 0 ? "+" : ""}${pct.toFixed(1)}%`;
  const isGood = lowerIsBetter ? pct < 0 : pct > 0;
  if (Math.abs(pct) < 0.5) return str; // neutral
  return isGood ? `\x1b[32m${str}\x1b[0m` : `\x1b[31m${str}\x1b[0m`;
}

// ── get MapLibre version from package.json ───────────────────────────────────

function getMaplibreVersion(): string {
  try {
    const pkg = JSON.parse(
      fs.readFileSync(path.join(__dirname, "node_modules/maplibre-gl/package.json"), "utf8"),
    );
    return pkg.version || "unknown";
  } catch {
    return "unknown";
  }
}

// ── main ─────────────────────────────────────────────────────────────────────

async function main(): Promise<void> {
  buildOptimizer();
  fs.mkdirSync(RESULTS_DIR, { recursive: true });

  // Check tile proxy is running
  await checkTileProxy();

  // Fetch style and rewrite URLs to use local caching proxy
  const remoteStyleJson = await fetchStyle();
  const originalStyleJson = rewriteStyleForProxy(remoteStyleJson);
  const originalSize = Buffer.byteLength(originalStyleJson, "utf8");
  console.log(`Original style: ${(originalSize / 1024).toFixed(1)} KB (URLs rewritten to local proxy)\n`);

  // Build all variants (optimize, stats, advisory — all outside the bench loop)
  const variants = buildVariants(originalStyleJson);
  console.log(`\nActive variants: ${variants.map((v) => v.id).join(", ")}`);

  // Resolve scenarios
  const allScenarios = getAllScenarios();
  const scenarios = filterScenarios(allScenarios, filters);
  if (scenarios.length === 0) {
    console.error("No scenarios matched filters:", filters);
    process.exit(1);
  }
  console.log(`\nRunning ${scenarios.length} scenarios, ${RUNS} runs each (${WARMUP} warmup)\n`);

  // Launch browser
  let browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });

  function applyDebugListeners(p: Page): void {
    if (!debug) return;
    p.on("console", (msg) => console.log(`  [browser] ${msg.text()}`));
    p.on("pageerror", (err) => console.error(`  [browser error] ${err.message}`));
  }

  // Open JSONL file for streaming — append each run as it completes
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const jsonlPath = path.join(RESULTS_DIR, `bench-${timestamp}.jsonl`);
  const jsonlFd = fs.openSync(jsonlPath, "w");
  console.log(`Streaming results to ${jsonlPath}\n`);

  const results: ScenarioResult[] = [];

  for (let si = 0; si < scenarios.length; si++) {
    const scenario = scenarios[si];
    console.log(`[${si + 1}/${scenarios.length}] ${scenario.id} (${scenario.location.name}, ${scenario.animationType})`);

    const variantRuns: Record<string, RunMetrics[]> = {};
    for (const v of variants) variantRuns[v.id] = [];
    const totalRuns = WARMUP + RUNS;

    // Interleaved: run all variants in sequence per iteration
    for (let run = 0; run < totalRuns; run++) {
      const isWarmup = run < WARMUP;
      const label = isWarmup ? `warmup ${run + 1}` : `run ${run - WARMUP + 1}/${RUNS}`;

      const jsonlBase = {
        scenario: scenario.id,
        location: scenario.location.name,
        lng: scenario.location.center[0],
        lat: scenario.location.center[1],
        zoom: scenario.location.zoom,
        animation: scenario.animationType,
        timestamp,
      };

      const parts: string[] = [];

      for (let vi = 0; vi < variants.length; vi++) {
        const variant = variants[vi];
        const page = await browser.newPage();
        applyDebugListeners(page);

        try {
          const metrics = await runBenchmarkInBrowser(page, variant.styleJson, scenario);
          if (!isWarmup) {
            variantRuns[variant.id].push(metrics);
            fs.writeSync(jsonlFd, JSON.stringify({ ...jsonlBase, variant: variant.id, run: run - WARMUP + 1, ...metrics }) + "\n");
          }
          parts.push(`${variant.id}: load=${metrics.loadMs.toFixed(0)}ms fps=${metrics.fps.toFixed(1)}`);
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err);
          parts.push(`${variant.id}: ERROR ${msg}`);
          if (msg.includes("Session closed") || msg.includes("Target closed") || msg.includes("Protocol error")) {
            console.log("\n  Recovering browser…");
            try { await browser.close(); } catch {}
            browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });
          }
        } finally {
          try { await page.close(); } catch {}
        }
      }

      console.log(`  ${label} ${parts.join(" | ")}`);
    }

    // Aggregate results — require at least original to have runs
    if (variantRuns["original"].length === 0) {
      console.log(`  Skipping ${scenario.id} — no successful original runs`);
      continue;
    }

    const aggregated: Record<string, AggregatedMetrics> = {};
    for (const v of variants) {
      if (variantRuns[v.id].length > 0) {
        aggregated[v.id] = aggregate(variantRuns[v.id]);
      }
    }

    const deltas: Record<string, Record<keyof RunMetrics, number>> = {};
    for (const v of variants) {
      if (v.id !== "original" && aggregated[v.id]) {
        deltas[v.id] = computeDeltas(aggregated["original"], aggregated[v.id]);
      }
    }

    results.push({ scenarioId: scenario.id, variants: aggregated, deltas, scenario });
  }

  await browser.close();

  // ── output ──────────────────────────────────────────────────────────────────

  if (results.length === 0) {
    console.error("No results collected.");
    process.exit(1);
  }

  const nonOrigVariants = variants.filter((v) => v.id !== "original");
  printTable(results, nonOrigVariants);
  const summary: BenchResult["summary"] = {};

  console.log("\n── Summary ──");
  for (const v of nonOrigVariants) {
    const withDelta = results.filter((r) => r.deltas[v.id]);
    if (withDelta.length === 0) continue;
    const avgLoadDelta = withDelta.reduce((s, r) => s + r.deltas[v.id].loadMs, 0) / withDelta.length;
    const avgIdleDelta = withDelta.reduce((s, r) => s + r.deltas[v.id].idleMs, 0) / withDelta.length;
    const avgFpsDelta = withDelta.reduce((s, r) => s + r.deltas[v.id].fps, 0) / withDelta.length;
    const avgP95FrameDelta = withDelta.reduce((s, r) => s + r.deltas[v.id].p95FrameMs, 0) / withDelta.length;
    summary[v.id] = { avgLoadDelta, avgIdleDelta, avgFpsDelta, avgP95FrameDelta };

    console.log(`  [${v.label}]`);
    console.log(`    Avg load delta:  ${colorDelta(avgLoadDelta, true)}`);
    console.log(`    Avg idle delta:  ${colorDelta(avgIdleDelta, true)}`);
    console.log(`    Avg FPS delta:   ${colorDelta(avgFpsDelta, false)}`);
    console.log(`    Avg p95 frame delta: ${colorDelta(avgP95FrameDelta, true)}`);
  }

  // Style sizes
  console.log("\n  Style sizes:");
  for (const v of variants) {
    const pct = v.id === "original"
      ? ""
      : ` (${((1 - Buffer.byteLength(v.styleJson, "utf8") / originalSize) * 100).toFixed(1)}% smaller)`;
    console.log(`    ${v.label}: ${formatKB(v.styleJson)}${pct}`);
  }

  fs.closeSync(jsonlFd);

  // Write JSON result
  const output: BenchResult = {
    meta: {
      timestamp,
      style: STYLE_URL,
      runsPerScenario: RUNS,
      warmupRuns: WARMUP,
      maplibreVersion: getMaplibreVersion(),
      renderer: "SwiftShader (headless Chrome)",
      optimizerFlags: "--all",
      variants: variants.map((v) => ({
        id: v.id,
        label: v.label,
        sizeBytes: Buffer.byteLength(v.styleJson, "utf8"),
      })),
    },
    scenarios: results,
    summary,
  };

  const outPath = path.join(RESULTS_DIR, `bench-${timestamp}.json`);
  fs.writeFileSync(outPath, JSON.stringify(output, null, 2));
  console.log(`\nResults written to ${outPath}`);
  console.log(`JSONL written to ${jsonlPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
