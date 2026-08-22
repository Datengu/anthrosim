import test from "node:test";
import assert from "node:assert/strict";
import {
  countEvents,
  eventsForEntity,
  genealogyForPerson,
  mapValues,
  parseLosslessJson,
  reconstructState,
  snapshotAtOrBefore,
  validateBundle,
} from "./model.mjs";

function fixture() {
  const eventRecords = [
    { sequence: 1, day: 365, provenance: "authoritative", event: {
      type: "birth", person: 3, female_parent: 1, male_parent: 2, household: 1, cell: 1, reproductive_sex: "female",
    } },
    { sequence: 2, day: 730, provenance: "authoritative", event: {
      type: "householdMigration", household: 1, people_moved: 3, origin: 1, destination: 4, distance_cells: 2,
    } },
    { sequence: 3, day: 1095, provenance: "authoritative", event: {
      type: "death", person: 2, household: 1, cell: 4, cause: "demographic_mortality", condition_permille: 900,
      probability_per_million: 10000,
    } },
  ];
  const snapshots = [
    {
      schemaVersion: 1, day: 365, provenance: "derived",
      population: { livingPopulation: 3, personRecords: 3, birthsSinceStart: 1, deathsSinceStart: 0, livingOccupiedCellCount: 1 },
      resources: { periodsProcessed: 4, unmetNeed: 0, finalFoodStock: 110 },
      migration: { decisionBoundaries: 4, movesCompleted: 0 },
    },
    {
      schemaVersion: 1, day: 1095, provenance: "derived",
      population: { livingPopulation: 2, personRecords: 3, birthsSinceStart: 1, deathsSinceStart: 1, livingOccupiedCellCount: 1 },
      resources: { periodsProcessed: 12, unmetNeed: 0, finalFoodStock: 100 },
      migration: { decisionBoundaries: 12, movesCompleted: 1 },
    },
  ];
  const events = { schemaVersion: 1, events: eventRecords };
  const metrics = { schemaVersion: 1, cadence: "annual_boundary_plus_terminal", snapshots };
  const checkpoint = {
    schemaVersion: 1,
    modelVersion: "0.1.0",
    experiment: {
      schemaVersion: 6,
      seed: 9,
      durationYears: 6,
      world: { schemaVersion: 1, width: 2, height: 2 },
      population: { schemaVersion: 3, initialPopulation: 2 },
    },
    time: 1095,
    completedYears: 3,
    population: {
      schemaVersion: 3,
      initialPopulation: 2,
      birthsSinceStart: 1,
      deathsSinceStart: 1,
      birthDays: [-7300, -7000, 365],
      locations: [4, 4, 4],
      households: [1, 1, 1],
      conditionPermille: [1000, 900, 1000],
    },
    resources: { schemaVersion: 1, periodsProcessed: 12, unmetNeed: 0, cellFoodStock: [10, 20, 30, 40] },
    migration: { schemaVersion: 1, movesCompleted: 1, decisionBoundaries: 12 },
    events,
    metrics,
    stateDigest64: "18446744073709551615",
  };
  return {
    manifest: {
      schemaVersion: 7,
      endTime: 1095,
      stateDigest64: "18446744073709551615",
      artifactSchemas: { manifest: 7, events: 1, metrics: 1, checkpoint: 1, world: 1, population: 3, resources: 1, migration: 1 },
      world: { width: 2, height: 2 },
      population: {
        initialPopulation: 2, personRecords: 3, birthsSinceStart: 1, deathsSinceStart: 1,
        livingPopulation: 2, livingOccupiedCellCount: 1,
      },
      resources: { unmetNeed: 0, finalFoodStock: 100 },
      migration: { movesCompleted: 1 },
      statistics: { authoritativeEventCount: 3, metricSnapshotCount: 2 },
    },
    world: {
      schemaVersion: 1, width: 2, height: 2,
      cells: [
        { baseProductivity: 100, waterAccess: 200, movementCost: 300 },
        { baseProductivity: 200, waterAccess: 300, movementCost: 400 },
        { baseProductivity: 300, waterAccess: 400, movementCost: 500 },
        { baseProductivity: 400, waterAccess: 500, movementCost: 600 },
      ],
    },
    initialPopulation: {
      schemaVersion: 3, initialPopulation: 2,
      birthDays: [-7300, -7000],
      reproductiveSexes: ["female", "male"],
      locations: [1, 1], households: [1, 1], femaleParents: [0, 0], maleParents: [0, 0],
      conditionPermille: [1000, 1000], householdLocations: [1],
    },
    events,
    metrics,
    checkpoint,
  };
}

function pausedFixture() {
  const bundle = fixture();
  bundle.manifest = null;
  return bundle;
}

test("lossless parser preserves unsafe JSON integers as exact decimal strings", () => {
  const parsed = parseLosslessJson('{"small":42,"digest":18446744073709551615,"negative":-9223372036854775808,"text":"18446744073709551615"}');
  assert.equal(parsed.small, 42);
  assert.equal(parsed.digest, "18446744073709551615");
  assert.equal(parsed.negative, "-9223372036854775808");
  assert.equal(parsed.text, "18446744073709551615");
});

test("completed bundle validation reconciles manifest, events, metrics and schemas", () => {
  const result = validateBundle(fixture());
  assert.equal(result.kind, "completed");
  assert.deepEqual(result.eventCounts, { birth: 1, death: 1, householdMigration: 1 });
  assert.equal(result.durationYears, 3);
  assert.equal(result.personRecords, 3);
});

test("paused bundle validation uses checkpoint as the authoritative boundary without a manifest", () => {
  const result = validateBundle(pausedFixture());
  assert.equal(result.kind, "paused");
  assert.equal(result.durationYears, 3);
  assert.equal(result.configuredDurationYears, 6);
  assert.equal(result.livingPopulation, 2);
  assert.equal(result.occupiedCells, 1);
  assert.equal(result.finalFoodStock, 100);
  assert.equal(result.stateDigest64, "18446744073709551615");
});

test("paused bundle rejects event history that disagrees with embedded checkpoint history", () => {
  const bundle = pausedFixture();
  bundle.events = structuredClone(bundle.events);
  bundle.events.events.pop();
  assert.throws(() => validateBundle(bundle), /events artifact disagrees/);
});

test("schema mismatch is rejected instead of silently interpreted", () => {
  const bundle = fixture();
  bundle.events.schemaVersion = 2;
  assert.throws(() => validateBundle(bundle), /events schema 2/);
});

test("timeline reconstruction applies serialized birth, household migration and death payloads in sequence", () => {
  const bundle = fixture();
  const year1 = reconstructState(bundle, 365);
  assert.equal(year1.people.get(3).location, 1);
  assert.equal(year1.people.get(3).reproductiveSex, "female");
  assert.equal(year1.cellResidents.get(1).length, 3);

  const year2 = reconstructState(bundle, 730);
  assert.equal(year2.people.get(1).location, 4);
  assert.equal(year2.people.get(2).location, 4);
  assert.equal(year2.people.get(3).location, 4);
  assert.equal(year2.cellResidents.get(4).length, 3);

  const year3 = reconstructState(bundle, 1095);
  assert.equal(year3.people.get(2).alive, false);
  assert.equal(year3.people.get(2).conditionPermille, 900);
  assert.deepEqual(year3.cellResidents.get(4), [1, 3]);
});

test("genealogy and event filters trace entities back to authoritative events", () => {
  const bundle = fixture();
  const state = reconstructState(bundle, 1095);
  const genealogy = genealogyForPerson(state, 1);
  assert.deepEqual(genealogy.children, [3]);
  assert.equal(eventsForEntity(bundle, { person: 3 }).length, 1);
  assert.equal(eventsForEntity(bundle, { household: 1 }).length, 3);
  assert.equal(eventsForEntity(bundle, { cell: 4 }).length, 2);
});

test("map overlays keep reconstructed population separate from authoritative terrain and checkpoint stock", () => {
  const bundle = fixture();
  const state = reconstructState(bundle, 1095);
  assert.deepEqual(mapValues(bundle, state, "population"), [0, 0, 0, 2]);
  assert.deepEqual(mapValues(bundle, state, "productivity"), [100, 200, 300, 400]);
  assert.deepEqual(mapValues(bundle, state, "finalFood"), [10, 20, 30, 40]);
});

test("numeric map overlays reject integers that cannot be represented exactly", () => {
  const bundle = fixture();
  bundle.checkpoint.resources.cellFoodStock[0] = "18446744073709551615";
  const state = reconstructState(bundle, 1095);
  assert.throws(() => mapValues(bundle, state, "finalFood"), /exact integer range/);
});

test("snapshot lookup returns the latest recorded derived observation at or before a day", () => {
  const bundle = fixture();
  assert.equal(snapshotAtOrBefore(bundle.metrics, 500).day, 365);
  assert.equal(snapshotAtOrBefore(bundle.metrics, 1095).day, 1095);
  assert.equal(snapshotAtOrBefore(bundle.metrics, 100), null);
});

test("countEvents ignores unknown future event kinds without corrupting known totals", () => {
  const counts = countEvents([...fixture().events.events, { event: { type: "futureEvent" } }]);
  assert.deepEqual(counts, { birth: 1, death: 1, householdMigration: 1 });
});
