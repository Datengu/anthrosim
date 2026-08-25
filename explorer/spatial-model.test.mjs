import test from "node:test";
import assert from "node:assert/strict";
import {
  safeFiniteSpatialValues,
  spatialCellDetails,
  spatialMapValues,
  spatialOverlayDescription,
  spatialOverlayOptions,
  validateSpatialArtifacts,
} from "./spatial-model.mjs";

function fixture() {
  const landscape = {
    schemaVersion: 1,
    width: 2,
    height: 1,
    geometry: {
      originX: 100,
      originY: 200,
      cellSizeX: 25,
      cellSizeY: 25,
      coordinateUnit: "metre",
      spatialReference: "EPSG:27700",
    },
    layers: [
      {
        layerId: "water",
        role: "water_accessibility",
        unit: "normalized_index",
        valueDomain: { min: 0, max: 1000 },
        evidenceInputId: "water-source",
        values: [100, null],
      },
      {
        layerId: "terrain",
        role: "terrain_traversal",
        unit: "normalized_index",
        valueDomain: { min: 0, max: 1000 },
        values: [250, 750],
      },
    ],
  };
  const report = {
    schemaVersion: 2,
    provenance: "derived",
    source: {
      seed: 7,
      endDay: 730,
      runStateDigest64: "18446744073709551615",
      landscapeIdentity: "landscape-v1-example",
    },
    semantics: {
      populationLocationBasis: "persistent_residence",
      occupancyIncludesTemporaryVisitors: false,
      occupancyIncludesTransit: false,
      birthCellAttribution: "persistent_residence",
      deathCellAttribution: "persistent_residence",
      physicalPresenceCompanionArtifact: "temporary-observability.json",
    },
    width: 2,
    height: 1,
    normalizedLayers: [
      { layerId: "water", role: "water_accessibility", unit: "normalized_index", nodataCells: 1 },
      { layerId: "terrain", role: "terrain_traversal", unit: "normalized_index", nodataCells: 0 },
    ],
    cells: [
      {
        cell: 1,
        gridX: 0,
        gridY: 0,
        derived: {
          provenance: "derived",
          occupancyFractionPermille: 1000,
          livingPersonDays: 900,
          terminalLivingPopulation: 2,
          resourceScarcityDeaths: 1,
          migrationPeopleOut: 3,
        },
      },
      {
        cell: 2,
        gridX: 1,
        gridY: 0,
        derived: {
          provenance: "derived",
          occupancyFractionPermille: 500,
          livingPersonDays: 365,
          terminalLivingPopulation: 1,
          resourceScarcityDeaths: 0,
          migrationPeopleOut: 0,
        },
      },
    ],
    summary: { provenance: "derived" },
  };
  return {
    world: { width: 2, height: 1 },
    landscape,
    landscapeManifest: {
      landscape: { width: 2, height: 1, landscapeIdentity: "landscape-v1-example" },
    },
    spatialMechanisms: null,
    spatialObservability: report,
    temporaryObservability: {
      schemaVersion: 1,
      source: { seed: 7, endDay: 730, runStateDigest64: "18446744073709551615" },
      summary: { visitorPersonDays: 12, peakVisitors: 3 },
      cells: [
        { cell: 1, persistentResidencePersonDays: 900, atResidencePersonDays: 888, visitorPersonDays: 4, visitorHouseholdDays: 2, arrivals: 1, returnDepartures: 1, peakVisitors: 2 },
        { cell: 2, persistentResidencePersonDays: 365, atResidencePersonDays: 365, visitorPersonDays: 8, visitorHouseholdDays: 4, arrivals: 2, returnDepartures: 2, peakVisitors: 3 },
      ],
    },
  };
}

const runInfo = {
  seed: 7,
  endTime: 730,
  stateDigest64: "18446744073709551615",
};

test("spatial artifact validation reconciles grid, run and landscape identities", () => {
  const result = validateSpatialArtifacts(fixture(), runInfo);
  assert.equal(result.available, true);
  assert.equal(result.cellCount, 2);
  assert.equal(result.hasObservability, true);
});

test("spatial validation rejects derived output attached to another run", () => {
  const bundle = fixture();
  bundle.spatialObservability.source.seed = 8;
  assert.throws(() => validateSpatialArtifacts(bundle, runInfo), /seed disagrees/);
});

test("normalized layers preserve nodata instead of silently coercing it", () => {
  const bundle = fixture();
  assert.deepEqual(spatialMapValues(bundle, "landscape:water"), [100, null]);
  assert.deepEqual(safeFiniteSpatialValues([100, null]), [100]);
  assert.match(spatialOverlayDescription(bundle, "landscape:water"), /Nodata remains missing/);
});

test("derived spatial overlays are read from the machine-readable report", () => {
  const bundle = fixture();
  assert.deepEqual(spatialMapValues(bundle, "derived:occupancyPersistence"), [1000, 500]);
  assert.deepEqual(spatialMapValues(bundle, "derived:personDays"), [900, 365]);
  assert.deepEqual(spatialMapValues(bundle, "derived:terminalPopulation"), [2, 1]);
  assert.match(spatialOverlayDescription(bundle, "derived:personDays"), /temporary visitors and transit are excluded/);
});

test("overlay options keep normalized input and derived observables visibly distinct", () => {
  const options = spatialOverlayOptions(fixture());
  assert.equal(options[0].provenance, "normalized_input");
  assert.equal(options[2].provenance, "derived");
  assert(options.some((option) => option.value === "landscape:terrain"));
  assert(options.some((option) => option.value === "derived:occupancyPersistence"));
});

test("cell details retain exact source-layer values beside derived metrics", () => {
  const details = spatialCellDetails(fixture(), 2);
  assert.equal(details.layers[0].value, null);
  assert.equal(details.layers[1].value, 750);
  assert.equal(details.report.derived.terminalLivingPopulation, 1);
  assert.equal(details.temporaryReport.visitorPersonDays, 8);
});

test("spatial semantics reject ambiguous location-basis metadata", () => {
  const bundle = fixture();
  bundle.spatialObservability.semantics.populationLocationBasis = "physical_presence";
  assert.throws(() => validateSpatialArtifacts(bundle, runInfo), /persistent residence/);
});
