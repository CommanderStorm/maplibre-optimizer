/**
 * Benchmark scenarios: locations × camera animations.
 *
 * Each scenario pairs a geographic location with a camera animation type.
 * Animations are defined as keyframe sequences for map.easeTo().
 */

// ── Locations ────────────────────────────────────────────────────────────────

export interface Location {
  name: string;
  center: [number, number];
  zoom: number;
}

const locations: Record<string, Location> = {
  munich:     { name: "Munich",                center: [11.576, 48.137],   zoom: 12 },
  berlin:     { name: "Berlin",                center: [13.405, 52.520],   zoom: 12 },
  paris:      { name: "Paris",                 center: [2.349, 48.864],    zoom: 12 },
  newyork:    { name: "New York",              center: [-73.985, 40.748],  zoom: 12 },
  rome:       { name: "Rome",                  center: [12.496, 41.903],   zoom: 12 },
  tokyo:      { name: "Tokyo",                 center: [139.692, 35.690],  zoom: 12 },
  beijing:    { name: "Beijing",               center: [116.397, 39.909],  zoom: 12 },
  seoul:      { name: "Seoul",                 center: [126.978, 37.566],  zoom: 12 },
  cairo:      { name: "Cairo",                 center: [31.236, 30.044],   zoom: 12 },
  blackforest:{ name: "Black Forest",          center: [8.200, 48.000],    zoom: 11 },
  amazon:     { name: "Amazon",                center: [-60.025, -3.119],  zoom: 10 },
  swiss:      { name: "Swiss Alps",            center: [8.232, 46.818],    zoom: 11 },
  sahara:     { name: "Sahara",                center: [2.000, 24.000],    zoom: 8  },
  venice:     { name: "Venice",                center: [12.338, 45.434],   zoom: 13 },
  stockholm:  { name: "Stockholm Archipelago", center: [18.520, 59.400],   zoom: 11 },
};

// ── Camera animation keyframes ───────────────────────────────────────────────

export interface Keyframe {
  center?: [number, number];
  zoom?: number;
  bearing?: number;
  pitch?: number;
  duration: number;
}

export type AnimationFactory = (loc: Location) => Keyframe[];

const KEYFRAME_DURATION = 750;

const animations: Record<string, AnimationFactory> = {
  zigzag(loc) {
    // 8 steps alternating zoom 10/14, moving along a bearing
    const steps: Keyframe[] = [];
    const [lng, lat] = loc.center;
    for (let i = 0; i < 8; i++) {
      const offset = (i + 1) * 0.005;
      steps.push({
        center: [lng + offset, lat + (i % 2 === 0 ? offset : -offset)],
        zoom: i % 2 === 0 ? 10 : 14,
        bearing: i * 15,
        duration: KEYFRAME_DURATION,
      });
    }
    return steps;
  },

  spiral(loc) {
    // 12 steps tracing a circle at constant zoom, rotating bearing 0-360
    const steps: Keyframe[] = [];
    const [lng, lat] = loc.center;
    const radius = 0.01;
    for (let i = 0; i < 12; i++) {
      const angle = (i / 12) * 2 * Math.PI;
      steps.push({
        center: [lng + radius * Math.cos(angle), lat + radius * Math.sin(angle)],
        zoom: loc.zoom,
        bearing: (i / 12) * 360,
        duration: KEYFRAME_DURATION,
      });
    }
    return steps;
  },

  zoomdrill(_loc) {
    // zoom 4→8→12→16→12→8→4, stationary
    return [4, 8, 12, 16, 12, 8, 4].map((z) => ({
      zoom: z,
      duration: KEYFRAME_DURATION,
    }));
  },

  pansweep(loc) {
    // 6 steps at constant zoom sweeping across a region
    const steps: Keyframe[] = [];
    const [lng, lat] = loc.center;
    for (let i = 0; i < 6; i++) {
      steps.push({
        center: [lng + (i - 3) * 0.02, lat + (i % 2 === 0 ? 0.01 : -0.01)],
        zoom: loc.zoom,
        duration: KEYFRAME_DURATION,
      });
    }
    return steps;
  },

  bearingspin(_loc) {
    // 8 steps rotating bearing 0-360 at 60 pitch, stationary
    return Array.from({ length: 8 }, (_, i) => ({
      bearing: (i / 8) * 360,
      pitch: 60,
      duration: KEYFRAME_DURATION,
    }));
  },
};

// ── Scenario definition ──────────────────────────────────────────────────────

export interface Scenario {
  id: string;
  location: Location;
  animationType: string;
  keyframes: Keyframe[];
}

/** Curated scenario matrix (18 scenarios). */
const scenarioMatrix: [string, string][] = [
  ["munich", "zigzag"],
  ["munich", "spiral"],
  ["paris", "zigzag"],
  ["paris", "zoomdrill"],
  ["tokyo", "spiral"],
  ["tokyo", "zigzag"],
  ["beijing", "pansweep"],
  ["newyork", "zoomdrill"],
  ["newyork", "bearingspin"],
  ["rome", "spiral"],
  ["blackforest", "zigzag"],
  ["blackforest", "pansweep"],
  ["amazon", "spiral"],
  ["swiss", "zoomdrill"],
  ["venice", "spiral"],
  ["cairo", "zigzag"],
  ["sahara", "pansweep"],
  ["stockholm", "bearingspin"],
];

export function getAllScenarios(): Scenario[] {
  return scenarioMatrix.map(([locKey, animKey]) => {
    const loc = locations[locKey];
    const animFactory = animations[animKey];
    return {
      id: `${locKey}-${animKey}`,
      location: loc,
      animationType: animKey,
      keyframes: animFactory(loc),
    };
  });
}

export function filterScenarios(scenarios: Scenario[], filters: string[]): Scenario[] {
  if (filters.length === 0) return scenarios;
  return scenarios.filter((s) => filters.some((f) => s.id.includes(f)));
}
