#!/usr/bin/env tsx
/**
 * Benchmark harness for maplibre-style-optimizer — cumulative ablation.
 *
 * Runs up to 17 ablation steps (baseline + 16 passes added one at a time)
 * across all scenarios.  Each step enables one additional optimizer pass on
 * top of all previous ones, showing the marginal contribution of each pass.
 *
 * Steps 16 (selectivity_reorder) and 17 (tile_rewrite) require --mbtiles.
 * The tile_rewrite step runs the advisory pipeline to produce MLT-encoded
 * tiles and a rewritten style, then serves those tiles via the proxy.
 *
 * Usage:
 *   just bench                                      # all scenarios, 15 runs, 15 ablation steps
 *   just bench --runs 1 munich-zigzag               # single quick scenario
 *   just bench --mbtiles /path/to/tiles.mbtiles     # enable steps 16-17 (selectivity + tile rewrite)
 *   just bench --isolated                           # per-pass isolated impact (non-cumulative)
 *   just bench-debug tokyo                          # with browser console output
 */

import os from "node:os";
import path from "node:path";
import fs from "node:fs";
import zlib from "node:zlib";
import { createHash } from "node:crypto";
import { execFileSync, execSync } from "node:child_process";
import { performance } from "node:perf_hooks";
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

const TILE_PROXY_PORT = 8765;
const TILE_PROXY_URL = `http://localhost:${TILE_PROXY_PORT}`;

// ── benchmark styles ────────────────────────────────────────────────────────
// Styles are fetched, cached, and their tile sources rewritten to use the
// openfreemap proxy. All styles must use OpenMapTiles schema so the tiles
// are interchangeable.

interface BenchStyle {
  id: string;
  url: string;
  cachePath: string;
  schema: string;
}

const BENCH_STYLES: BenchStyle[] = [
  // OpenFreeMap built-in styles (already point to the right tile server)
  { id: "liberty",  url: "https://tiles.openfreemap.org/styles/liberty",  cachePath: path.join(RESULTS_DIR, "_cached_liberty.json"),  schema: "omt" },
  { id: "bright",   url: "https://tiles.openfreemap.org/styles/bright",   cachePath: path.join(RESULTS_DIR, "_cached_bright.json"),   schema: "omt" },
  { id: "positron", url: "https://tiles.openfreemap.org/styles/positron", cachePath: path.join(RESULTS_DIR, "_cached_positron.json"), schema: "omt" },
  { id: "fiord",    url: "https://tiles.openfreemap.org/styles/fiord",    cachePath: path.join(RESULTS_DIR, "_cached_fiord.json"),    schema: "omt" },
  // OpenMapTiles community styles — more verbose/unoptimized, good for showing optimizer impact
  { id: "dark-matter",  url: "https://cdn.jsdelivr.net/gh/openmaptiles/dark-matter-gl-style@v1.9/style.json",      cachePath: path.join(RESULTS_DIR, "_cached_dark-matter.json"),  schema: "omt" },
  { id: "osm-bright",   url: "https://cdn.jsdelivr.net/gh/openmaptiles/osm-bright-gl-style@v1.11/style.json",      cachePath: path.join(RESULTS_DIR, "_cached_osm-bright.json"),   schema: "omt" },
  { id: "klokan-basic", url: "https://cdn.jsdelivr.net/gh/openmaptiles/klokantech-basic-gl-style@v1.10/style.json", cachePath: path.join(RESULTS_DIR, "_cached_klokan-basic.json"), schema: "omt" },
  { id: "toner",        url: "https://cdn.jsdelivr.net/gh/openmaptiles/toner-gl-style@v1.0/style.json",             cachePath: path.join(RESULTS_DIR, "_cached_toner.json"),        schema: "omt" },
  { id: "osm-liberty",  url: "https://maputnik.github.io/osm-liberty/style.json",                                   cachePath: path.join(RESULTS_DIR, "_cached_osm-liberty.json"),  schema: "omt" },
  { id: "americana",       url: "https://americanamap.org/style.json",                                              cachePath: path.join(RESULTS_DIR, "_cached_americana.json"),       schema: "omt" },
  { id: "stadia-outdoors", url: "https://tiles.stadiamaps.com/styles/outdoors.json",                                 cachePath: path.join(RESULTS_DIR, "_cached_stadia-outdoors.json"), schema: "omt" },
  // Government/institutional styles — verbose, many layers, legacy syntax needs migration
  { id: "icgc-fosc",      url: "https://geoserveis.icgc.cat/contextmaps/icgc_mapa_base_fosc.json",                  cachePath: path.join(RESULTS_DIR, "_cached_icgc-fosc.json"),  schema: "omt" },
  { id: "icgc-gris",      url: "https://geoserveis.icgc.cat/contextmaps/icgc_mapa_base_gris.json",                  cachePath: path.join(RESULTS_DIR, "_cached_icgc-gris.json"),  schema: "omt" },
  { id: "basemap-top",    url: "https://sgx.geodatenzentrum.de/gdz_basemapde_vektor/styles/bm_web_top.json",        cachePath: path.join(RESULTS_DIR, "_cached_basemap-top.json"), schema: "basemap-de" },
  { id: "basemap-col",    url: "https://sgx.geodatenzentrum.de/gdz_basemapde_vektor/styles/bm_web_col.json",        cachePath: path.join(RESULTS_DIR, "_cached_basemap-col.json"), schema: "basemap-de" },
];

// ── CLI args ─────────────────────────────────────────────────────────────────

const argv = process.argv.slice(2);
const debug = argv.includes("--debug");
const isolated = argv.includes("--isolated");
const runsArg = argv.findIndex((a) => a === "--runs");
const warmupArg = argv.findIndex((a) => a === "--warmup");
const mbtilesArg = argv.findIndex((a) => a === "--mbtiles");
const stylesArg = argv.findIndex((a) => a === "--styles");
const RUNS = runsArg >= 0 ? parseInt(argv[runsArg + 1], 10) : 15;
const WARMUP = warmupArg >= 0 ? parseInt(argv[warmupArg + 1], 10) : 1;
const MBTILES = mbtilesArg >= 0 ? argv[mbtilesArg + 1] : undefined;
// --styles liberty,bright or omit for all
const styleFilter = stylesArg >= 0 ? argv[stylesArg + 1].split(",") : undefined;
const STYLES = styleFilter
  ? BENCH_STYLES.filter((s) => styleFilter.includes(s.id))
  : BENCH_STYLES;
const filters = argv.filter(
  (a, i) =>
    !a.startsWith("--") &&
    i !== runsArg + 1 &&
    i !== warmupArg + 1 &&
    i !== mbtilesArg + 1 &&
    i !== stylesArg + 1,
);

const RUN_TIMEOUT = 120_000;

// ── types ────────────────────────────────────────────────────────────────────

interface RunMetrics {
  loadMs: number;
  idleMs: number;
  fps: number;
  meanFrameMs: number;
  frameTimeVariance: number;
  droppedFrameRatio: number;
  p50FrameMs: number;
  p95FrameMs: number;
  p99FrameMs: number;
  jankCount: number;
  animationMs: number;
  styleParseMs: number;
  firstTileMs: number;
  firstFrameMs: number;
  heapUsedMB: number;
  peakHeapMB: number;
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
  /**
   * SHA-256 of `styleJson`. Used to dedupe browser-side measurements within
   * a scenario iteration: ablation steps whose output is byte-identical to
   * an earlier step (common when a pass is a no-op for a given style) share
   * a single render and copy the metrics across. Saves 20-60% of iteration
   * time on styles with sparse ablation deltas.
   */
  styleHash: string;
  styleBytes: number;
  gzipBytes: number;
  brotliBytes: number;
  complexity: ComplexityReport | null;
  /**
   * Wall-clock time of the style-optimizer CLI invocation that produced this
   * variant (ms). For `tile_rewrite` variants this covers the combined
   * optimize-with-advisory + advisory-rewrite invocations. For the `baseline`
   * variant (no passes) this is 0. Consumed by the post-hoc Pareto analysis
   * in `thesis/scripts/pareto_analysis.py` as the `preprocessing_ms` axis.
   */
  preprocessingMs: number;
}

interface ComplexityReport {
  ast_nodes: number;
  max_depth: number;
  layer_count: number;
  filter_count: number;
  expression_types: Record<string, number>;
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
  { pass: "simplify_unary",           flag: "--simplify-unary" },
  { pass: "expression_kind",          flag: "--expression-kind" },
  { pass: "constant_fold",            flag: "--constant-fold" },
  { pass: "constant_fold_stats",      flag: "--constant-fold-stats" },
  { pass: "simplify_expressions",     flag: "--simplify-expressions" },
  { pass: "strip_defaults",           flag: "--strip-defaults" },
  { pass: "minify_colors",            flag: "--minify-colors" },
  { pass: "strip_metadata",           flag: "--strip-metadata" },
  { pass: "dead_elimination",         flag: "--dead-elimination" },
  { pass: "dead_elimination_stats",   flag: "--dead-elimination-stats" },
  { pass: "metadata_refinement",      flag: "--metadata-refinement" },
  { pass: "metadata_refinement_paint", flag: "--metadata-refinement-paint" },
  { pass: "metadata_refinement_stats", flag: "--metadata-refinement-stats" },
  { pass: "cleanup",                  flag: "--cleanup" },
  { pass: "layer_merge",              flag: "--layer-merge" },
  // Only when --mbtiles is provided
  { pass: "selectivity_reorder",      flag: "--selectivity-reorder" },
  // Virtual step: runs advisory to rewrite tiles (MLT) and style; requires --mbtiles
  { pass: "tile_rewrite",             flag: "__tile_rewrite__" },
];

// ── Puppeteer args ───────────────────────────────────────────────────────────

const HALF_CPUS = Math.max(1, Math.floor(os.cpus().length / 2));

const PUPPETEER_ARGS = [
  // Use real GPU via ANGLE/Vulkan for realistic performance measurement.
  // Unlike the render tests (which use SwiftShader for pixel-exact comparison),
  // benchmarks need real GPU timings to be meaningful.
  "--enable-gpu",
  "--enable-webgl",
  "--ignore-gpu-blocklist",
  "--enable-unsafe-swiftshader",
  "--use-angle=vulkan",
  "--enable-features=Vulkan,UseSkiaRenderer",
  // Disable vsync so the GPU renders as fast as it can — without this, all
  // frame-time metrics are ceiling-limited at 16.67ms (60 Hz) and show zero
  // differentiation between baseline and optimized styles.
  "--disable-frame-rate-limit",
  "--disable-gpu-vsync",
  // Limit Chrome's parallelism to half the available CPUs for more stable benchmarks
  `--renderer-process-limit=${HALF_CPUS}`,
  "--disable-background-networking",
  "--disable-background-timer-throttling",
  "--disable-backgrounding-occluded-windows",
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

// ── fetch and cache styles ───────────────────────────────────────────────────

async function fetchStyle(style: BenchStyle): Promise<string> {
  if (fs.existsSync(style.cachePath)) {
    console.log(`Using cached ${style.id} from ${style.cachePath}`);
    return fs.readFileSync(style.cachePath, "utf8");
  }
  console.log(`Fetching ${style.id} from ${style.url}…`);
  const resp = await fetch(style.url);
  if (!resp.ok) throw new Error(`Failed to fetch ${style.id}: ${resp.status} ${resp.statusText}`);
  const text = await resp.text();
  JSON.parse(text);

  // Run gl-style-migrate to convert legacy property functions and filters to expressions
  const rawPath = style.cachePath + ".raw";
  fs.writeFileSync(rawPath, text);
  try {
    const migrated = execFileSync("gl-style-migrate", [rawPath], { encoding: "utf8", timeout: 10_000 });
    JSON.parse(migrated);
    fs.writeFileSync(style.cachePath, migrated);
    console.log(`Cached ${style.id} (${(text.length / 1024).toFixed(1)} KB raw → ${(migrated.length / 1024).toFixed(1)} KB migrated)`);
    return migrated;
  } catch {
    // Migration failed — use the raw style as-is
    fs.writeFileSync(style.cachePath, text);
    console.log(`Cached ${style.id} (${(text.length / 1024).toFixed(1)} KB, migration skipped)`);
    return text;
  } finally {
    try { fs.unlinkSync(rawPath); } catch {}
  }
}

// ── optimize style via Rust binary ───────────────────────────────────────────

function optimizeStyle(inputPath: string, passFlags: string[], extraArgs: string[] = []): string {
  const tmpOut = path.join(RESULTS_DIR, `_bench_output_${process.pid}.json`);
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

/** Run the advisory command to rewrite tiles and style for MLT serving. */
function runAdvisory(
  advisoryJsonPath: string,
  mbtilesPath: string,
  stylePath: string,
  outputDir: string,
): void {
  fs.mkdirSync(outputDir, { recursive: true });
  execFileSync(OPTIMIZER, [
    "advisory",
    "--advisory", advisoryJsonPath,
    "--tiles", mbtilesPath,
    "--style", stylePath,
    "--output", outputDir,
  ], { timeout: 300_000 });
}

/** Signal the tile proxy to load a new mbtiles file. */
async function proxyLoadMbtiles(mbtilesPath: string): Promise<void> {
  const resp = await fetch(`${TILE_PROXY_URL}/control/load-mbtiles`, {
    method: "POST",
    body: JSON.stringify({ path: mbtilesPath }),
  });
  if (!resp.ok) throw new Error(`Failed to load mbtiles in proxy: ${resp.status} ${await resp.text()}`);
}

/** Signal the tile proxy to unload the current mbtiles file. */
async function proxyUnloadMbtiles(): Promise<void> {
  const resp = await fetch(`${TILE_PROXY_URL}/control/unload-mbtiles`, {
    method: "POST",
  });
  if (!resp.ok) throw new Error(`Failed to unload mbtiles in proxy: ${resp.status} ${await resp.text()}`);
}

/** Rewrite a style's vector sources to serve from the proxy's mbtiles endpoint. */
function rewriteStyleForMbtiles(styleJson: string): string {
  const style = JSON.parse(styleJson);
  if (style.sources) {
    for (const src of Object.values(style.sources as Record<string, any>)) {
      if (src.type === "vector") {
        src.tiles = [`${TILE_PROXY_URL}/mbtiles/{z}/{x}/{y}`];
        delete src.url;
      }
    }
  }
  return JSON.stringify(style);
}

// ── compressed size helpers (Benchmark 1) ─────────────────────────────────────

function gzipSize(data: string): number {
  return zlib.gzipSync(Buffer.from(data, "utf8"), { level: 9 }).length;
}

function brotliSize(data: string): number {
  return zlib.brotliCompressSync(Buffer.from(data, "utf8"), {
    params: { [zlib.constants.BROTLI_PARAM_QUALITY]: zlib.constants.BROTLI_MAX_QUALITY },
  }).length;
}

// ── complexity report (Benchmark 2) ──────────────────────────────────────────

function getComplexityReport(styleJsonPath: string): ComplexityReport | null {
  try {
    const output = execFileSync(OPTIMIZER, ["complexity", "--input", styleJsonPath], {
      timeout: 10_000,
      encoding: "utf8",
    });
    return JSON.parse(output) as ComplexityReport;
  } catch {
    return null;
  }
}

function formatKB(json: string): string {
  return `${(Buffer.byteLength(json, "utf8") / 1024).toFixed(1)} KB`;
}

/**
 * Build the cumulative ablation variants.
 *
 * Returns an array of Variant objects: baseline (no passes) followed by one
 * variant per ablation step, each accumulating all passes from previous steps.
 * Steps 16 (selectivity_reorder) and 17 (tile_rewrite) are only included when
 * --mbtiles is provided.
 */
function hashStyleJson(styleJson: string): string {
  return createHash("sha256").update(styleJson).digest("hex");
}

async function buildVariants(originalStyleJson: string, schema: string): Promise<Variant[]> {
  const variants: Variant[] = [];

  // Write input style once for all optimizer invocations
  const inputPath = path.join(RESULTS_DIR, `_bench_input_${process.pid}.json`);
  fs.writeFileSync(inputPath, originalStyleJson);

  // Step 0: baseline (no optimization)
  const baselineBytes = Buffer.byteLength(originalStyleJson, "utf8");
  const baselineGzip = gzipSize(originalStyleJson);
  const baselineBrotli = brotliSize(originalStyleJson);
  const baselineComplexity = getComplexityReport(inputPath);
  variants.push({
    id: "step-00-baseline",
    label: "baseline",
    passes: [],
    styleJson: originalStyleJson,
    styleHash: hashStyleJson(originalStyleJson),
    styleBytes: baselineBytes,
    gzipBytes: baselineGzip,
    brotliBytes: baselineBrotli,
    complexity: baselineComplexity,
    preprocessingMs: 0,
  });
  console.log(`  step-00-baseline: ${formatKB(originalStyleJson)} (gzip: ${(baselineGzip / 1024).toFixed(1)} KB, br: ${(baselineBrotli / 1024).toFixed(1)} KB)`);

  // Collect stats if mbtiles provided and schema matches OMT (stats are meaningless for other schemas)
  let statsPath: string | undefined;
  if (MBTILES && schema === "omt") {
    statsPath = path.join(RESULTS_DIR, `_bench_stats_${process.pid}.json`);
    console.log(`Collecting tile statistics from ${MBTILES}…`);
    collectStats(MBTILES, statsPath);
  }

  // Determine how many steps to run
  // Last two steps (selectivity_reorder, tile_rewrite) require --mbtiles
  const maxSteps = MBTILES ? ABLATION_STEPS.length : ABLATION_STEPS.length - 2;

  // Build pass flags depending on mode
  const accumulatedFlags: string[] = [];
  const accumulatedPasses: string[] = [];

  const advisoryDir = path.join(RESULTS_DIR, `_bench_advisory_${process.pid}`);

  try {
    for (let i = 0; i < maxSteps; i++) {
      const step = ABLATION_STEPS[i];

      // tile_rewrite is a virtual step — skip in isolated mode
      if (step.pass === "tile_rewrite" && isolated) continue;

      // tile_rewrite: run advisory pipeline to rewrite tiles + style for MLT
      if (step.pass === "tile_rewrite") {
        const stepNum = String(i + 1).padStart(2, "0");
        const variantId = `step-${stepNum}-${step.pass}`;

        // Run the full optimizer with --advisory to produce the advisory JSON
        const advisoryJsonPath = path.join(RESULTS_DIR, `_bench_advisory_${process.pid}.json`);
        const allPassFlags = accumulatedFlags.filter((f) => f !== "__tile_rewrite__");
        const optimizedStylePath = path.join(RESULTS_DIR, `_bench_optimized_for_advisory_${process.pid}.json`);
        const tmpOut = path.join(RESULTS_DIR, `_bench_advisory_opt_${process.pid}.json`);
        // Preprocessing timer covers the combined optimize-with-advisory + advisory-rewrite
        // invocations, since together they are the offline pipeline this variant exercises.
        const preprocessingStart = performance.now();
        try {
          execFileSync(OPTIMIZER, [
            "optimize", "--input", inputPath, "--output", tmpOut,
            ...allPassFlags,
            "--stats", statsPath!,
            "--advisory", advisoryJsonPath,
          ], { timeout: 30_000 });
          fs.copyFileSync(tmpOut, optimizedStylePath);
        } finally {
          try { fs.unlinkSync(tmpOut); } catch {}
        }

        // Run the advisory command to produce rewritten mbtiles + style
        console.log(`  Running advisory tile rewrite…`);
        runAdvisory(advisoryJsonPath, MBTILES!, optimizedStylePath, advisoryDir);
        const preprocessingMs = performance.now() - preprocessingStart;

        // Find the rewritten mbtiles and style in the output directory
        const advisoryFiles = fs.readdirSync(advisoryDir);
        const rewrittenMbtiles = advisoryFiles.find((f) => f.endsWith(".mbtiles"));
        const rewrittenStyle = advisoryFiles.find((f) => f.endsWith(".json"));

        if (!rewrittenStyle) {
          console.log(`  WARNING: advisory produced no rewritten style, skipping tile_rewrite step`);
          continue;
        }

        // Read the advisory-rewritten style and rewrite source URLs for the proxy
        let advisoryStyleJson = fs.readFileSync(path.join(advisoryDir, rewrittenStyle), "utf8");
        advisoryStyleJson = rewriteStyleForMbtiles(advisoryStyleJson);

        // Signal the proxy to serve the rewritten mbtiles
        if (rewrittenMbtiles) {
          await proxyLoadMbtiles(path.resolve(advisoryDir, rewrittenMbtiles));
        }

        const styleBytes = Buffer.byteLength(advisoryStyleJson, "utf8");
        const gzBytes = gzipSize(advisoryStyleJson);
        const brBytes = brotliSize(advisoryStyleJson);

        const tmpComplexity = path.join(RESULTS_DIR, `_bench_complexity_${process.pid}.json`);
        fs.writeFileSync(tmpComplexity, advisoryStyleJson);
        const complexity = getComplexityReport(tmpComplexity);
        try { fs.unlinkSync(tmpComplexity); } catch {}

        const pctSmaller = ((1 - styleBytes / baselineBytes) * 100).toFixed(1);

        accumulatedPasses.push(step.pass);
        variants.push({
          id: variantId,
          label: `+${step.pass}`,
          passes: [...accumulatedPasses],
          styleJson: advisoryStyleJson,
          styleHash: hashStyleJson(advisoryStyleJson),
          styleBytes,
          gzipBytes: gzBytes,
          brotliBytes: brBytes,
          complexity,
          preprocessingMs,
        });
        console.log(`  ${variantId}: ${formatKB(advisoryStyleJson)} (${pctSmaller}% smaller, gzip: ${(gzBytes / 1024).toFixed(1)} KB, br: ${(brBytes / 1024).toFixed(1)} KB)`);

        // Clean up advisory intermediate files
        try { fs.unlinkSync(advisoryJsonPath); } catch {}
        try { fs.unlinkSync(optimizedStylePath); } catch {}

        continue;
      }

      let passFlags: string[];
      let passList: string[];
      let variantId: string;

      if (isolated) {
        // Isolated mode: each variant has only one pass
        passFlags = [step.flag];
        passList = [step.pass];
        const stepNum = String(i + 1).padStart(2, "0");
        variantId = `isolated-${stepNum}-${step.pass}`;
      } else {
        // Cumulative mode: accumulate passes
        accumulatedFlags.push(step.flag);
        accumulatedPasses.push(step.pass);
        passFlags = [...accumulatedFlags];
        passList = [...accumulatedPasses];
        const stepNum = String(i + 1).padStart(2, "0");
        variantId = `step-${stepNum}-${step.pass}`;
      }

      // Pass --stats when any accumulated pass needs tile statistics
      const extraArgs: string[] = [];
      const statsDependent = [
        "constant_fold_stats",
        "dead_elimination_stats",
        "metadata_refinement_stats",
        "selectivity_reorder",
      ];
      if (statsPath && passList.some((p) => statsDependent.includes(p))) {
        extraArgs.push("--stats", statsPath);
      }

      const preprocessingStart = performance.now();
      const optimizedJson = optimizeStyle(inputPath, passFlags, extraArgs);
      const preprocessingMs = performance.now() - preprocessingStart;
      const styleBytes = Buffer.byteLength(optimizedJson, "utf8");
      const gzBytes = gzipSize(optimizedJson);
      const brBytes = brotliSize(optimizedJson);

      // Write optimized style to get complexity report
      const tmpComplexity = path.join(RESULTS_DIR, `_bench_complexity_${process.pid}.json`);
      fs.writeFileSync(tmpComplexity, optimizedJson);
      const complexity = getComplexityReport(tmpComplexity);
      try { fs.unlinkSync(tmpComplexity); } catch {}

      const pctSmaller = ((1 - styleBytes / baselineBytes) * 100).toFixed(1);

      variants.push({
        id: variantId,
        label: isolated ? step.pass : `+${step.pass}`,
        passes: passList,
        styleJson: optimizedJson,
        styleHash: hashStyleJson(optimizedJson),
        styleBytes,
        gzipBytes: gzBytes,
        brotliBytes: brBytes,
        complexity,
        preprocessingMs,
      });
      console.log(`  ${variantId}: ${formatKB(optimizedJson)} (${pctSmaller}% smaller, gzip: ${(gzBytes / 1024).toFixed(1)} KB, br: ${(brBytes / 1024).toFixed(1)} KB)`);
    }
  } finally {
    try { fs.unlinkSync(inputPath); } catch {}
    if (statsPath) {
      try { fs.unlinkSync(statsPath); } catch {}
    }
    // Clean up advisory output directory
    try { fs.rmSync(advisoryDir, { recursive: true, force: true }); } catch {}
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
      // Benchmark 5: measure style parse time
      var parseStart = performance.now();
      var style = JSON.parse(styleJSON);
      var styleParseMs = performance.now() - parseStart;

      var startTime = performance.now();
      var frameTimestamps = [];
      var firstFrameMs = null;
      var firstTileMs = null;
      var peakHeap = 0;
      var heapSampler = null;

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

      // Benchmark 5: capture first tile data event
      map.on("data", function(e) {
        if (firstTileMs === null && e.dataType === "source" && e.tile) {
          firstTileMs = performance.now() - startTime;
        }
      });

      map.on("render", function() {
        var now = performance.now();
        frameTimestamps.push(now);
        if (firstFrameMs === null) {
          firstFrameMs = now - startTime;
        }
      });

      var loadMs, idleMs;

      map.once("load", function() {
        loadMs = performance.now() - startTime;

        // Benchmark 7: start heap sampling
        if (window.performance && window.performance.memory) {
          heapSampler = setInterval(function() {
            var mem = window.performance.memory;
            if (mem && mem.usedJSHeapSize > peakHeap) {
              peakHeap = mem.usedJSHeapSize;
            }
          }, 200);
        }

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
              if (heapSampler) clearInterval(heapSampler);

              // Final heap measurement
              var heapUsed = 0;
              if (window.performance && window.performance.memory) {
                var mem = window.performance.memory;
                heapUsed = mem.usedJSHeapSize;
                if (heapUsed > peakHeap) peakHeap = heapUsed;
              }

              map.remove();
              resolve({
                loadMs: loadMs,
                idleMs: idleMs,
                animationMs: animationMs,
                frames: frameTimestamps,
                styleParseMs: styleParseMs,
                firstTileMs: firstTileMs || 0,
                firstFrameMs: firstFrameMs || 0,
                heapUsedMB: heapUsed / (1024 * 1024),
                peakHeapMB: peakHeap / (1024 * 1024),
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
  const style = JSON.parse(styleJson);

  // Rewrite all vector tile sources to use the proxy's openfreemap tiles
  if (style.sources) {
    for (const [key, src] of Object.entries(style.sources as Record<string, any>)) {
      if (src.type === "vector") {
        // Point to the proxy's openmaptiles source
        src.url = `${TILE_PROXY_URL}/sources/openmaptiles`;
        delete src.tiles;
      }
      // Rewrite raster/raster-dem tile URLs if they point to openfreemap
      if (src.tiles && Array.isArray(src.tiles)) {
        src.tiles = src.tiles.map((t: string) =>
          t.replace("https://tiles.openfreemap.org", TILE_PROXY_URL),
        );
      }
      if (src.url && typeof src.url === "string") {
        src.url = src.url.replace("https://tiles.openfreemap.org", TILE_PROXY_URL);
      }
    }
  }

  // Rewrite sprite and glyphs URLs to use the proxy
  if (style.sprite && typeof style.sprite === "string") {
    style.sprite = style.sprite.replace("https://tiles.openfreemap.org", TILE_PROXY_URL);
    // External sprites (jsdelivr etc.) → use openfreemap's sprite via proxy
    if (!style.sprite.startsWith(TILE_PROXY_URL)) {
      style.sprite = `${TILE_PROXY_URL}/sprites/liberty/sprite`;
    }
  }
  if (style.glyphs && typeof style.glyphs === "string") {
    style.glyphs = style.glyphs.replace("https://tiles.openfreemap.org", TILE_PROXY_URL);
    if (!style.glyphs.startsWith(TILE_PROXY_URL)) {
      style.glyphs = `${TILE_PROXY_URL}/fonts/{fontstack}/{range}.pbf`;
    }
  }

  return JSON.stringify(style);
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
  ) as {
    loadMs: number; idleMs: number; animationMs: number; frames: number[];
    styleParseMs: number; firstTileMs: number; firstFrameMs: number;
    heapUsedMB: number; peakHeapMB: number;
  };

  // Benchmark 7: also capture Puppeteer-level heap metrics as fallback
  let heapUsedMB = raw.heapUsedMB;
  let peakHeapMB = raw.peakHeapMB;
  if (heapUsedMB === 0) {
    try {
      const puppeteerMetrics = await page.metrics();
      heapUsedMB = (puppeteerMetrics.JSHeapUsedSize ?? 0) / (1024 * 1024);
      peakHeapMB = (puppeteerMetrics.JSHeapTotalSize ?? 0) / (1024 * 1024);
    } catch {}
  }

  return computeMetrics({ ...raw, heapUsedMB, peakHeapMB });
}

// ── compute per-run metrics from raw frame data ──────────────────────────────

function computeMetrics(raw: {
  loadMs: number;
  idleMs: number;
  animationMs: number;
  frames: number[];
  styleParseMs: number;
  firstTileMs: number;
  firstFrameMs: number;
  heapUsedMB: number;
  peakHeapMB: number;
}): RunMetrics {
  const { loadMs, idleMs, animationMs, frames, styleParseMs, firstTileMs, firstFrameMs, heapUsedMB, peakHeapMB } = raw;

  const deltas: number[] = [];
  for (let i = 1; i < frames.length; i++) {
    deltas.push(frames[i] - frames[i - 1]);
  }

  if (deltas.length === 0) {
    return {
      loadMs, idleMs, animationMs,
      fps: 0, meanFrameMs: 0, frameTimeVariance: 0, droppedFrameRatio: 0,
      p50FrameMs: 0, p95FrameMs: 0, p99FrameMs: 0, jankCount: 0,
      styleParseMs, firstTileMs, firstFrameMs,
      heapUsedMB, peakHeapMB,
    };
  }

  const sorted = [...deltas].sort((a, b) => a - b);
  const fps = deltas.length / (animationMs / 1000);
  const p50FrameMs = percentile(sorted, 0.5);
  const p95FrameMs = percentile(sorted, 0.95);
  const p99FrameMs = percentile(sorted, 0.99);

  const meanFrameMs = animationMs / frames.length;
  const meanDelta = deltas.reduce((a, b) => a + b, 0) / deltas.length;
  const frameTimeVariance = Math.sqrt(
    deltas.reduce((sum, d) => sum + (d - meanDelta) ** 2, 0) / deltas.length,
  );
  const droppedFrameRatio = deltas.filter((d) => d > 16.67).length / deltas.length;

  const median = p50FrameMs;
  const jankCount = deltas.filter((d) => d > 2 * median).length;

  return {
    loadMs, idleMs, fps, meanFrameMs, frameTimeVariance, droppedFrameRatio,
    p50FrameMs, p95FrameMs, p99FrameMs, jankCount, animationMs,
    styleParseMs, firstTileMs, firstFrameMs,
    heapUsedMB, peakHeapMB,
  };
}

function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return 0;
  const idx = Math.max(0, Math.ceil(sorted.length * p) - 1);
  return sorted[idx];
}

// ── aggregation across runs ──────────────────────────────────────────────────

function aggregate(runs: RunMetrics[]): AggregatedMetrics {
  const keys: (keyof RunMetrics)[] = [
    "loadMs", "idleMs", "fps", "meanFrameMs", "frameTimeVariance", "droppedFrameRatio",
    "p50FrameMs", "p95FrameMs", "p99FrameMs", "jankCount", "animationMs",
    "styleParseMs", "firstTileMs", "firstFrameMs",
    "heapUsedMB", "peakHeapMB",
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

  // Resolve scenarios
  const allScenarios = getAllScenarios();
  const scenarios = filterScenarios(allScenarios, filters);
  if (scenarios.length === 0) {
    console.error("No scenarios matched filters:", filters);
    process.exit(1);
  }

  const mode = isolated ? "isolated" : "cumulative";
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const jsonlPath = path.join(RESULTS_DIR, `bench-${timestamp}.jsonl`);
  const jsonlFd = fs.openSync(jsonlPath, "w");
  console.log(`Streaming results to ${jsonlPath}\n`);

  console.log(`Styles: ${STYLES.map((s) => s.id).join(", ")}`);
  console.log(`Scenarios: ${scenarios.length}, Runs: ${RUNS}, Warmup: ${WARMUP}, Mode: ${mode}\n`);

  let browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });

  function applyDebugListeners(p: Page): void {
    if (!debug) return;
    p.on("console", (msg) => console.log(`  [browser] ${msg.text()}`));
    p.on("pageerror", (err) => console.error(`  [browser error] ${err.message}`));
  }

  const allResults: ScenarioResult[] = [];
  let lastVariants: Variant[] = [];

  for (const benchStyle of STYLES) {
    console.log(`\n${"═".repeat(72)}`);
    console.log(`Style: ${benchStyle.id}`);
    console.log(`${"═".repeat(72)}`);

    const remoteStyleJson = await fetchStyle(benchStyle);
    const originalStyleJson = rewriteStyleForProxy(remoteStyleJson);
    const originalSize = Buffer.byteLength(originalStyleJson, "utf8");
    console.log(`Original: ${(originalSize / 1024).toFixed(1)} KB (URLs rewritten to local proxy)\n`);

    console.log(`Building ${mode} ablation variants…`);
    const variants = await buildVariants(originalStyleJson, benchStyle.schema);
    lastVariants = variants;
    console.log(`${variants.length} ablation steps\n`);

    for (let si = 0; si < scenarios.length; si++) {
      const scenario = scenarios[si];
      const scenarioId = `${benchStyle.id}/${scenario.id}`;
      console.log(`[${si + 1}/${scenarios.length}] ${scenarioId} (${scenario.location.name}, ${scenario.animationType})`);

      const variantRuns: Record<string, RunMetrics[]> = {};
      for (const v of variants) variantRuns[v.id] = [];
      const totalRuns = WARMUP + RUNS;

      for (let run = 0; run < totalRuns; run++) {
        const isWarmup = run < WARMUP;
        const label = isWarmup ? `warmup ${run + 1}` : `run ${run - WARMUP + 1}/${RUNS}`;

        const parts: string[] = [];
        // Dedupe: skip browser render for variants with byte-identical style.
        const metricsByHash = new Map<string, RunMetrics>();
        const errorByHash = new Map<string, string>();

        for (let vi = 0; vi < variants.length; vi++) {
          const variant = variants[vi];

          let metrics: RunMetrics | undefined = metricsByHash.get(variant.styleHash);
          let errMsg: string | undefined;
          let deduped = false;

          if (metrics === undefined) {
            errMsg = errorByHash.get(variant.styleHash);
            if (errMsg === undefined) {
              const page = await browser.newPage();
              applyDebugListeners(page);

              try {
                metrics = await runBenchmarkInBrowser(page, variant.styleJson, scenario);
                metricsByHash.set(variant.styleHash, metrics);
              } catch (err: unknown) {
                errMsg = err instanceof Error ? err.message : String(err);
                errorByHash.set(variant.styleHash, errMsg);
              } finally {
                try { await page.close(); } catch {}
              }
            }
          } else {
            deduped = true;
          }

          if (metrics !== undefined) {
            if (!isWarmup) {
              variantRuns[variant.id].push(metrics);
              const record: Record<string, unknown> = {
                style: benchStyle.id,
                schema: benchStyle.schema,
                has_stats: MBTILES !== undefined && benchStyle.schema === "omt",
                scenario: scenarioId,
                location: scenario.location.name,
                lng: scenario.location.center[0],
                lat: scenario.location.center[1],
                zoom: scenario.location.zoom,
                animation: scenario.animationType,
                variant: variant.id,
                passes: variant.passes,
                mode,
                style_bytes: variant.styleBytes,
                gzip_bytes: variant.gzipBytes,
                brotli_bytes: variant.brotliBytes,
                style_hash: variant.styleHash,
                deduped,
                preprocessing_ms: variant.preprocessingMs,
                run: run - WARMUP + 1,
                timestamp,
                ...metrics,
              };
              if (variant.complexity) {
                record.layer_count = variant.complexity.layer_count;
                record.filter_count = variant.complexity.filter_count;
                record.property_count = variant.complexity.property_count;
                record.expression_property_count = variant.complexity.expression_property_count;
                record.scalar_property_count = variant.complexity.scalar_property_count;
                record.total_expression_nodes = variant.complexity.total_expression_nodes;
                record.ast_nodes = variant.complexity.ast_nodes;
                record.max_depth = variant.complexity.max_depth;
                record.expression_types = variant.complexity.expression_types;
              }
              fs.writeSync(jsonlFd, JSON.stringify(record) + "\n");
            }
            const marker = deduped ? "·" : "";
            parts.push(`${variant.id.replace(/step-|isolated-/, "s")}: ${metrics.loadMs.toFixed(0)}ms${marker}`);
          } else if (errMsg !== undefined) {
            console.error(`\n  ⚠ ${variant.id} error: ${errMsg}`);
            parts.push(`${variant.id}: ERR`);
            if (errMsg.includes("Session closed") || errMsg.includes("Target closed") || errMsg.includes("Protocol error")) {
              console.log("\n  Recovering browser…");
              try { await browser.close(); } catch {}
              browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });
            }
          }
        }

        console.log(`  ${label} ${parts.join(" | ")}`);
      }

      if (variantRuns[variants[0].id].length === 0) {
        console.log(`  Skipping ${scenarioId} — no successful baseline runs`);
        continue;
      }

      const aggregated: Record<string, AggregatedMetrics> = {};
      for (const v of variants) {
        if (variantRuns[v.id].length > 0) {
          aggregated[v.id] = aggregate(variantRuns[v.id]);
        }
      }

      allResults.push({ scenarioId, variants: aggregated });
    }

    // Unload mbtiles from proxy if tile_rewrite was active
    if (MBTILES && variants.some((v) => v.passes.includes("tile_rewrite"))) {
      await proxyUnloadMbtiles();
    }

    // Per-style size summary
    console.log(`\n── ${benchStyle.id} Style Sizes ──`);
    for (const v of variants) {
      const kb = (v.styleBytes / 1024).toFixed(1);
      const gzKb = (v.gzipBytes / 1024).toFixed(1);
      const brKb = (v.brotliBytes / 1024).toFixed(1);
      const pct = v.id.includes("baseline")
        ? ""
        : ` (${((1 - v.styleBytes / variants[0].styleBytes) * 100).toFixed(1)}% smaller)`;
      console.log(`  ${v.label}: ${kb} KB | gzip ${gzKb} KB | br ${brKb} KB${pct}`);
    }
  }

  const chromeVersion = await browser.version();
  await browser.close();

  if (allResults.length === 0) {
    console.error("No results collected.");
    process.exit(1);
  }

  printSummaryTable(allResults, lastVariants);

  fs.closeSync(jsonlFd);

  const meta = {
    timestamp,
    styles: STYLES.map((s) => s.id),
    mode,
    runsPerScenario: RUNS,
    warmupRuns: WARMUP,
    maplibreVersion: getMaplibreVersion(),
    renderer: "Hardware GPU (headless Chrome, vsync disabled)",
    chromeVersion,
    cpuModel: os.cpus()[0]?.model ?? "unknown",
    nodeVersion: process.version,
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
