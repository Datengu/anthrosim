#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import {
  eventsForEntity,
  inspectionTargetForEvent,
  parseLosslessJson,
  reconstructState,
  validateBundle,
} from "../explorer/model.mjs";

const REQUIRED_FILES = {
  world: "world.json",
  initialPopulation: "initial-population.json",
  events: "events.json",
  metrics: "metrics.json",
  checkpoint: "checkpoint.json",
};

function fail(message) {
  throw new Error(message);
}

function validateEventInspectionSemantics() {
  const birthTarget = inspectionTargetForEvent({ type: "birth", person: 17, household: 4 });
  if (birthTarget?.kind !== "person" || birthTarget.id !== 17) fail("birth event does not route to its person inspector");

  const migrationTarget = inspectionTargetForEvent({ type: "householdMigration", household: 4 });
  if (migrationTarget?.kind !== "household" || migrationTarget.id !== 4) fail("permanent migration event does not route to its household inspector");

  const temporaryTarget = inspectionTargetForEvent({ type: "temporaryJourneyDeparted", household: 9 });
  if (temporaryTarget?.kind !== "household" || temporaryTarget.id !== 9) fail("temporary-mobility event does not route to its household inspector");

  if (inspectionTargetForEvent({ type: "unknown" }) !== null) fail("entity-free event unexpectedly produced an inspector target");

  const residenceFixture = {
    events: { events: [
      { event: { type: "temporaryJourneyCompleted", household: 9, residence: 23 } },
    ] },
  };
  if (eventsForEntity(residenceFixture, { cell: 23 }).length !== 1) fail("M9 residence cell is not included in Explorer cell-event filtering");
}
async function readArtifact(path, { optional = false } = {}) {
  try {
    return parseLosslessJson(await readFile(path, "utf8"));
  } catch (error) {
    if (optional && error?.code === "ENOENT") return null;
    throw error;
  }
}

async function main() {
  validateEventInspectionSemantics();
  const runDir = resolve(process.argv[2] ?? "");
  if (!process.argv[2]) fail("usage: node scripts/validate-explorer-bundle.mjs <run-dir>");

  const bundle = {};
  for (const [key, name] of Object.entries(REQUIRED_FILES)) {
    bundle[key] = await readArtifact(resolve(runDir, name));
  }
  bundle.manifest = await readArtifact(resolve(runDir, "manifest.json"), { optional: true });

  const summary = validateBundle(bundle);
  const finalState = reconstructState(bundle, summary.endTime);
  const living = [...finalState.people.values()].filter((person) => person.alive).length;
  if (living !== summary.livingPopulation) {
    fail(`reconstructed living population ${living} != authoritative ${summary.livingPopulation}`);
  }
  if (summary.occupiedCells !== null && finalState.cellResidents.size !== summary.occupiedCells) {
    fail(`reconstructed occupied cells ${finalState.cellResidents.size} != authoritative ${summary.occupiedCells}`);
  }
  if (finalState.people.size !== summary.personRecords) {
    fail(`reconstructed person records ${finalState.people.size} != authoritative ${summary.personRecords}`);
  }

  const checkpointPopulation = bundle.checkpoint.population;
  for (let index = 0; index < checkpointPopulation.locations.length; index += 1) {
    const person = finalState.people.get(index + 1);
    if (!person) fail(`checkpoint person ${index + 1} missing from reconstructed state`);
    if (person.location !== Number(checkpointPopulation.locations[index])) {
      fail(`person ${index + 1} reconstructed location ${person.location} != checkpoint ${checkpointPopulation.locations[index]}`);
    }
    if (person.household !== Number(checkpointPopulation.households[index])) {
      fail(`person ${index + 1} reconstructed household ${person.household} != checkpoint ${checkpointPopulation.households[index]}`);
    }
  }

  const finalFood = bundle.checkpoint.resources.cellFoodStock.reduce((sum, value) => sum + BigInt(value), 0n);
  if (summary.finalFoodStock !== null && finalFood !== BigInt(summary.finalFoodStock)) {
    fail(`checkpoint cell food sum ${finalFood} != authoritative ${summary.finalFoodStock}`);
  }

  console.log(`M6 ${summary.kind} bundle validation passed: ${runDir}`);
  console.log(`  boundary=${summary.durationYears} years of configured ${summary.configuredDurationYears}, ${summary.cellCount} cells, ${finalState.people.size} person records`);
  console.log(`  ${summary.eventCounts.birth} births, ${summary.eventCounts.death} deaths, ${summary.eventCounts.householdMigration} migrations`);
  console.log(`  living=${living}, occupiedCells=${finalState.cellResidents.size}, checkpointFoodStock=${finalFood}`);
}

main().catch((error) => {
  console.error(error.stack ?? error);
  process.exitCode = 1;
});
