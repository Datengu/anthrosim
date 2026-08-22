const REQUIRED_ARTIFACTS = [
  "world",
  "initialPopulation",
  "events",
  "metrics",
  "checkpoint",
];

const COMPLETED_ARTIFACT_SCHEMA_KEYS = {
  manifest: "manifest",
  world: "world",
  initialPopulation: "population",
  events: "events",
  metrics: "metrics",
  checkpoint: "checkpoint",
};

const MAX_SAFE_BIGINT = BigInt(Number.MAX_SAFE_INTEGER);

function asNumber(value, label) {
  const number = Number(value);
  if (!Number.isFinite(number)) throw new Error(`${label} is not numeric`);
  if (Number.isInteger(number) && !Number.isSafeInteger(number)) {
    throw new Error(`${label} exceeds JavaScript's exact integer range`);
  }
  return number;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function exactJsonEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function terminalSnapshot(metrics, endTime) {
  const terminal = metrics.snapshots?.at(-1) ?? null;
  if (terminal) assert(terminal.day === endTime, "terminal metric day disagrees with run boundary");
  return terminal;
}

function runEndTime(bundle) {
  return asNumber(bundle.manifest?.endTime ?? bundle.checkpoint.time, "run boundary time");
}

export function parseLosslessJson(text) {
  let output = "";
  let index = 0;
  let inString = false;
  let escaped = false;

  while (index < text.length) {
    const character = text[index];
    if (inString) {
      output += character;
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === '"') inString = false;
      index += 1;
      continue;
    }

    if (character === '"') {
      inString = true;
      output += character;
      index += 1;
      continue;
    }

    if (character === "-" || (character >= "0" && character <= "9")) {
      let end = index + 1;
      while (end < text.length && !/[\s,\]}]/.test(text[end])) end += 1;
      const token = text.slice(index, end);
      if (/^-?\d+$/.test(token)) {
        const integer = BigInt(token);
        if (integer > MAX_SAFE_BIGINT || integer < -MAX_SAFE_BIGINT) {
          output += JSON.stringify(token);
          index = end;
          continue;
        }
      }
      output += token;
      index = end;
      continue;
    }

    output += character;
    index += 1;
  }

  return JSON.parse(output);
}

export function validateBundle(bundle) {
  for (const name of REQUIRED_ARTIFACTS) {
    assert(bundle?.[name], `missing ${name} artifact`);
  }

  const { manifest, world, initialPopulation, events, metrics, checkpoint } = bundle;
  const kind = manifest ? "completed" : "paused";
  const endTime = runEndTime(bundle);
  assert(checkpoint.completedYears * 365 === checkpoint.time,
    "checkpoint is not at the declared completed annual boundary");
  assert(checkpoint.time === endTime, "checkpoint time disagrees with run boundary");
  assert(world.cells.length === world.width * world.height, "world cell count does not match dimensions");

  if (manifest) {
    assert(manifest.schemaVersion >= 7, `manifest schema ${manifest.schemaVersion} predates M5/M6 observability`);
    assert(manifest.artifactSchemas, "manifest does not declare artifact schemas");
    for (const [artifact, schemaKey] of Object.entries(COMPLETED_ARTIFACT_SCHEMA_KEYS)) {
      const expected = manifest.artifactSchemas[schemaKey];
      const actual = bundle[artifact].schemaVersion;
      assert(expected === actual, `${artifact} schema ${actual} does not match manifest declaration ${expected}`);
    }
    assert(world.width === manifest.world.width && world.height === manifest.world.height,
      "world dimensions disagree with manifest");
    assert(initialPopulation.initialPopulation === manifest.population.initialPopulation,
      "initial population disagrees with manifest");
  } else {
    assert(checkpoint.schemaVersion >= 1, "paused checkpoint has no supported version");
    assert(checkpoint.experiment, "paused checkpoint is missing experiment configuration");
    assert(world.schemaVersion === checkpoint.experiment.world.schemaVersion,
      "world schema disagrees with paused checkpoint experiment");
    assert(initialPopulation.schemaVersion === checkpoint.population.schemaVersion,
      "initial population schema disagrees with paused checkpoint population");
    assert(world.width === checkpoint.experiment.world.width && world.height === checkpoint.experiment.world.height,
      "world dimensions disagree with paused checkpoint experiment");
    assert(initialPopulation.initialPopulation === checkpoint.experiment.population.initialPopulation,
      "initial population disagrees with paused checkpoint experiment");
    assert(events.schemaVersion === checkpoint.events.schemaVersion,
      "events schema disagrees with paused checkpoint");
    assert(metrics.schemaVersion === checkpoint.metrics.schemaVersion,
      "metrics schema disagrees with paused checkpoint");
    assert(exactJsonEqual(events, checkpoint.events), "events artifact disagrees with paused checkpoint history");
    assert(exactJsonEqual(metrics, checkpoint.metrics), "metrics artifact disagrees with paused checkpoint history");
  }

  const counts = countEvents(events.events);
  const expectedBirths = manifest?.population.birthsSinceStart ?? checkpoint.population.birthsSinceStart;
  const expectedDeaths = manifest?.population.deathsSinceStart ?? checkpoint.population.deathsSinceStart;
  const expectedMoves = manifest?.migration.movesCompleted ?? checkpoint.migration.movesCompleted;
  assert(counts.birth === expectedBirths,
    `birth events ${counts.birth} disagree with authoritative total ${expectedBirths}`);
  assert(counts.death === expectedDeaths,
    `death events ${counts.death} disagree with authoritative total ${expectedDeaths}`);
  assert(counts.householdMigration === expectedMoves,
    `migration events ${counts.householdMigration} disagree with authoritative total ${expectedMoves}`);

  let previousSequence = 0;
  let previousDay = 0;
  for (const record of events.events) {
    assert(record.provenance === "authoritative", `event ${record.sequence} is not authoritative`);
    assert(record.sequence === previousSequence + 1, `event sequence is discontinuous at ${record.sequence}`);
    assert(record.day >= previousDay, `event sequence ${record.sequence} moves backwards in time`);
    previousSequence = record.sequence;
    previousDay = record.day;
  }

  for (const snapshot of metrics.snapshots) {
    assert(snapshot.provenance === "derived", `metric snapshot at day ${snapshot.day} is not marked derived`);
  }

  const personRecords = manifest?.population.personRecords ?? checkpoint.population.birthDays.length;
  const livingPopulation = manifest?.population.livingPopulation ??
    initialPopulation.initialPopulation + expectedBirths - expectedDeaths;
  const terminal = terminalSnapshot(metrics, endTime);
  const occupiedCells = manifest?.population.livingOccupiedCellCount ?? terminal?.population.livingOccupiedCellCount ?? null;
  const finalFoodStock = manifest?.resources.finalFoodStock ?? terminal?.resources.finalFoodStock ?? null;

  if (manifest) {
    assert(events.events.length === manifest.statistics.authoritativeEventCount,
      "event log length disagrees with manifest authoritative event count");
    assert(metrics.snapshots.length === manifest.statistics.metricSnapshotCount,
      "metric snapshot count disagrees with manifest");
  }

  if (terminal) {
    assert(terminal.population.livingPopulation === livingPopulation,
      "terminal living population disagrees with authoritative state");
    assert(terminal.population.birthsSinceStart === expectedBirths,
      "terminal birth total disagrees with authoritative state");
    assert(terminal.population.deathsSinceStart === expectedDeaths,
      "terminal death total disagrees with authoritative state");
    assert(terminal.migration.movesCompleted === expectedMoves,
      "terminal migration total disagrees with authoritative state");
    if (manifest) {
      assert(terminal.resources.unmetNeed === manifest.resources.unmetNeed,
        "terminal unmet need disagrees with manifest");
    } else {
      assert(terminal.resources.unmetNeed === checkpoint.resources.unmetNeed,
        "terminal unmet need disagrees with paused checkpoint");
      assert(terminal.resources.periodsProcessed === checkpoint.resources.periodsProcessed,
        "terminal resource period count disagrees with paused checkpoint");
      assert(terminal.migration.decisionBoundaries === checkpoint.migration.decisionBoundaries,
        "terminal migration boundary count disagrees with paused checkpoint");
    }
  }

  return {
    kind,
    endTime,
    durationDays: endTime,
    durationYears: endTime / 365,
    configuredDurationYears: checkpoint.experiment.durationYears,
    seed: checkpoint.experiment.seed,
    cellCount: world.cells.length,
    personRecords,
    livingPopulation,
    occupiedCells,
    finalFoodStock,
    stateDigest64: manifest?.stateDigest64 ?? checkpoint.stateDigest64,
    eventCounts: counts,
  };
}

export function countEvents(records) {
  const counts = { birth: 0, death: 0, householdMigration: 0 };
  for (const record of records ?? []) {
    const type = record?.event?.type;
    if (Object.hasOwn(counts, type)) counts[type] += 1;
  }
  return counts;
}

export function reconstructState(bundle, day) {
  const endTime = runEndTime(bundle);
  const targetDay = Math.max(0, Math.min(asNumber(day, "day"), endTime));
  const population = bundle.initialPopulation;
  const people = new Map();
  const householdMembers = new Map();
  const householdLocations = new Map();

  const founderCount = population.birthDays.length;
  for (let index = 0; index < founderCount; index += 1) {
    const id = index + 1;
    const household = asNumber(population.households[index], `household for person ${id}`);
    const location = asNumber(population.locations[index], `location for person ${id}`);
    const person = {
      id,
      birthDay: asNumber(population.birthDays[index], `birth day for person ${id}`),
      deathDay: null,
      reproductiveSex: population.reproductiveSexes[index],
      location,
      household,
      femaleParent: asNumber(population.femaleParents[index] ?? 0, `female parent for person ${id}`),
      maleParent: asNumber(population.maleParents[index] ?? 0, `male parent for person ${id}`),
      conditionPermille: asNumber(population.conditionPermille[index] ?? 1000, `condition for person ${id}`),
      conditionSource: "initial_population",
      alive: true,
      founder: true,
    };
    people.set(id, person);
    addHouseholdMember(householdMembers, household, id);
    if (!householdLocations.has(household)) householdLocations.set(household, location);
  }

  for (const record of bundle.events.events) {
    if (record.day > targetDay) break;
    const event = record.event;
    if (event.type === "birth") {
      const id = asNumber(event.person, "birth person");
      const person = {
        id,
        birthDay: record.day,
        deathDay: null,
        reproductiveSex: event.reproductive_sex ?? event.reproductiveSex,
        location: asNumber(event.cell, "birth cell"),
        household: asNumber(event.household, "birth household"),
        femaleParent: asNumber(event.female_parent ?? event.femaleParent ?? 0, "female parent"),
        maleParent: asNumber(event.male_parent ?? event.maleParent ?? 0, "male parent"),
        conditionPermille: 1000,
        conditionSource: "birth_default",
        alive: true,
        founder: false,
      };
      people.set(id, person);
      addHouseholdMember(householdMembers, person.household, id);
      householdLocations.set(person.household, person.location);
    } else if (event.type === "death") {
      const person = people.get(asNumber(event.person, "death person"));
      if (person) {
        person.alive = false;
        person.deathDay = record.day;
        person.conditionPermille = asNumber(event.condition_permille ?? event.conditionPermille ?? person.conditionPermille,
          "death condition");
        person.conditionSource = "death_event";
      }
    } else if (event.type === "householdMigration") {
      const household = asNumber(event.household, "migration household");
      const destination = asNumber(event.destination, "migration destination");
      householdLocations.set(household, destination);
      for (const personId of householdMembers.get(household) ?? []) {
        const person = people.get(personId);
        if (person?.alive) person.location = destination;
      }
    }
  }

  if (targetDay === endTime) {
    const finalConditions = bundle.checkpoint.population?.conditionPermille ?? [];
    for (let index = 0; index < finalConditions.length; index += 1) {
      const person = people.get(index + 1);
      if (person) {
        person.conditionPermille = asNumber(finalConditions[index], `final condition for person ${index + 1}`);
        person.conditionSource = "final_checkpoint";
      }
    }
  }

  const cellResidents = new Map();
  for (const person of people.values()) {
    if (!person.alive || person.birthDay > targetDay) continue;
    if (!cellResidents.has(person.location)) cellResidents.set(person.location, []);
    cellResidents.get(person.location).push(person.id);
  }

  return { day: targetDay, people, householdMembers, householdLocations, cellResidents };
}

function addHouseholdMember(index, household, personId) {
  if (!index.has(household)) index.set(household, new Set());
  index.get(household).add(personId);
}

export function genealogyForPerson(state, personId) {
  const id = asNumber(personId, "person id");
  const person = state.people.get(id);
  if (!person) return null;
  const children = [];
  for (const candidate of state.people.values()) {
    if (candidate.femaleParent === id || candidate.maleParent === id) children.push(candidate.id);
  }
  children.sort((a, b) => a - b);
  return {
    person,
    femaleParent: person.femaleParent > 0 ? state.people.get(person.femaleParent) ?? null : null,
    maleParent: person.maleParent > 0 ? state.people.get(person.maleParent) ?? null : null,
    children,
  };
}

export function eventsForEntity(bundle, { person = null, household = null, cell = null, type = null } = {}) {
  return bundle.events.events.filter((record) => {
    const event = record.event;
    if (type && event.type !== type) return false;
    if (person !== null) {
      const personId = Number(person);
      const touchesPerson = Number(event.person) === personId ||
        Number(event.female_parent ?? event.femaleParent) === personId ||
        Number(event.male_parent ?? event.maleParent) === personId;
      if (!touchesPerson) return false;
    }
    if (household !== null && Number(event.household) !== Number(household)) return false;
    if (cell !== null) {
      const cellId = Number(cell);
      const touchesCell = Number(event.cell) === cellId || Number(event.origin) === cellId || Number(event.destination) === cellId;
      if (!touchesCell) return false;
    }
    return true;
  });
}

export function snapshotAtOrBefore(metrics, day) {
  let selected = null;
  for (const snapshot of metrics.snapshots ?? []) {
    if (snapshot.day > day) break;
    selected = snapshot;
  }
  return selected;
}

export function mapValues(bundle, state, overlay) {
  const cellCount = bundle.world.cells.length;
  if (overlay === "population") {
    return Array.from({ length: cellCount }, (_, index) => state.cellResidents.get(index + 1)?.length ?? 0);
  }
  if (overlay === "productivity") return bundle.world.cells.map((cell) => asNumber(cell.baseProductivity, "baseline productivity"));
  if (overlay === "water") return bundle.world.cells.map((cell) => asNumber(cell.waterAccess, "water access"));
  if (overlay === "movement") return bundle.world.cells.map((cell) => asNumber(cell.movementCost, "movement cost"));
  if (overlay === "finalFood") {
    return bundle.checkpoint.resources.cellFoodStock.map((value, index) => asNumber(value, `final food stock for cell ${index + 1}`));
  }
  throw new Error(`unknown map overlay ${overlay}`);
}

export function summarizeCell(bundle, state, cellId) {
  const id = asNumber(cellId, "cell id");
  const cell = bundle.world.cells[id - 1];
  if (!cell) return null;
  const residents = state.cellResidents.get(id) ?? [];
  return {
    id,
    world: cell,
    livingPopulation: residents.length,
    residents,
    finalFoodStock: bundle.checkpoint.resources.cellFoodStock[id - 1] ?? 0,
    finalOnly: state.day !== runEndTime(bundle),
  };
}
