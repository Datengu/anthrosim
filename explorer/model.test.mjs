import test from "node:test";
import assert from "node:assert/strict";
import {
  countEvents,
  eventsForEntity,
  genealogyForPerson,
  mapValues,
  reconstructState,
  snapshotAtOrBefore,
  validateBundle,
} from "./model.mjs";

function fixture() {
  const events = [
    { sequence: 1, day: 365, provenance: "authoritative", event: {
      type: "birth", person: 3, femaleParent: 1, maleParent: 2, household: 1, cell: 1, reproductiveSex: "female",
    } },
    { sequence: 2, day: 730, provenance: "authoritative", event: {
      type: "householdMigration", household: 1, peopleMoved: 3, origin: 1, destination: 4, distanceCells: 2,
    } },
    { sequence: 3, day: 1095, provenance: "authoritative", event: {
      type: "death", person: 2, household: 1, cell: 4, cause: "demographic_mortality", conditionPermille: 900,
      probabilityPerMillion: 10000,
    } },
  ];
  const snapshots = [
    { schemaVersion: 1, day: 365, provenance: "derived", population: { livingPopulation: 3, birthsSinceStart: 1, deathsSinceStart: 0 }, resources: { unmetNeed: 0 }, migration: { movesCompleted: 0 } },
    { schemaVersion: 1, day: 1095, provenance: "derived", population: { livingPopulation: 2, birthsSinceStart: 1, deathsSinceStart: 1 }, resources: { unmetNeed: 0 }, migration: { movesCompleted: 1 } },
  ];
  return {
    manifest: {
      schemaVersion: 7,
      endTime: 1095,
      artifactSchemas: { manifest: 7, events: 1, metrics: 1, checkpoint: 1, world: 1, population: 3, resources: 1, migration: 1 },
      world: { width: 2, height: 2 },
      population: { initialPopulation: 2, birthsSinceStart: 1, deathsSinceStart: 1, livingPopulation: 2 },
      resources: { unmetNeed: 0 },
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
    events: { schemaVersion: 1, events },
    metrics: { schemaVersion: 1, cadence: "annual_boundary_plus_terminal", snapshots },
    checkpoint: {
      schemaVersion: 1, completedYears: 3, time: 1095,
      resources: { schemaVersion: 1, cellFoodStock: [10, 20, 30, 40] },
    },
  };
}

test("bundle validation reconciles manifest, events, metrics and schemas", () => {
  const result = validateBundle(fixture());
  assert.deepEqual(result.eventCounts, { birth: 1, death: 1, householdMigration: 1 });
  assert.equal(result.durationYears, 3);
});

test("schema mismatch is rejected instead of silently interpreted", () => {
  const bundle = fixture();
  bundle.events.schemaVersion = 2;
  assert.throws(() => validateBundle(bundle), /events schema 2/);
});

test("timeline reconstruction applies birth, household migration and death in sequence", () => {
  const bundle = fixture();
  const year1 = reconstructState(bundle, 365);
  assert.equal(year1.people.get(3).location, 1);
  assert.equal(year1.cellResidents.get(1).length, 3);

  const year2 = reconstructState(bundle, 730);
  assert.equal(year2.people.get(1).location, 4);
  assert.equal(year2.people.get(2).location, 4);
  assert.equal(year2.people.get(3).location, 4);
  assert.equal(year2.cellResidents.get(4).length, 3);

  const year3 = reconstructState(bundle, 1095);
  assert.equal(year3.people.get(2).alive, false);
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

test("map overlays keep reconstructed population separate from authoritative terrain and final stock", () => {
  const bundle = fixture();
  const state = reconstructState(bundle, 1095);
  assert.deepEqual(mapValues(bundle, state, "population"), [0, 0, 0, 2]);
  assert.deepEqual(mapValues(bundle, state, "productivity"), [100, 200, 300, 400]);
  assert.deepEqual(mapValues(bundle, state, "finalFood"), [10, 20, 30, 40]);
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
