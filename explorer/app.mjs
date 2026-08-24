import {
  eventsForEntity,
  genealogyForPerson,
  mapValues,
  parseLosslessJson,
  reconstructState,
  snapshotAtOrBefore,
  summarizeCell,
  validateBundle,
} from "./model.mjs";

const REQUIRED_FILES = {
  world: "world.json",
  initialPopulation: "initial-population.json",
  events: "events.json",
  metrics: "metrics.json",
  checkpoint: "checkpoint.json",
};

let bundle;
let runInfo;
let timelineDays = [];
let selectedDay = 0;
let selectedCell = 1;
let reconstructed;

const byId = (id) => document.getElementById(id);

async function fetchArtifact(file, { optional = false } = {}) {
  const response = await fetch(`/run/${file}`, { cache: "no-store" });
  if (optional && response.status === 404) return null;
  if (!response.ok) throw new Error(`could not read ${file}: HTTP ${response.status}`);
  return parseLosslessJson(await response.text());
}

async function loadBundle() {
  const entries = await Promise.all(Object.entries(REQUIRED_FILES).map(async ([key, file]) => [key, await fetchArtifact(file)]));
  return {
    ...Object.fromEntries(entries),
    manifest: await fetchArtifact("manifest.json", { optional: true }),
    temporaryObservability: await fetchArtifact("temporary-observability.json", { optional: true }),
  };
}

function create(tag, text = null, className = null) {
  const node = document.createElement(tag);
  if (text !== null) node.textContent = String(text);
  if (className) node.className = className;
  return node;
}

function addDefinition(list, term, value) {
  list.append(create("dt", term), create("dd", value));
}

function formatNumber(value) {
  const number = Number(value ?? 0);
  return Number.isSafeInteger(number) ? number.toLocaleString() : String(value);
}

function yearLabel(day) {
  return `Year ${(day / 365).toLocaleString(undefined, { maximumFractionDigits: 2 })} · day ${formatNumber(day)}`;
}

function snapshotIndex(snapshot) {
  return bundle.metrics.snapshots.indexOf(snapshot);
}

function renderTimeline() {
  const slider = byId("timeline");
  const index = timelineDays.indexOf(selectedDay);
  slider.max = Math.max(0, timelineDays.length - 1);
  slider.value = Math.max(0, index);
  byId("timeline-label").textContent = yearLabel(selectedDay);
  byId("timeline-prev").disabled = index <= 0;
  byId("timeline-next").disabled = index >= timelineDays.length - 1;

  const snapshot = snapshotAtOrBefore(bundle.metrics, selectedDay);
  const cards = byId("summary-cards");
  cards.replaceChildren();
  const values = snapshot ? [
    ["Living population", snapshot.population.livingPopulation],
    ["Births", snapshot.population.birthsSinceStart],
    ["Deaths", snapshot.population.deathsSinceStart],
    ["Occupied cells", snapshot.population.livingOccupiedCellCount],
    ["Migration moves", snapshot.migration.movesCompleted],
    ["Unmet need", snapshot.resources.unmetNeed],
  ] : [
    ["Living population", bundle.initialPopulation.initialPopulation],
    ["Births", 0],
    ["Deaths", 0],
    ["Occupied cells", reconstructed.cellResidents.size],
    ["Migration moves", 0],
    ["Unmet need", 0],
  ];

  for (const [label, value] of values) {
    const card = create("div", null, "summary-card");
    card.append(create("span", label, "label"), create("strong", formatNumber(value), "value"));
    cards.append(card);
  }

  if (snapshot) {
    const snapshotPosition = snapshotIndex(snapshot);
    byId("snapshot-source").textContent = `metrics.json → snapshots[${snapshotPosition}] · provenance=${snapshot.provenance}`;
    byId("snapshot-raw").textContent = JSON.stringify(snapshot, null, 2);
  } else {
    byId("snapshot-source").textContent = "initial-population.json + authoritative event replay · reconstructed initial boundary";
    byId("snapshot-raw").textContent = JSON.stringify({
      day: 0,
      livingPopulation: bundle.initialPopulation.initialPopulation,
      occupiedCellsReconstructed: reconstructed.cellResidents.size,
    }, null, 2);
  }
}

function overlayProvenance(overlay) {
  if (overlay === "population") {
    return `Reconstructed display from founder locations plus authoritative events through day ${selectedDay}.`;
  }
  if (overlay === "finalFood") {
    return `Authoritative checkpoint cell food stock at day ${runInfo.endTime}; it does not represent earlier selected boundaries.`;
  }
  const labels = { productivity: "baseProductivity", water: "waterAccess", movement: "movementCost" };
  return `Authoritative simulation-world field world.json → cells[].${labels[overlay]}. This is model-facing state; for landscape-bound runs it may be transformed from normalized input and is not historical ground truth.`;
}

function colorFor(value, min, max, overlay) {
  if (overlay === "population" && value === 0) return "hsl(215 30% 8%)";
  const span = Math.max(1, max - min);
  let ratio = (value - min) / span;
  if (overlay === "population") ratio = Math.log1p(value) / Math.log1p(Math.max(1, max));
  ratio = Math.max(0, Math.min(1, ratio));
  const light = 14 + ratio * 52;
  const hue = overlay === "movement" ? 38 - ratio * 22 : 215 - ratio * 35;
  return `hsl(${hue} 68% ${light}%)`;
}

function renderTemporaryMobility() {
  const panel = byId("temporary-m9");
  const report = bundle.temporaryObservability;
  if (!report) {
    panel.hidden = true;
    return;
  }
  panel.hidden = false;
  const summary = report.summary;
  const cards = byId("temporary-summary");
  cards.replaceChildren();
  const meanVisitors = summary.meanVisitorsMillipersons === null || summary.meanVisitorsMillipersons === undefined
    ? "—"
    : (Number(summary.meanVisitorsMillipersons) / 1000).toLocaleString(undefined, { maximumFractionDigits: 3 });
  const values = [
    ["Journeys started", summary.journeysStarted],
    ["Not started", summary.notStartedTotal],
    ["Arrivals", summary.arrivals],
    ["Completed", summary.journeysCompleted],
    ["Visitor person-days", summary.visitorPersonDays],
    ["Transit person-days", summary.transitPersonDays],
    ["Peak visitors", summary.peakVisitors],
    ["Mean visitors", meanVisitors],
  ];
  for (const [label, value] of values) {
    const card = create("div", null, "summary-card");
    card.append(create("span", label, "label"), create("strong", formatNumber(value), "value"));
    cards.append(card);
  }

  const metadata = byId("temporary-metadata");
  metadata.replaceChildren();
  addDefinition(metadata, "Focal region", report.source.regionId);
  addDefinition(metadata, "Region identity", report.source.regionIdentity);
  addDefinition(metadata, "Program identity", report.source.temporaryMobilityProgramIdentity);
  addDefinition(metadata, "Travel model", report.source.travelModelIdentity ?? "not available");
  addDefinition(metadata, "Observation boundary", `day ${formatNumber(report.source.endDay)}`);
  addDefinition(metadata, "Run state digest", report.source.runStateDigest64);
  byId("temporary-provenance").textContent =
    "Derived from authoritative events/checkpoint state. Persistent residence, visiting and transit remain separate; transit has no invented map cell.";
  byId("temporary-raw").textContent = JSON.stringify(report, null, 2);
}

function renderMap() {
  const overlay = byId("overlay").value;
  const values = mapValues(bundle, reconstructed, overlay);
  const min = Math.min(...values);
  const max = Math.max(...values);
  byId("legend-min").textContent = formatNumber(min);
  byId("legend-max").textContent = formatNumber(max);
  byId("map-provenance").textContent = overlayProvenance(overlay);

  const canvas = byId("world-map");
  const widthCells = bundle.world.width;
  const heightCells = bundle.world.height;
  canvas.width = 900;
  canvas.height = Math.max(240, Math.round(900 * heightCells / widthCells));
  const context = canvas.getContext("2d");
  context.clearRect(0, 0, canvas.width, canvas.height);
  const cellWidth = canvas.width / widthCells;
  const cellHeight = canvas.height / heightCells;

  for (let row = 0; row < heightCells; row += 1) {
    for (let col = 0; col < widthCells; col += 1) {
      const index = row * widthCells + col;
      context.fillStyle = colorFor(values[index], min, max, overlay);
      context.fillRect(col * cellWidth, row * cellHeight, Math.ceil(cellWidth), Math.ceil(cellHeight));
    }
  }

  const selectedIndex = selectedCell - 1;
  if (selectedIndex >= 0 && selectedIndex < values.length) {
    const col = selectedIndex % widthCells;
    const row = Math.floor(selectedIndex / widthCells);
    context.strokeStyle = "white";
    context.lineWidth = Math.max(2, Math.min(cellWidth, cellHeight) * 0.12);
    context.strokeRect(col * cellWidth + 1, row * cellHeight + 1, cellWidth - 2, cellHeight - 2);
  }
  canvas.setAttribute("aria-label", `${widthCells} by ${heightCells} AnthroSim world grid, ${overlay} overlay, selected cell ${selectedCell}`);
}

function entityButton(kind, id, label = null) {
  const button = create("button", label ?? `${kind} ${id}`, "entity-link");
  button.type = "button";
  button.dataset.kind = kind;
  button.dataset.id = id;
  return button;
}

function renderCell(cellId) {
  const summary = summarizeCell(bundle, reconstructed, cellId);
  if (!summary) return renderMissing(`Cell ${cellId} does not exist.`);
  selectedCell = summary.id;
  const container = byId("inspector-content");
  container.replaceChildren();
  container.append(create("h3", `Cell ${summary.id}`));
  const dl = create("dl");
  addDefinition(dl, "Living population", summary.livingPopulation);
  addDefinition(dl, "Baseline productivity", summary.world.baseProductivity);
  addDefinition(dl, "Water access", summary.world.waterAccess);
  addDefinition(dl, "Movement cost", summary.world.movementCost);
  addDefinition(dl, "Elevation", summary.world.elevation);
  addDefinition(dl, "Environmental stress", summary.world.environmentalStress);
  addDefinition(dl, "Checkpoint food stock", `${formatNumber(summary.finalFoodStock)}${summary.finalOnly ? " (checkpoint boundary only)" : ""}`);
  container.append(dl);
  const links = create("div", null, "entity-links");
  for (const personId of summary.residents.slice(0, 80)) links.append(entityButton("person", personId));
  if (summary.residents.length > 80) links.append(create("span", `+${summary.residents.length - 80} more`, "muted"));
  container.append(create("h4", "Living residents at selected time"), links);
  appendRelevantEvents(container, { cell: summary.id });
  renderMap();
}

function renderHousehold(householdId) {
  const id = Number(householdId);
  const members = [...(reconstructed.householdMembers.get(id) ?? [])];
  if (!members.length) return renderMissing(`Household ${id} does not exist at this boundary.`);
  const container = byId("inspector-content");
  container.replaceChildren();
  container.append(create("h3", `Household ${id}`));
  const living = members.filter((personId) => reconstructed.people.get(personId)?.alive);
  const dl = create("dl");
  addDefinition(dl, "Reconstructed location", reconstructed.householdLocations.get(id) ?? "—");
  addDefinition(dl, "Known members", members.length);
  addDefinition(dl, "Living members", living.length);
  addDefinition(dl, "Completed migrations through this day", eventsForEntity(bundle, { household: id, type: "householdMigration" }).filter((record) => record.day <= selectedDay).length);
  container.append(dl);
  const links = create("div", null, "entity-links");
  for (const personId of members.slice(0, 100)) links.append(entityButton("person", personId));
  container.append(create("h4", "Members"), links);
  appendRelevantEvents(container, { household: id });
}

function conditionText(person) {
  if (person.conditionSource === "final_checkpoint") return `${person.conditionPermille}/1000 · authoritative checkpoint boundary`;
  if (person.conditionSource === "death_event") return `${person.conditionPermille}/1000 · authoritative death event`;
  return "not serialized at this historical boundary";
}

function renderPerson(personId) {
  const id = Number(personId);
  const genealogy = genealogyForPerson(reconstructed, id);
  if (!genealogy) return renderMissing(`Person ${id} has not been born by this boundary or does not exist.`);
  const person = genealogy.person;
  const container = byId("inspector-content");
  container.replaceChildren();
  container.append(create("h3", `Person ${id}`));
  const dl = create("dl");
  addDefinition(dl, "Status", person.alive ? "living" : `dead since day ${person.deathDay}`);
  addDefinition(dl, "Born", `${person.birthDay} (${person.founder ? "founder" : "simulated birth"})`);
  addDefinition(dl, "Age at selected boundary", `${Math.max(0, Math.floor((selectedDay - person.birthDay) / 365))} years`);
  addDefinition(dl, "Reproductive sex", person.reproductiveSex ?? "not recorded");
  addDefinition(dl, "Household", person.household);
  addDefinition(dl, "Reconstructed cell", person.location);
  addDefinition(dl, "Condition", conditionText(person));
  container.append(dl);

  const family = create("div", null, "entity-links");
  if (genealogy.femaleParent) family.append(entityButton("person", genealogy.femaleParent.id, `Female parent ${genealogy.femaleParent.id}`));
  if (genealogy.maleParent) family.append(entityButton("person", genealogy.maleParent.id, `Male parent ${genealogy.maleParent.id}`));
  for (const child of genealogy.children.slice(0, 50)) family.append(entityButton("person", child, `Child ${child}`));
  container.append(create("h4", "Genealogy"), family);
  appendRelevantEvents(container, { person: id });
}

function appendRelevantEvents(container, filter) {
  const relevant = eventsForEntity(bundle, filter).filter((record) => record.day <= selectedDay);
  container.append(create("h4", `Relevant authoritative events (${relevant.length})`));
  const list = create("div", null, "entity-links");
  for (const record of relevant.slice(-20)) {
    const button = create("button", `#${record.sequence} · ${eventSummary(record)}`, "entity-link");
    button.type = "button";
    button.addEventListener("click", () => showRawEvent(record));
    list.append(button);
  }
  container.append(list);
}

function renderMissing(message) {
  byId("inspector-content").replaceChildren(create("p", message, "muted"));
}

function inspect(kind, id) {
  byId("entity-kind").value = kind;
  byId("entity-id").value = id;
  if (kind === "cell") renderCell(id);
  else if (kind === "household") renderHousehold(id);
  else renderPerson(id);
}

function eventSummary(record) {
  const event = record.event;
  if (event.type === "birth") return `birth · person ${event.person} · household ${event.household} · cell ${event.cell}`;
  if (event.type === "death") return `death · person ${event.person} · ${event.cause} · cell ${event.cell}`;
  if (event.type === "householdMigration") {
    const peopleMoved = event.people_moved ?? event.peopleMoved;
    return `migration · household ${event.household} · ${event.origin} → ${event.destination} · ${peopleMoved} people`;
  }
  if (event.type === "temporaryJourneyNotStarted") {
    return `temporary journey not started · household ${event.household} · ${event.reason}`;
  }
  if (event.type === "temporaryJourneyDeparted") {
    return `temporary departure · household ${event.household} · ${event.residence} → ${event.destination}`;
  }
  if (event.type === "temporaryJourneyArrived") {
    return `temporary arrival · household ${event.household} · destination ${event.destination}`;
  }
  if (event.type === "temporaryReturnDeparted") {
    return `temporary return departure · household ${event.household} · destination ${event.destination} → residence ${event.residence}`;
  }
  if (event.type === "temporaryJourneyCompleted") {
    return `temporary journey completed · household ${event.household} · residence ${event.residence}`;
  }
  return event.type;
}

function showRawEvent(record) {
  const container = byId("inspector-content");
  const details = create("details", null, "raw-details");
  details.open = true;
  details.append(create("summary", `events.json → events[${record.sequence - 1}] · authoritative`));
  details.append(create("p", "Unsafe JSON integers are preserved as exact decimal strings by the explorer.", "source-ref"));
  details.append(create("pre", JSON.stringify(record, null, 2)));
  container.append(details);
  details.scrollIntoView({ behavior: "smooth", block: "nearest" });
}

function parseEntityFilter(text) {
  const match = text.trim().match(/^(person|household|cell)\s*:\s*(\d+)$/i);
  if (!match) return null;
  return { [match[1].toLowerCase()]: Number(match[2]) };
}

function renderEvents() {
  const type = byId("event-type").value || null;
  const entity = parseEntityFilter(byId("event-filter").value);
  let records = entity ? eventsForEntity(bundle, { ...entity, type }) : eventsForEntity(bundle, { type });
  if (byId("event-through-time").checked) records = records.filter((record) => record.day <= selectedDay);
  const freeText = byId("event-filter").value.trim();
  if (freeText && !entity) {
    const needle = freeText.toLowerCase();
    records = records.filter((record) => JSON.stringify(record.event).toLowerCase().includes(needle));
  }
  byId("event-count").textContent = `${formatNumber(records.length)} matching authoritative events${records.length > 300 ? " · showing first 300" : ""}`;
  const list = byId("event-list");
  list.replaceChildren();
  for (const record of records.slice(0, 300)) {
    const row = create("div", null, "event-row");
    row.append(create("span", `#${record.sequence}`, "sequence"), create("span", `day ${record.day}`, "time"));
    const button = create("button", eventSummary(record), "event-description");
    button.type = "button";
    button.addEventListener("click", () => {
      const event = record.event;
      if (event.type === "householdMigration") inspect("household", event.household);
      else inspect("person", event.person);
      showRawEvent(record);
    });
    row.append(button);
    list.append(row);
  }
}

function moveTimeline(delta) {
  const current = timelineDays.indexOf(selectedDay);
  const next = Math.max(0, Math.min(timelineDays.length - 1, current + delta));
  setDay(timelineDays[next]);
}

function setDay(day) {
  selectedDay = day;
  reconstructed = reconstructState(bundle, day);
  renderTimeline();
  renderMap();
  renderEvents();
  inspect(byId("entity-kind").value, Number(byId("entity-id").value));
}

function bindInteractions() {
  byId("timeline").addEventListener("input", (event) => setDay(timelineDays[Number(event.target.value)]));
  byId("timeline-prev").addEventListener("click", () => moveTimeline(-1));
  byId("timeline-next").addEventListener("click", () => moveTimeline(1));
  byId("overlay").addEventListener("change", renderMap);
  byId("event-type").addEventListener("change", renderEvents);
  byId("event-filter").addEventListener("input", renderEvents);
  byId("event-through-time").addEventListener("change", renderEvents);
  byId("entity-form").addEventListener("submit", (event) => {
    event.preventDefault();
    inspect(byId("entity-kind").value, Number(byId("entity-id").value));
  });
  byId("inspector-content").addEventListener("click", (event) => {
    const target = event.target.closest("[data-kind][data-id]");
    if (target) inspect(target.dataset.kind, Number(target.dataset.id));
  });
  byId("world-map").addEventListener("click", (event) => {
    const canvas = byId("world-map");
    const rect = canvas.getBoundingClientRect();
    const col = Math.min(bundle.world.width - 1, Math.max(0, Math.floor((event.clientX - rect.left) / rect.width * bundle.world.width)));
    const row = Math.min(bundle.world.height - 1, Math.max(0, Math.floor((event.clientY - rect.top) / rect.height * bundle.world.height)));
    inspect("cell", row * bundle.world.width + col + 1);
  });
}

async function start() {
  try {
    bundle = await loadBundle();
    runInfo = validateBundle(bundle);
    timelineDays = [...new Set([0, ...bundle.metrics.snapshots.map((snapshot) => snapshot.day), runInfo.endTime])].sort((a, b) => a - b);
    selectedDay = timelineDays.at(-1) ?? runInfo.endTime;
    reconstructed = reconstructState(bundle, selectedDay);

    const boundaryText = runInfo.kind === "paused"
      ? `paused at year ${runInfo.durationYears} of configured ${runInfo.configuredDurationYears}`
      : `${runInfo.durationYears} simulated years · completed run`;
    byId("run-subtitle").textContent = `seed ${runInfo.seed} · ${bundle.world.width}×${bundle.world.height} cells · ${formatNumber(runInfo.personRecords)} person records · ${boundaryText}`;
    byId("bundle-status").textContent = `${runInfo.kind === "paused" ? "Paused checkpoint" : "Completed bundle"} · state ${runInfo.stateDigest64}`;
    byId("bundle-status").classList.add("good");
    byId("app").hidden = false;
    bindInteractions();
    renderTimeline();
    renderMap();
    renderTemporaryMobility();
    renderEvents();
    inspect("cell", 1);
  } catch (error) {
    byId("bundle-status").textContent = "Invalid run bundle";
    byId("fatal-error").hidden = false;
    byId("fatal-message").textContent = error?.stack ?? String(error);
  }
}

start();