#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { mapValues, parseLosslessJson, reconstructState, validateBundle } from "../explorer/model.mjs";

const FILES = {
  manifest: "manifest.json",
  world: "world.json",
  initialPopulation: "initial-population.json",
  events: "events.json",
  metrics: "metrics.json",
  checkpoint: "checkpoint.json",
};

function fail(message) {
  throw new Error(message);
}

async function main() {
  const runDir = resolve(process.argv[2] ?? "");
  if (!process.argv[2]) fail("usage: node scripts/validate-explorer-bundle.mjs <run-dir>");

  const bundle = {};
  for (const [key, name] of Object.entries(FILES)) {
    bundle[key] = parseLosslessJson(await readFile(resolve(runDir, name), "utf8"));
  }

  const summary = validateBundle(bundle);
  const finalState = reconstructState(bundle, bundle.manifest.endTime);
  const living = [...finalState.people.values()].filter((person) => person.alive).length;
  if (living !== bundle.manifest.population.livingPopulation) {
    fail(`reconstructed living population ${living} != manifest ${bundle.manifest.population.livingPopulation}`);
  }
  if (finalState.cellResidents.size !== bundle.manifest.population.livingOccupiedCellCount) {
    fail(`reconstructed occupied cells ${finalState.cellResidents.size} != manifest ${bundle.manifest.population.livingOccupiedCellCount}`);
  }
  if (finalState.people.size !== bundle.manifest.population.personRecords) {
    fail(`reconstructed person records ${finalState.people.size} != manifest ${bundle.manifest.population.personRecords}`);
  }

  const checkpointPopulation = bundle.checkpoint.population;
  for (let index = 0; index < checkpointPopulation.locations.length; index += 1) {
    const person = finalState.people.get(index + 1);
    if (!person) fail(`checkpoint person ${index + 1} missing from reconstructed state`);
    if (person.location !== Number(checkpointPopulation.locations[index])) {
      fail(`person ${index + 1} reconstructed location ${person.location} != final checkpoint ${checkpointPopulation.locations[index]}`);
    }
    if (person.household !== Number(checkpointPopulation.households[index])) {
      fail(`person ${index + 1} reconstructed household ${person.household} != final checkpoint ${checkpointPopulation.households[index]}`);
    }
  }

  const finalFood = mapValues(bundle, finalState, "finalFood").reduce((sum, value) => sum + value, 0);
  if (finalFood !== bundle.manifest.resources.finalFoodStock) {
    fail(`final cell food sum ${finalFood} != manifest ${bundle.manifest.resources.finalFoodStock}`);
  }

  console.log(`M6 bundle validation passed: ${runDir}`);
  console.log(`  ${summary.durationYears} years, ${summary.cellCount} cells, ${finalState.people.size} person records`);
  console.log(`  ${summary.eventCounts.birth} births, ${summary.eventCounts.death} deaths, ${summary.eventCounts.householdMigration} migrations`);
  console.log(`  final living=${living}, occupiedCells=${finalState.cellResidents.size}, foodStock=${finalFood}`);
}

main().catch((error) => {
  console.error(error.stack ?? error);
  process.exitCode = 1;
});
