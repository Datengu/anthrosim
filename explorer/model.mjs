const REQUIRED_ARTIFACTS = [
  "manifest",
  "world",
  "initialPopulation",
  "events",
  "metrics",
  "checkpoint",
];

const ARTIFACT_SCHEMA_KEYS = {
  manifest: "manifest",
  world: "world",
  initialPopulation: "population",
  events: "events",
  metrics: "metrics",
  checkpoint: "checkpoint",
};

function asNumber(value, label) {
  const number = Number(value);
  if (!Number.isFinite(number)) {
    throw new Error(`${label} is not numeric`);
  }
  return number;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

export function validateBundle(bundle) {
  for (const name of REQUIRED_ARTIFACTS) {
    assert(bundle?.[name], `missing ${name} artifact`);
  }

  const { manifest, world, initialPopulation, events, metrics, checkpoint } = bundle;
  assert(manifest.schemaVersion >= 7, `manifest schema ${manifest.schemaVersion} predates M5/M6 observability`);
  assert(manifest.artifactSchemas, "manifest does not declare artifact schemas");

  for (const [artifact, schemaKey] of Object.entries(ARTIFACT_SCHEMA_KEYS)) {
    const expected = manifest.artifactSchemas[schemaKey];
    const actual = bundle[artifact].schemaVersion;
    assert(expected === actual, `${artifact} schema ${actual} does not match manifest declaration ${expected}`);
  }

  assert(world.width === manifest.world.width && world.height === manifest.world.height,
    "world dimensions disagree with manifest");
  assert(world.cells.length === world.width * world.height, "world cell count does not match dimensions");
  assert(initialPopulation.initialPopulation === manifest.population.initialPopulation,
    "initial population disagrees with manifest");
  assert(checkpoint.completedYears * 365 === checkpoint.time,
    "checkpoint is not at the declared completed annual boundary");

  const counts = countEvents(events.events);
  assert(counts.birth === manifest.population.birthsSinceStart,
    `birth events ${counts.birth} disagree with manifest ${manifest.population.birthsSinceStart}`);
  assert(counts.death === manifest.population.deathsSinceStart,
    `death events ${counts.death} disagree with manifest ${manifest.population.deathsSinceStart}`);
  assert(counts.householdMigration === manifest.migration.movesCompleted,
    `migration events ${counts.householdMigration} disagree with manifest ${manifest.migration.movesCompleted}`);
  assert(events.events.length === manifest.statistics.authoritativeEventCount,
    "event log length disagrees with manifest authoritative event count");
  assert(metrics.snapshots.length === manifest.statistics.metricSnapshotCount,
    "metric snapshot count disagrees with manifest");

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

  const terminal = metrics.snapshots.at(-1);
  if (terminal) {
    assert(terminal.day === manifest.endTime, "terminal metric day disagrees with manifest end time");
    assert(terminal.population.livingPopulation === manifest.population.livingPopulation,
      "terminal living population disagrees with manifest");
    assert(terminal.population.birthsSinceStart === manifest.population.birthsSinceStart,
      "terminal birth total disagrees with manifest");
    assert(terminal.population.deathsSinceStart === manifest.population.deathsSinceStart,
      "terminal death total disagrees with manifest");
    assert(terminal.resources.unmetNeed === manifest.resources.unmetNeed,
      "terminal unmet need disagrees with manifest");
    assert(terminal.migration.movesCompleted === manifest.migration.movesCompleted,
      "terminal migration total disagrees with manifest");
  }

  return {
    eventCounts: counts,
    durationDays: manifest.endTime,
    durationYears: manifest.endTime / 365,
    cellCount: world.cells.length,
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
  const targetDay = Math.max(0, Math.min(asNumber(day, "day"), bundle.manifest.endTime));
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
        reproductiveSex: event.reproductiveSex,
        location: asNumber(event.cell, "birth cell"),
        household: asNumber(event.household, "birth household"),
        femaleParent: asNumber(event.female_parent ?? event.femaleParent ?? 0, "female parent"),
        maleParent: asNumber(event.male_parent ?? event.maleParent ?? 0, "male parent"),
        conditionPermille: 1000,
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
  if (overlay === "productivity") return bundle.world.cells.map((cell) => Number(cell.baseProductivity));
  if (overlay === "water") return bundle.world.cells.map((cell) => Number(cell.waterAccess));
  if (overlay === "movement") return bundle.world.cells.map((cell) => Number(cell.movementCost));
  if (overlay === "finalFood") return bundle.checkpoint.resources.cellFoodStock.map(Number);
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
    finalFoodStock: Number(bundle.checkpoint.resources.cellFoodStock[id - 1] ?? 0),
    finalOnly: state.day !== bundle.manifest.endTime,
  };
}
