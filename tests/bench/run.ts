#!/usr/bin/env tsx
/**
 * Benchmark harness for maplibre-style-optimizer — cumulative ablation.
 *
 * Runs 12 ablation steps (baseline + 11 passes added one at a time) across
 * all scenarios.  Each step enables one additional optimizer pass on top of
 * all previous ones, showing the marginal contribution of each pass.
 *
 * Usage:
 *   just bench                                      # all scenarios, 15 runs, 11 ablation steps
 *   just bench --runs 1 munich-zigzag               # single quick scenario
 *   just bench --mbtiles /path/to/tiles.mbtiles     # enable step 12 (selectivity_reorder)
 *   just bench-debug tokyo                          # with browser console output
 */

import path from "node:path";
import fs from "node:fs";
import { execFileSync, execSync } from "node:child_process";
import puppeteer, { type Page } from "puppeteer";
import { getAllScenarios, filterScenarios, type Scenario } from "./scenarios.js";

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
  passes: string[];
  styleJson: string;
  styleBytes: number;
}

interface ScenarioResult {
  scenarioId: string;
  variants: Record<string, AggregatedMetrics>;
}

// ── ablation sequence ────────────────────────────────────────────────────────

/**
 * Cumulative ablation: each step adds one pass on top of all previous ones.
 * The `flag` is the CLI flag name (with hyphens) passed to the optimizer.
 * The `pass` is the human-readable pass name (with underscores).
 */
const ABLATION_STEPS: { pass: string; flag: string }[] = [
  { pass: "simplify_unary",      flag: "--simplify-unary" },
  { pass: "expression_kind",     flag: "--expression-kind" },
  { pass: "constant_fold",       flag: "--constant-fold" },
  { pass: "simplify_expressions", flag: "--simplify-expressions" },
  { pass: "strip_defaults",      flag: "--strip-defaults" },
  { pass: "minify_colors",       flag: "--minify-colors" },
  { pass: "strip_metadata",      flag: "--strip-metadata" },
  { pass: "dead_elimination",    flag: "--dead-elimination" },
  { pass: "metadata_refinement", flag: "--metadata-refinement" },
  { pass: "cleanup",             flag: "--cleanup" },
  // Step 12: only when --mbtiles is provided
  { pass: "selectivity_reorder", flag: "--selectivity-reorder" },
];

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
  JSON.parse(text);
  fs.writeFileSync(CACHED_STYLE, text);
  console.log(`Cached style (${(text.length / 1024).toFixed(1)} KB)`);
  return text;
}

// ── optimize style via Rust binary ───────────────────────────────────────────

function optimizeStyle(inputPath: string, passFlags: string[], extraArgs: string[] = []): string {
  const tmpOut = path.join(RESULTS_DIR, "_bench_output.json");
  try {
    execFileSync(OPTIMIZER, ["optimize", "--input", inputPath, "--output", tmpOut, ...passFlags, ...extraArgs], {
      timeout: 30_000,
    });
    return fs.readFileSync(tmpOut, "utf8");
  } finally {
    try { fs.unlinkSync(tmpOut); } catch {}
  }
}

function collectStats(mbtilesPath: string, outputPath: string): void {
  execFileSync(OPTIMIZER, [
    "stats", "--input", mbtilesPath, "--source-name", "openmaptiles", "--output", outputPath,
  ]);
}

function formatKB(json: string): string {
  return `${(Buffer.byteLength(json, "utf8") / 1024).toFixed(1)} KB`;
}

/**
 * Build the cumulative ablation variants.
 *
 * Returns an array of Variant objects: baseline (no passes) followed by one
 * variant per ablation step, each accumulating all passes from previous steps.
 * Step 12 (selectivity_reorder) is only included when --mbtiles is provided.
 */
function buildVariants(originalStyleJson: string): Variant[] {
  const variants: Variant[] = [];

  // Step 0: baseline (no optimization)
  const baselineBytes = Buffer.byteLength(originalStyleJson, "utf8");
  variants.push({
    id: "step-00-baseline",
    label: "baseline",
    passes: [],
    styleJson: originalStyleJson,
    styleBytes: baselineBytes,
  });
  console.log(`  step-00-baseline: ${formatKB(originalStyleJson)}`);

  // Collect stats if mbtiles provided (needed for selectivity_reorder)
  let statsPath: string | undefined;
  if (MBTILES) {
    statsPath = path.join(RESULTS_DIR, "_bench_stats.json");
    console.log(`Collecting tile statistics from ${MBTILES}…`);
    collectStats(MBTILES, statsPath);
  }

  // Determine how many steps to run
  const maxSteps = MBTILES ? ABLATION_STEPS.length : ABLATION_STEPS.length - 1;

  // Write input style once for all optimizer invocations
  const inputPath = path.join(RESULTS_DIR, "_bench_input.json");
  fs.writeFileSync(inputPath, originalStyleJson);

  // Build cumulative pass flags
  const accumulatedFlags: string[] = [];
  const accumulatedPasses: string[] = [];

  try {
    for (let i = 0; i < maxSteps; i++) {
      const step = ABLATION_STEPS[i];
      accumulatedFlags.push(step.flag);
      accumulatedPasses.push(step.pass);

      const stepNum = String(i + 1).padStart(2, "0");
      const variantId = `step-${stepNum}-${step.pass}`;

      // Build extra args for selectivity_reorder
      const extraArgs: string[] = [];
      if (step.pass === "selectivity_reorder" && statsPath) {
        extraArgs.push("--stats", statsPath);
      }

      const optimizedJson = optimizeStyle(inputPath, accumulatedFlags, extraArgs);
      const styleBytes = Buffer.byteLength(optimizedJson, "utf8");
      const pctSmaller = ((1 - styleBytes / baselineBytes) * 100).toFixed(1);

      variants.push({
        id: variantId,
        label: `+${step.pass}`,
        passes: [...accumulatedPasses],
        styleJson: optimizedJson,
        styleBytes,
      });
      console.log(`  ${variantId}: ${formatKB(optimizedJson)} (${pctSmaller}% smaller)`);
    }
  } finally {
    try { fs.unlinkSync(inputPath); } catch {}
    if (statsPath) {
      try { fs.unlinkSync(statsPath); } catch {}
    }
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

function rewriteStyleForProxy(styleJson: string): string {
  return styleJson.replaceAll(
    "https://tiles.openfreemap.org",
    TILE_PROXY_URL,
  );
}

// ── run a single benchmark in the browser ────────────────────────────────────

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

// ── console output formatting ────────────────────────────────────────────────

/** Format a delta percentage, optionally with ANSI color. Pad to `width` visible chars. */
function colorDelta(pct: number, lowerIsBetter: boolean, width = 0): string {
  const raw = `${pct >= 0 ? "+" : ""}${pct.toFixed(1)}%`;
  const padded = width > 0 ? raw.padStart(width) : raw;
  const isGood = lowerIsBetter ? pct < 0 : pct > 0;
  if (Math.abs(pct) < 0.5) return padded;
  return isGood ? `\x1b[32m${padded}\x1b[0m` : `\x1b[31m${padded}\x1b[0m`;
}

function printSummaryTable(results: ScenarioResult[], variants: Variant[]): void {
  // Show a condensed table: scenario vs baseline load/p95, then final step's delta
  const baseline = variants[0];
  const final = variants[variants.length - 1];

  const header = [
    "Scenario".padEnd(26),
    "Base Load".padStart(10),
    "Base p95".padStart(10),
    "Final Load".padStart(11),
    "Final p95".padStart(10),
    "ΔLoad".padStart(8),
    "Δp95".padStart(8),
  ].join(" │ ");
  const sep = "─".repeat(header.length);

  console.log("\n" + sep);
  console.log(header);
  console.log(sep);

  for (const r of results) {
    const baseAgg = r.variants[baseline.id];
    const finalAgg = r.variants[final.id];
    if (!baseAgg || !finalAgg) continue;

    const bLoad = baseAgg.median.loadMs.toFixed(0) + "ms";
    const bP95 = baseAgg.median.p95FrameMs.toFixed(1) + "ms";
    const fLoad = finalAgg.median.loadMs.toFixed(0) + "ms";
    const fP95 = finalAgg.median.p95FrameMs.toFixed(1) + "ms";

    const dLoad = baseAgg.median.loadMs === 0 ? 0 :
      ((finalAgg.median.loadMs - baseAgg.median.loadMs) / baseAgg.median.loadMs) * 100;
    const dP95 = baseAgg.median.p95FrameMs === 0 ? 0 :
      ((finalAgg.median.p95FrameMs - baseAgg.median.p95FrameMs) / baseAgg.median.p95FrameMs) * 100;

    console.log([
      r.scenarioId.padEnd(26),
      bLoad.padStart(10),
      bP95.padStart(10),
      fLoad.padStart(11),
      fP95.padStart(10),
      colorDelta(dLoad, true, 8),
      colorDelta(dP95, true, 8),
    ].join(" │ "));
  }
  console.log(sep);
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

  await checkTileProxy();

  const remoteStyleJson = await fetchStyle();
  const originalStyleJson = rewriteStyleForProxy(remoteStyleJson);
  const originalSize = Buffer.byteLength(originalStyleJson, "utf8");
  console.log(`Original style: ${(originalSize / 1024).toFixed(1)} KB (URLs rewritten to local proxy)\n`);

  // Build all ablation variants
  console.log("Building ablation variants…");
  const variants = buildVariants(originalStyleJson);
  console.log(`\n${variants.length} ablation steps: ${variants.map((v) => v.id).join(", ")}`);

  // Resolve scenarios
  const allScenarios = getAllScenarios();
  const scenarios = filterScenarios(allScenarios, filters);
  if (scenarios.length === 0) {
    console.error("No scenarios matched filters:", filters);
    process.exit(1);
  }
  console.log(`\nRunning ${scenarios.length} scenarios × ${variants.length} variants, ${RUNS} runs each (${WARMUP} warmup)\n`);

  let browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });

  function applyDebugListeners(p: Page): void {
    if (!debug) return;
    p.on("console", (msg) => console.log(`  [browser] ${msg.text()}`));
    p.on("pageerror", (err) => console.error(`  [browser error] ${err.message}`));
  }

  // Open JSONL file for streaming
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

      const parts: string[] = [];

      for (let vi = 0; vi < variants.length; vi++) {
        const variant = variants[vi];
        const page = await browser.newPage();
        applyDebugListeners(page);

        try {
          const metrics = await runBenchmarkInBrowser(page, variant.styleJson, scenario);
          if (!isWarmup) {
            variantRuns[variant.id].push(metrics);
            const record = {
              scenario: scenario.id,
              location: scenario.location.name,
              lng: scenario.location.center[0],
              lat: scenario.location.center[1],
              zoom: scenario.location.zoom,
              animation: scenario.animationType,
              variant: variant.id,
              passes: variant.passes,
              style_bytes: variant.styleBytes,
              run: run - WARMUP + 1,
              timestamp,
              ...metrics,
            };
            fs.writeSync(jsonlFd, JSON.stringify(record) + "\n");
          }
          parts.push(`${variant.id.replace("step-", "s")}: ${metrics.loadMs.toFixed(0)}ms`);
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err);
          parts.push(`${variant.id}: ERR`);
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

    // Aggregate results
    if (variantRuns[variants[0].id].length === 0) {
      console.log(`  Skipping ${scenario.id} — no successful baseline runs`);
      continue;
    }

    const aggregated: Record<string, AggregatedMetrics> = {};
    for (const v of variants) {
      if (variantRuns[v.id].length > 0) {
        aggregated[v.id] = aggregate(variantRuns[v.id]);
      }
    }

    results.push({ scenarioId: scenario.id, variants: aggregated });
  }

  await browser.close();

  // ── output ──────────────────────────────────────────────────────────────────

  if (results.length === 0) {
    console.error("No results collected.");
    process.exit(1);
  }

  printSummaryTable(results, variants);

  // Style sizes
  console.log("\n── Style Sizes (ablation) ──");
  for (const v of variants) {
    const kb = (v.styleBytes / 1024).toFixed(1);
    const pct = v.id.includes("baseline")
      ? ""
      : ` (${((1 - v.styleBytes / variants[0].styleBytes) * 100).toFixed(1)}% smaller)`;
    console.log(`  ${v.label}: ${kb} KB${pct}`);
  }

  fs.closeSync(jsonlFd);

  // Write JSON metadata
  const meta = {
    timestamp,
    style: STYLE_URL,
    runsPerScenario: RUNS,
    warmupRuns: WARMUP,
    maplibreVersion: getMaplibreVersion(),
    renderer: "SwiftShader (headless Chrome)",
    ablationSteps: variants.map((v) => ({
      id: v.id,
      label: v.label,
      passes: v.passes,
      styleBytes: v.styleBytes,
    })),
  };

  const metaPath = path.join(RESULTS_DIR, `bench-${timestamp}.meta.json`);
  fs.writeFileSync(metaPath, JSON.stringify(meta, null, 2));
  console.log(`\nMetadata written to ${metaPath}`);
  console.log(`JSONL written to ${jsonlPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
