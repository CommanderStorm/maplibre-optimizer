#!/usr/bin/env tsx
/**
 * Benchmark harness for maplibre-style-optimizer.
 *
 * Compares rendering performance of original vs optimized MapLibre GL styles
 * across diverse geographic locations and camera animations using headless Chrome.
 *
 * Usage:
 *   just bench                          # run all 18 scenarios, 5 runs each
 *   just bench --runs 1 munich-zigzag   # single quick scenario
 *   just bench-debug tokyo              # with browser console output
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

// ── CLI args ─────────────────────────────────────────────────────────────────

const argv = process.argv.slice(2);
const debug = argv.includes("--debug");
const runsArg = argv.findIndex((a) => a === "--runs");
const warmupArg = argv.findIndex((a) => a === "--warmup");
const RUNS = runsArg >= 0 ? parseInt(argv[runsArg + 1], 10) : 5;
const WARMUP = warmupArg >= 0 ? parseInt(argv[warmupArg + 1], 10) : 1;
const filters = argv.filter(
  (a, i) =>
    !a.startsWith("--") &&
    i !== runsArg + 1 &&
    i !== warmupArg + 1,
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

interface ScenarioResult {
  scenarioId: string;
  original: AggregatedMetrics;
  optimized: AggregatedMetrics;
  delta: Record<keyof RunMetrics, number>;
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
    styleSizeOriginal: number;
    styleSizeOptimized: number;
  };
  scenarios: ScenarioResult[];
  summary: {
    avgLoadDelta: number;
    avgIdleDelta: number;
    avgFpsDelta: number;
    avgP95FrameDelta: number;
  };
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

function optimizeStyle(styleJson: string): string {
  const tmpIn = path.join(RESULTS_DIR, "_bench_input.json");
  const tmpOut = path.join(RESULTS_DIR, "_bench_output.json");
  fs.writeFileSync(tmpIn, styleJson);
  try {
    execFileSync(OPTIMIZER, ["--input", tmpIn, "--output", tmpOut, "--all"], {
      timeout: 30_000,
    });
    return fs.readFileSync(tmpOut, "utf8");
  } finally {
    try { fs.unlinkSync(tmpIn); } catch {}
    try { fs.unlinkSync(tmpOut); } catch {}
  }
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

// ── run a single benchmark in the browser ────────────────────────────────────

async function runBenchmarkInBrowser(
  page: Page,
  styleJson: string,
  scenario: Scenario,
): Promise<RunMetrics> {
  await page.setViewport({ width: 1024, height: 768, deviceScaleFactor: 1 });
  const cssText = fs.readFileSync(MAPLIBRE_CSS, "utf8");
  await page.setContent(`<!DOCTYPE html>
<html><head><meta charset="utf-8">
<style>
body { margin: 0; }
#map { width: 1024px; height: 768px; }
${cssText}
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

function printTable(results: ScenarioResult[]): void {
  const header = [
    "Scenario".padEnd(26),
    "Load(ms)".padEnd(15),
    "Idle(ms)".padEnd(15),
    "FPS".padEnd(15),
    "Δ Load".padEnd(9),
    "Δ Idle".padEnd(9),
    "Δ FPS".padEnd(9),
    "Δ p95".padEnd(9),
  ].join("│ ");

  const sep = "─".repeat(header.length);

  console.log("\n" + sep);
  console.log(header);
  console.log(sep);

  for (const r of results) {
    const om = r.original.median;
    const optm = r.optimized.median;
    const d = r.delta;

    const row = [
      r.scenarioId.padEnd(26),
      `${om.loadMs.toFixed(0)} / ${optm.loadMs.toFixed(0)}`.padEnd(15),
      `${om.idleMs.toFixed(0)} / ${optm.idleMs.toFixed(0)}`.padEnd(15),
      `${om.fps.toFixed(1)} / ${optm.fps.toFixed(1)}`.padEnd(15),
      colorDelta(d.loadMs, true).padEnd(9),
      colorDelta(d.idleMs, true).padEnd(9),
      colorDelta(d.fps, false).padEnd(9),
      colorDelta(d.p95FrameMs, true).padEnd(9),
    ].join("│ ");
    console.log(row);
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

  // Fetch & optimize style
  const originalStyleJson = await fetchStyle();
  const originalSize = Buffer.byteLength(originalStyleJson, "utf8");
  console.log(`Original style: ${(originalSize / 1024).toFixed(1)} KB`);

  const optimizedStyleJson = optimizeStyle(originalStyleJson);
  const optimizedSize = Buffer.byteLength(optimizedStyleJson, "utf8");
  console.log(`Optimized style: ${(optimizedSize / 1024).toFixed(1)} KB (${((1 - optimizedSize / originalSize) * 100).toFixed(1)}% smaller)`);

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

  const results: ScenarioResult[] = [];

  for (let si = 0; si < scenarios.length; si++) {
    const scenario = scenarios[si];
    console.log(`[${si + 1}/${scenarios.length}] ${scenario.id} (${scenario.location.name}, ${scenario.animationType})`);

    const origRuns: RunMetrics[] = [];
    const optRuns: RunMetrics[] = [];
    const totalRuns = WARMUP + RUNS;

    // Interleaved: orig, opt, orig, opt, ...
    for (let run = 0; run < totalRuns; run++) {
      const isWarmup = run < WARMUP;
      const label = isWarmup ? `warmup ${run + 1}` : `run ${run - WARMUP + 1}/${RUNS}`;

      // Original run
      {
        const page = await browser.newPage();
        applyDebugListeners(page);
        try {
          const metrics = await runBenchmarkInBrowser(page, originalStyleJson, scenario);
          if (!isWarmup) origRuns.push(metrics);
          process.stdout.write(`  ${label} orig: load=${metrics.loadMs.toFixed(0)}ms fps=${metrics.fps.toFixed(1)}`);
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err);
          process.stdout.write(`  ${label} orig: ERROR ${msg}`);
          if (msg.includes("Session closed") || msg.includes("Target closed") || msg.includes("Protocol error")) {
            console.log("\n  Recovering browser…");
            try { await browser.close(); } catch {}
            browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });
          }
        } finally {
          try { await page.close(); } catch {}
        }
      }

      // Optimized run
      {
        const page = await browser.newPage();
        applyDebugListeners(page);
        try {
          const metrics = await runBenchmarkInBrowser(page, optimizedStyleJson, scenario);
          if (!isWarmup) optRuns.push(metrics);
          console.log(` | opt: load=${metrics.loadMs.toFixed(0)}ms fps=${metrics.fps.toFixed(1)}`);
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err);
          console.log(` | opt: ERROR ${msg}`);
          if (msg.includes("Session closed") || msg.includes("Target closed") || msg.includes("Protocol error")) {
            console.log("  Recovering browser…");
            try { await browser.close(); } catch {}
            browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });
          }
        } finally {
          try { await page.close(); } catch {}
        }
      }
    }

    if (origRuns.length > 0 && optRuns.length > 0) {
      const original = aggregate(origRuns);
      const optimized = aggregate(optRuns);
      const delta = computeDeltas(original, optimized);
      results.push({ scenarioId: scenario.id, original, optimized, delta });
    } else {
      console.log(`  Skipping ${scenario.id} — insufficient successful runs`);
    }
  }

  await browser.close();

  // ── output ──────────────────────────────────────────────────────────────────

  if (results.length === 0) {
    console.error("No results collected.");
    process.exit(1);
  }

  printTable(results);

  // Summary averages
  const avgLoadDelta = results.reduce((s, r) => s + r.delta.loadMs, 0) / results.length;
  const avgIdleDelta = results.reduce((s, r) => s + r.delta.idleMs, 0) / results.length;
  const avgFpsDelta = results.reduce((s, r) => s + r.delta.fps, 0) / results.length;
  const avgP95FrameDelta = results.reduce((s, r) => s + r.delta.p95FrameMs, 0) / results.length;

  console.log("\n── Summary ──");
  console.log(`  Avg load delta:  ${colorDelta(avgLoadDelta, true)}`);
  console.log(`  Avg idle delta:  ${colorDelta(avgIdleDelta, true)}`);
  console.log(`  Avg FPS delta:   ${colorDelta(avgFpsDelta, false)}`);
  console.log(`  Avg p95 frame delta: ${colorDelta(avgP95FrameDelta, true)}`);
  console.log(`  Style size: ${(originalSize / 1024).toFixed(1)} KB → ${(optimizedSize / 1024).toFixed(1)} KB (${((1 - optimizedSize / originalSize) * 100).toFixed(1)}% reduction)`);

  // Write JSON result
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const output: BenchResult = {
    meta: {
      timestamp,
      style: STYLE_URL,
      runsPerScenario: RUNS,
      warmupRuns: WARMUP,
      maplibreVersion: getMaplibreVersion(),
      renderer: "SwiftShader (headless Chrome)",
      optimizerFlags: "--all",
      styleSizeOriginal: originalSize,
      styleSizeOptimized: optimizedSize,
    },
    scenarios: results,
    summary: { avgLoadDelta, avgIdleDelta, avgFpsDelta, avgP95FrameDelta },
  };

  const outPath = path.join(RESULTS_DIR, `bench-${timestamp}.json`);
  fs.writeFileSync(outPath, JSON.stringify(output, null, 2));
  console.log(`\nResults written to ${outPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
