#!/usr/bin/env tsx
/**
 * Render-test harness for maplibre-style-optimizer.
 *
 * For every render-test fixture in the maplibre-gl-js submodule:
 *   1. Render the original style  → pixels A
 *   2. Run the optimizer (--all)  → optimised style
 *   3. Render the optimised style → pixels B
 *   4. Compare A vs B with pixelmatch — they must match within tolerance.
 *
 * Usage:
 *   just render-test                     # run all
 *   just render-test background-color    # filter by name
 *   just render-test --debug             # show browser console
 *   just render-test --stats stats.json  # pass tile stats to optimizer
 *   just render-test --concurrency 8     # run N tests in parallel
 *
 * When a test fails, individual passes are automatically bisected to identify culprits.
 */

import path from "node:path";
import fs from "node:fs";
import fsp from "node:fs/promises";
import os from "node:os";
import http from "node:http";
import { execFile, execSync } from "node:child_process";
import { promisify } from "node:util";
import { globSync } from "glob";
import st from "st";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
import puppeteer, { type Page } from "puppeteer";

const execFileAsync = promisify(execFile);

// ── paths ────────────────────────────────────────────────────────────────────

const __dirname = import.meta.dirname!;
const REPO_ROOT = path.resolve(__dirname, "../..");
const SUBMODULE = path.resolve(__dirname, "../maplibre-gl-js");
const FIXTURES = path.join(SUBMODULE, "test/integration/render/tests");
const ASSETS = path.join(SUBMODULE, "test/integration/assets");
const MAPLIBRE_JS = path.resolve(
  __dirname,
  "node_modules/maplibre-gl/dist/maplibre-gl-dev.js",
);
const OPTIMIZER = path.join(
  REPO_ROOT,
  "target/release/maplibre-style-optimize",
);
const RESULTS_DIR = path.join(__dirname, "results");

// ── types ────────────────────────────────────────────────────────────────────

interface TestMetadata {
  id: string;
  width: number;
  height: number;
  pixelRatio: number;
  allowed: number;
  threshold: number;
  timeout: number;
  fadeDuration?: number;
  crossSourceCollisions?: boolean;
  operations?: unknown[][];
  [key: string]: unknown;
}

interface TestStyle {
  version: number;
  metadata: { test: TestMetadata };
  sources?: Record<string, Record<string, unknown>>;
  sprite?: string | Array<{ url: string }>;
  glyphs?: string;
  layers: unknown[];
  [key: string]: unknown;
}

interface FailedTest {
  id: string;
  ratio: number;
  culpritPasses?: string[];
}

interface ErroredTest {
  id: string;
  error: string;
}

// ── cli args ─────────────────────────────────────────────────────────────────

const argv = process.argv.slice(2);
const debug = argv.includes("--debug");

let statsPath: string | undefined;
const statsIdx = argv.indexOf("--stats");
if (statsIdx !== -1 && statsIdx + 1 < argv.length) {
  statsPath = argv[statsIdx + 1];
  if (!fs.existsSync(statsPath)) {
    console.error(`Stats file not found: ${statsPath}`);
    process.exit(1);
  }
}

let concurrency: number;
const concurrencyIdx = argv.indexOf("--concurrency");
if (concurrencyIdx !== -1 && concurrencyIdx + 1 < argv.length) {
  concurrency = parseInt(argv[concurrencyIdx + 1], 10);
  if (Number.isNaN(concurrency) || concurrency < 1) {
    console.error(`Invalid concurrency value: ${argv[concurrencyIdx + 1]}`);
    process.exit(1);
  }
} else {
  concurrency = Math.min(os.cpus().length, 8);
}

const filters = argv.filter(
  (a, i) =>
    !a.startsWith("--") &&
    argv[i - 1] !== "--stats" &&
    argv[i - 1] !== "--concurrency",
);

// ── asset server ─────────────────────────────────────────────────────────────

function startAssetServer(port: number): Promise<http.Server> {
  const mount = st({ path: ASSETS, cors: true, passthrough: true });
  const server = http.createServer((req, res) => {
    res.setHeader("Access-Control-Allow-Origin", "*");
    mount(req, res, () => {
      if (req.url?.includes("/sparse204/1-")) {
        res.writeHead(204);
        res.end("");
      } else {
        res.writeHead(404);
        res.end("");
      }
    });
  });
  return new Promise((resolve) =>
    server.listen(port, "0.0.0.0", () => resolve(server)),
  );
}

// ── URL localisation (mirrors maplibre-gl-js test infra) ─────────────────────

function localizeURLs(style: Partial<TestStyle>, port: number): void {
  const rewrite = (url: string): string =>
    url
      .replace(/^local:\/\//, `http://localhost:${port}/`)
      .replace(/^mapbox:\/\/sprites\//, `http://localhost:${port}/sprites/`)
      .replace(/^mapbox:\/\/fonts/, `http://localhost:${port}/glyphs`)
      .replace(/^mapbox:\/\//, `http://localhost:${port}/tiles/`);

  if (style.sources) {
    for (const src of Object.values(style.sources)) {
      if (Array.isArray(src.tiles))
        src.tiles = src.tiles.map((t: string) => rewrite(t));
      if (Array.isArray(src.urls))
        src.urls = src.urls.map((u: string) => rewrite(u));
      if (typeof src.url === "string") src.url = rewrite(src.url);
      if (typeof src.data === "string") src.data = rewrite(src.data);
    }
  }
  if (typeof style.sprite === "string") style.sprite = rewrite(style.sprite);
  else if (Array.isArray(style.sprite))
    for (const s of style.sprite) s.url = rewrite(s.url);
  if (typeof style.glyphs === "string") style.glyphs = rewrite(style.glyphs);

  // handle operations that set/add styles or sources
  const ops = style.metadata?.test?.operations;
  if (ops) {
    for (const op of ops) {
      if (op[0] === "addSource" && op[2]) {
        localizeURLs(
          { sources: { _: op[2] as Record<string, unknown> } },
          port,
        );
      } else if (op[0] === "setStyle" && typeof op[1] === "object") {
        localizeURLs(op[1] as Partial<TestStyle>, port);
      } else if (
        op[0] === "setStyle" &&
        typeof op[1] === "string" &&
        (op[1] as string).startsWith("local://")
      ) {
        try {
          const relPath = (op[1] as string).replace(/^local:\/\//, "");
          const styleJSON = JSON.parse(
            fs.readFileSync(path.join(ASSETS, relPath), "utf8"),
          );
          localizeURLs(styleJSON, port);
          op[1] = styleJSON;
          op[2] = { diff: false };
        } catch {
          /* skip if missing */
        }
      }
    }
  }
}

// ── legacy filter detection ──────────────────────────────────────────────────

const COMPARISON_OPS = new Set([
  "==",
  "!=",
  "<",
  ">",
  "<=",
  ">=",
  "in",
  "!in",
  "has",
  "!has",
  "none",
]);

/** Returns true if any layer uses a legacy (non-expression) filter. */
function usesLegacyFilter(style: TestStyle): boolean {
  for (const layer of style.layers as Record<string, unknown>[]) {
    if (isLegacyFilter(layer.filter)) return true;
  }
  return false;
}

function isLegacyFilter(filter: unknown): boolean {
  if (!Array.isArray(filter) || filter.length === 0) return false;
  const op = filter[0];
  if (op === "all" || op === "any") {
    return filter.slice(1).some(isLegacyFilter);
  }
  // A comparison with a plain-string second arg (not an array expression)
  // is a legacy filter: ["==", "fieldname", value]
  if (COMPARISON_OPS.has(op) && filter.length >= 2 && typeof filter[1] === "string") {
    return true;
  }
  return false;
}

// ── discover test fixtures ───────────────────────────────────────────────────

function discoverTests(port: number): { styles: TestStyle[]; skipped: number } {
  let skipped = 0;
  const styles: TestStyle[] = globSync("**/style.json", { cwd: FIXTURES })
    .map((rel) => {
      const id = path.dirname(rel);
      const style: TestStyle = JSON.parse(
        fs.readFileSync(path.join(FIXTURES, rel), "utf8"),
      );
      style.metadata = style.metadata || ({} as TestStyle["metadata"]);
      style.metadata.test = {
        id,
        width: 512,
        height: 512,
        pixelRatio: 1,
        allowed: 0.00025,
        threshold: 0.1285,
        timeout: 40_000,
        ...style.metadata.test,
      };
      return style;
    })
    .filter((style) => {
      const id = style.metadata.test.id;
      if (filters.length && !filters.some((f) => id.includes(f))) return false;
      if (id.startsWith("debug/")) {
        skipped++;
        return false;
      }

      // Skip tests that dynamically mutate layers/sources/visibility — the
      // optimizer only handles static styles, so these are out of scope.
      const ops = style.metadata.test.operations;
      if (ops) {
        const dynamicOps = [
          "addLayer",
          "addSource",
          "removeLayer",
          "removeSource",
          "addCustomLayer",
        ];
        if (ops.some((op) => dynamicOps.includes(op[0] as string))) {
          skipped++;
          return false;
        }
        // Skip tests that dynamically change visibility (optimizer removes
        // visibility:none layers statically).
        if (
          ops.some(
            (op) =>
              op[0] === "setLayoutProperty" && op[2] === "visibility",
          )
        ) {
          skipped++;
          return false;
        }
      }

      // Skip terrain/globe tests that require DEM tiles we don't serve.
      if (id.includes("terrain") || id.includes("globe")) {
        skipped++;
        return false;
      }

      // Skip styles that use legacy filters — the optimizer does not support
      // them.  Legacy filters have the form ["op", "fieldname", ...] where
      // the second element is a plain string rather than an expression array.
      if (usesLegacyFilter(style)) {
        skipped++;
        return false;
      }

      return true;
    });

  for (const style of styles) localizeURLs(style, port);
  return { styles, skipped };
}

// ── optimise a style JSON via our Rust binary ────────────────────────────────

let tmpCounter = 0;

async function optimizeStyleWithPasses(
  styleJSON: TestStyle,
  passFlags: string[],
  stats?: string,
): Promise<TestStyle> {
  const id = tmpCounter++;
  const tmpIn = path.join(RESULTS_DIR, `_opt_input_${id}.json`);
  const tmpOut = path.join(RESULTS_DIR, `_opt_output_${id}.json`);
  await fsp.writeFile(tmpIn, JSON.stringify(styleJSON));
  try {
    const args = ["optimize", "--input", tmpIn, "--output", tmpOut, ...passFlags];
    if (stats) args.push("--stats", stats);
    await execFileAsync(OPTIMIZER, args, { timeout: 30_000 });
    return JSON.parse(await fsp.readFile(tmpOut, "utf8"));
  } finally {
    await Promise.all([
      fsp.unlink(tmpIn).catch(() => {}),
      fsp.unlink(tmpOut).catch(() => {}),
    ]);
  }
}

async function optimizeStyle(styleJSON: TestStyle): Promise<TestStyle> {
  return optimizeStyleWithPasses(styleJSON, ["--all"], statsPath);
}

function restoreTestMetadata(optimised: TestStyle, original: TestStyle): void {
  optimised.metadata = optimised.metadata || ({} as TestStyle["metadata"]);
  optimised.metadata.test = original.metadata.test;
}

// ── bisect: find which individual passes cause a diff ────────────────────────

const PASS_FLAGS = [
  "--simplify-unary",
  "--expression-kind",
  "--constant-fold",
  "--dead-elimination",
  "--metadata-refinement",
  "--selectivity-reorder",
  "--strip-metadata",
  "--strip-defaults",
  "--simplify-expressions",
  "--cleanup",
] as const;

async function bisectPasses(
  page: Page,
  style: TestStyle,
  origPixels: Uint8Array,
  w: number,
  h: number,
  allowed: number,
  threshold: number,
): Promise<string[]> {
  const culprits: string[] = [];
  const styleSnapshot = JSON.stringify(style);
  const layersSnapshot = JSON.stringify(style.layers);
  for (const flag of PASS_FLAGS) {
    try {
      const opt = await optimizeStyleWithPasses(JSON.parse(styleSnapshot) as TestStyle, [flag], statsPath);
      restoreTestMetadata(opt, style);

      // Skip render if this pass didn't change the style
      if (JSON.stringify(opt.layers) === layersSnapshot) continue;

      const pixels = await renderStyle(page, opt);
      const { ratio } = comparePixels(origPixels, pixels, w, h, threshold);
      if (ratio > allowed) {
        culprits.push(flag);
      }
    } catch {
      // If a single pass errors, note it but continue
      culprits.push(`${flag}(error)`);
    }
  }
  return culprits;
}

// ── browser-side render function (plain JS string to avoid tsx transforms) ───

const RENDER_FN = `
window.__renderStyle = function(style) {
  var options = style.metadata.test;

  function waitLoaded(map) {
    return new Promise(function(resolve) {
      if (map.loaded()) return resolve();
      map.once("load", resolve);
    });
  }

  function waitIdle(map) {
    return new Promise(function(resolve) { map.once("idle", resolve); });
  }

  function sleep(ms) {
    return new Promise(function(resolve) { setTimeout(resolve, ms); });
  }

  async function applyOperations(map, ops) {
    if (!ops) return;
    for (var i = 0; i < ops.length; i++) {
      var op = ops[i];
      var name = op[0];
      var args = op.slice(1);
      if (name === "wait") {
        if (args.length === 0) {
          // wait for map to be fully loaded (poll via render events)
          while (!map.loaded()) { await map.once("render"); }
        } else if (typeof args[0] === "string") {
          // wait for a named event (e.g. "idle")
          await map.once(args[0]);
        } else {
          // wait N ms then trigger a render
          await sleep(args[0]);
          map._render();
        }
      } else if (name === "sleep") {
        await sleep(args[0]);
      } else if (name === "idle") {
        map.repaint = false;
        await map.once("idle");
      } else if (name === "setStyle") {
        map.setStyle(args[0], {localIdeographFontFamily: false});
      } else if (typeof map[name] === "function") {
        map[name].apply(map, args);
      }
    }
  }

  return new Promise(async function(resolve, reject) {
    setTimeout(function() { reject(new Error("render timeout")); }, options.timeout || 40000);
    try {
      var map = new maplibregl.Map({
        container: "map",
        style: style,
        interactive: false,
        attributionControl: false,
        pixelRatio: options.pixelRatio || 1,
        canvasContextAttributes: { preserveDrawingBuffer: true, powerPreference: "default" },
        fadeDuration: options.fadeDuration || 0,
        crossSourceCollisions: options.crossSourceCollisions !== undefined ? options.crossSourceCollisions : true,
        maxCanvasSize: [8192, 8192],
      });
      map.repaint = true;

      await waitLoaded(map);
      await applyOperations(map, options.operations);

      var gl = map.painter.context.gl;
      var vp = gl.getParameter(gl.VIEWPORT);
      var w = vp[2], h = vp[3];
      var buf = new Uint8Array(w * h * 4);
      gl.readPixels(0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, buf);

      // flip scanlines (WebGL is bottom-up)
      var stride = w * 4;
      var tmp = new Uint8Array(stride);
      for (var i = 0, j = h - 1; i < j; i++, j--) {
        var si = i * stride, sj = j * stride;
        tmp.set(buf.slice(si, si + stride));
        buf.set(buf.slice(sj, sj + stride), si);
        buf.set(tmp, sj);
      }

      map.remove();
      resolve(Array.from(buf));
    } catch (e) {
      reject(e);
    }
  });
};
`;

// ── page pool ────────────────────────────────────────────────────────────────

/** Initialise a page with maplibre-gl and the render helper pre-loaded. */
async function initPage(page: Page): Promise<void> {
  await page.setContent(`<!DOCTYPE html>
<html><head><meta charset="utf-8">
<style>body{margin:0}#map{box-sizing:content-box}</style>
</head><body><div id="map"></div></body></html>`);
  await page.addScriptTag({ path: MAPLIBRE_JS });
  await page.addScriptTag({ content: RENDER_FN });
}

// ── render a style in headless chrome and return raw RGBA pixels ─────────────

async function renderStyle(page: Page, style: TestStyle): Promise<Uint8Array> {
  const width = style.metadata.test.width;
  const height = style.metadata.test.height;

  await page.setViewport({
    width,
    height,
    deviceScaleFactor: style.metadata.test.pixelRatio || 1,
  });

  // Reset the map container for a fresh render without reloading maplibre-gl
  await page.evaluate(
    (w: number, h: number) => {
      const el = document.getElementById("map")!;
      el.style.width = `${w}px`;
      el.style.height = `${h}px`;
      el.innerHTML = "";
    },
    width,
    height,
  );

  // call the browser-side render function (no tsx transforms applied)
  const data = await page.evaluate(
    (s) => (window as any).__renderStyle(s),
    style as any,
  );

  return new Uint8Array(data as number[]);
}

// ── compare two pixel buffers ────────────────────────────────────────────────

function comparePixels(
  a: Uint8Array,
  b: Uint8Array,
  width: number,
  height: number,
  threshold: number,
): { ratio: number; diffPng: PNG } {
  const diff = new PNG({ width, height });
  const numDiff = pixelmatch(a, b, diff.data, width, height, { threshold });
  return { ratio: numDiff / (width * height), diffPng: diff };
}

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

// ── main ─────────────────────────────────────────────────────────────────────

async function main(): Promise<void> {
  buildOptimizer();
  fs.mkdirSync(RESULTS_DIR, { recursive: true });

  const PORT = 2900;
  const server = await startAssetServer(PORT);
  console.log(`Asset server listening on ${PORT}`);

  const { styles, skipped } = discoverTests(PORT);
  console.log(
    `Discovered ${styles.length} render tests (${skipped} skipped)`,
  );

  const PUPPETEER_ARGS = [
    "--disable-gpu",
    "--enable-features=AllowSwiftShaderFallback,AllowSoftwareGLFallbackDueToCrashes",
    "--enable-unsafe-swiftshader",
  ];

  // Clamp concurrency to test count
  const poolSize = Math.min(concurrency, styles.length || 1);
  console.log(`Using ${poolSize} concurrent browser page(s)`);

  let browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });

  function applyDebugListeners(p: Page): void {
    if (!debug) return;
    p.on("console", (msg) => console.log(`  [browser] ${msg.text()}`));
    p.on("pageerror", (err) =>
      console.error(`  [browser error] ${err.message}`),
    );
  }

  async function createPage(): Promise<Page> {
    const p = await browser.newPage();
    applyDebugListeners(p);
    await initPage(p);
    return p;
  }

  // Create page pool — each page has maplibre-gl pre-loaded
  const pages = await Promise.all(
    Array.from({ length: poolSize }, () => createPage()),
  );

  const failed: FailedTest[] = [];
  const errored: ErroredTest[] = [];
  let passedCount = 0;
  let completed = 0;
  let nextIdx = 0;

  /** Mutable page reference so crash recovery can swap the underlying page. */
  interface PageRef { page: Page }

  async function recoverPage(ref: PageRef): Promise<void> {
    try { await ref.page.close(); } catch {}
    try {
      ref.page = await createPage();
    } catch {
      // Browser itself is dead — relaunch everything
      console.log("  Browser crashed, relaunching…");
      try { await browser.close(); } catch (e) { console.error("  Failed to close browser:", e); }
      browser = await puppeteer.launch({ headless: true, args: PUPPETEER_ARGS });
      ref.page = await createPage();
    }
  }

  async function processTest(ref: PageRef, style: TestStyle): Promise<void> {
    const id = style.metadata.test.id;
    const w = Math.floor(
      style.metadata.test.width * (style.metadata.test.pixelRatio || 1),
    );
    const h = Math.floor(
      style.metadata.test.height * (style.metadata.test.pixelRatio || 1),
    );
    const allowed = style.metadata.test.allowed;
    const threshold = style.metadata.test.threshold;

    try {
      // 1. optimise first so we can detect no-ops before rendering
      const optimised = await optimizeStyle(
        JSON.parse(JSON.stringify(style)) as TestStyle,
      );
      restoreTestMetadata(optimised, style);

      // 2. skip both renders if optimizer didn't change the style
      if (
        JSON.stringify(optimised.layers) === JSON.stringify(style.layers) &&
        JSON.stringify(optimised.sources) === JSON.stringify(style.sources)
      ) {
        completed++;
        passedCount++;
        console.log(`${completed}/${styles.length}: passed ${id} (no-op)`);
        return;
      }

      // 3. render original
      const origPixels = await renderStyle(ref.page, style);

      // 4. render optimised
      const optPixels = await renderStyle(ref.page, optimised);

      // 5. compare original vs optimised
      const { ratio, diffPng } = comparePixels(
        origPixels,
        optPixels,
        w,
        h,
        threshold,
      );

      completed++;

      if (ratio <= allowed) {
        passedCount++;
        console.log(`${completed}/${styles.length}: passed ${id}`);
      } else {
        const entry: FailedTest = { id, ratio };

        console.log(`  Bisecting passes for ${id}…`);
        const culprits = await bisectPasses(
          ref.page, style, origPixels, w, h, allowed, threshold,
        );
        entry.culpritPasses = culprits;
        if (culprits.length > 0) {
          console.log(`  Culprits: ${culprits.join(", ")}`);
        } else {
          console.log(`  No single pass triggers the failure (interaction effect)`);
        }

        failed.push(entry);
        console.log(
          `\x1b[31m${completed}/${styles.length}: FAILED ${id}  diff=${ratio.toFixed(6)}\x1b[0m`,
        );

        // write diff artefacts for inspection
        const dir = path.join(RESULTS_DIR, id.replace(/\//g, "__"));
        fs.mkdirSync(dir, { recursive: true });
        fs.writeFileSync(path.join(dir, "diff.png"), PNG.sync.write(diffPng));

        const origPng = new PNG({ width: w, height: h });
        origPng.data = Buffer.from(origPixels);
        fs.writeFileSync(
          path.join(dir, "original.png"),
          PNG.sync.write(origPng),
        );

        const optPng = new PNG({ width: w, height: h });
        optPng.data = Buffer.from(optPixels);
        fs.writeFileSync(
          path.join(dir, "optimised.png"),
          PNG.sync.write(optPng),
        );
      }
    } catch (err: unknown) {
      completed++;
      const msg = err instanceof Error ? err.message : String(err);
      errored.push({ id, error: msg });
      console.log(
        `\x1b[91m${completed}/${styles.length}: ERROR ${id}: ${msg}\x1b[0m`,
      );

      // If the page crashed, try to recover it
      if (
        msg.includes("Session closed") ||
        msg.includes("Target closed") ||
        msg.includes("Protocol error")
      ) {
        console.log("  Recovering page…");
        await recoverPage(ref);
      }
    }
  }

  // Worker-pool pattern: each page picks up the next test when it finishes
  async function worker(ref: PageRef): Promise<void> {
    while (true) {
      const idx = nextIdx++;
      if (idx >= styles.length) break;
      await processTest(ref, styles[idx]);
    }
  }

  await Promise.all(pages.map((p) => worker({ page: p })));

  await browser.close();
  server.close();

  // ── summary ────────────────────────────────────────────────────────────────
  console.log("\n── Summary ──");
  console.log(`  ${passedCount} passed`);
  if (failed.length) console.log(`  \x1b[31m${failed.length} failed\x1b[0m`);
  if (errored.length)
    console.log(`  \x1b[91m${errored.length} errored\x1b[0m`);
  console.log(`  ${styles.length} total`);

  if (failed.length || errored.length) {
    if (failed.length) {
      console.log("\nFailed tests:");
      for (const f of failed) {
        let line = `  ${f.id}  diff=${f.ratio.toFixed(6)}`;
        if (f.culpritPasses) {
          line += f.culpritPasses.length > 0
            ? `  culprits: ${f.culpritPasses.join(", ")}`
            : "  culprits: (interaction effect)";
        }
        console.log(line);
      }
    }
    if (errored.length) {
      console.log("\nErrored tests:");
      for (const e of errored) console.log(`  ${e.id}: ${e.error}`);
    }
    process.exit(1);
  }
  process.exit(0);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
