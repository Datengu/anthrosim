import { parseLosslessJson } from "./model.mjs";
import {
  safeFiniteSpatialValues,
  spatialCellDetails,
  spatialMapValues,
  spatialOverlayDescription,
  spatialOverlayOptions,
  validateSpatialArtifacts,
} from "./spatial-model.mjs";

const byId = (id) => document.getElementById(id);

async function fetchArtifact(file, { optional = true } = {}) {
  const response = await fetch(`/run/${file}`, { cache: "no-store" });
  if (optional && response.status === 404) return null;
  if (!response.ok) throw new Error(`could not read ${file}: HTTP ${response.status}`);
  return parseLosslessJson(await response.text());
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

function formatValue(value) {
  if (value === null || value === undefined) return "nodata";
  const number = Number(value);
  if (Number.isSafeInteger(number)) return number.toLocaleString();
  return String(value);
}

function formatPermille(value) {
  if (value === null || value === undefined) return "not available";
  return `${(Number(value) / 10).toLocaleString(undefined, { maximumFractionDigits: 1 })}%`;
}

function renderMetadata(bundle) {
  const geometry = bundle.landscape.geometry;
  const metadata = byId("spatial-metadata");
  metadata.replaceChildren();
  addDefinition(metadata, "Spatial reference", geometry.spatialReference);
  addDefinition(metadata, "Coordinate unit", geometry.coordinateUnit);
  addDefinition(metadata, "Grid origin", `${geometry.originX}, ${geometry.originY}`);
  addDefinition(metadata, "Cell size", `${geometry.cellSizeX} × ${geometry.cellSizeY} ${geometry.coordinateUnit}`);
  addDefinition(metadata, "Grid", `${bundle.landscape.width} × ${bundle.landscape.height} cells`);
  if (bundle.landscapeManifest?.landscape?.landscapeIdentity) {
    addDefinition(metadata, "Landscape identity", bundle.landscapeManifest.landscape.landscapeIdentity);
  }
  if (bundle.spatialObservability?.source?.runStateDigest64) {
    addDefinition(metadata, "Observed run state", bundle.spatialObservability.source.runStateDigest64);
  }
  if (bundle.spatialObservability?.source?.spatialConfigIdentity) {
    addDefinition(metadata, "Spatial transform identity", bundle.spatialObservability.source.spatialConfigIdentity);
  }
  if (bundle.spatialObservability?.source?.spatialModelSemanticsId) {
    addDefinition(metadata, "Spatial semantics", bundle.spatialObservability.source.spatialModelSemanticsId);
  }
  if (bundle.spatialObservability?.semantics) {
    addDefinition(metadata, "Population location basis", "persistent residence");
    addDefinition(metadata, "Death cell attribution", "persistent residence (not necessarily physical death location)");
    if (bundle.spatialObservability.semantics.physicalPresenceCompanionArtifact) {
      addDefinition(metadata, "Physical-presence companion", bundle.spatialObservability.semantics.physicalPresenceCompanionArtifact);
    }
  }
}

function renderLayerCatalogue(bundle) {
  const catalogue = byId("spatial-layer-catalogue");
  catalogue.replaceChildren();
  for (const layer of bundle.landscape.layers) {
    const descriptor = bundle.spatialObservability?.normalizedLayers?.find((candidate) => candidate.layerId === layer.layerId);
    const card = create("div", null, "spatial-layer-card");
    card.append(create("strong", layer.layerId));
    card.append(create("span", `Normalized input · ${layer.role} · ${layer.unit}`, "source-ref"));
    if (layer.valueDomain) {
      card.append(create("span", `Declared domain ${layer.valueDomain.min}..=${layer.valueDomain.max}`, "muted"));
    }
    if (layer.evidenceInputId) {
      card.append(create("span", `Evidence input ${layer.evidenceInputId}`, "muted"));
    }
    if (descriptor) {
      card.append(create("span", `Nodata cells ${formatValue(descriptor.nodataCells)}`, "muted"));
    }
    catalogue.append(card);
  }
}

function renderSummary(bundle) {
  const cards = byId("spatial-summary");
  cards.replaceChildren();
  const summary = bundle.spatialObservability?.summary;
  if (!summary) {
    cards.append(create("p", "No spatial-observability.json is present. Normalized input layers remain inspectable, but derived M8.5 observables have not been generated for this run.", "muted"));
    return;
  }
  const values = [
    ["Terminal resident population", summary.terminalLivingPopulation],
    ["Terminal residence-occupied cells", summary.terminalOccupiedCells],
    ["Residence cell-time occupied", formatPermille(summary.cellTimeOccupiedPermille)],
    ["Largest residence-cell share", formatPermille(summary.terminalLargestCellSharePermille)],
    ["Population HHI", summary.terminalPopulationHerfindahlPerMillion === null ? "not available" : `${formatValue(summary.terminalPopulationHerfindahlPerMillion)} / 1,000,000`],
    ["Migration moves", summary.migrationMoves],
    ["People moved", summary.migrationPeopleMoved],
    ["Migration distance", `${formatValue(summary.migrationTotalDistanceCells)} cell-steps`],
  ];
  if (bundle.temporaryObservability?.summary) {
    values.push(
      ["M9 visitor person-days", bundle.temporaryObservability.summary.visitorPersonDays],
      ["M9 peak visitors", bundle.temporaryObservability.summary.peakVisitors],
    );
  }
  for (const [label, value] of values) {
    const card = create("div", null, "summary-card");
    card.append(create("span", label, "label"), create("strong", formatValue(value), "value"));
    cards.append(card);
  }
}

function colorFor(value, min, max, derived) {
  if (value === null) return "hsl(215 8% 28%)";
  const span = Math.max(1, max - min);
  const ratio = Math.max(0, Math.min(1, (value - min) / span));
  const light = 15 + ratio * 55;
  const hue = derived ? 275 - ratio * 55 : 165 + ratio * 55;
  return `hsl(${hue} 62% ${light}%)`;
}

function renderCell(bundle, cellId) {
  const details = spatialCellDetails(bundle, cellId);
  const container = byId("spatial-cell-details");
  container.replaceChildren();
  if (!details) {
    container.append(create("p", `Cell ${cellId} does not exist.`, "muted"));
    return;
  }
  const report = details.report;
  container.append(create("h3", `Spatial cell ${cellId}`));
  if (report) {
    container.append(create("p", `Grid index (${report.gridX}, ${report.gridY}) · report row spatial-observability.json → cells[${cellId - 1}]`, "source-ref"));
  }
  const inputHeading = create("h4", "Normalized input values");
  container.append(inputHeading);
  const inputList = create("dl");
  for (const layer of details.layers) {
    addDefinition(inputList, `${layer.layerId} · ${layer.unit}`, layer.value === null ? "nodata" : layer.value);
  }
  container.append(inputList);

  if (report) {
    container.append(create("h4", "Authoritative model-facing / checkpoint values"));
    const model = create("dl");
    addDefinition(model, "Movement cost", report.modelFacing.movementCost);
    addDefinition(model, "Water access", report.modelFacing.waterAccess);
    addDefinition(model, "Base productivity", report.modelFacing.baseProductivity);
    addDefinition(model, "Initial food stock", report.modelFacing.initialFoodStock);
    addDefinition(model, "Terminal food stock", report.modelFacing.terminalFoodStock);
    container.append(model);

    container.append(create("h4", "Derived persistent-residence observables"));
    const derived = create("dl");
    addDefinition(derived, "Residence occupancy duration", `${formatValue(report.derived.occupiedDurationDays)} days`);
    addDefinition(derived, "Residence occupancy persistence", formatPermille(report.derived.occupancyFractionPermille));
    addDefinition(derived, "Resident living person-days", report.derived.livingPersonDays);
    addDefinition(derived, "Terminal resident population", report.derived.terminalLivingPopulation);
    addDefinition(derived, "Residence-attributed births / deaths", `${report.derived.births} / ${report.derived.deaths}`);
    addDefinition(derived, "Residence-attributed scarcity deaths", report.derived.resourceScarcityDeaths);
    addDefinition(derived, "Migration people in / out", `${report.derived.migrationPeopleIn} / ${report.derived.migrationPeopleOut}`);
    container.append(derived);

    if (details.temporaryReport) {
      const temporary = details.temporaryReport;
      container.append(create("h4", "M9 physical-presence companion"));
      const presence = create("dl");
      addDefinition(presence, "Persistent-residence person-days", temporary.persistentResidencePersonDays);
      addDefinition(presence, "At-residence person-days", temporary.atResidencePersonDays);
      addDefinition(presence, "Visitor person-days", temporary.visitorPersonDays);
      addDefinition(presence, "Visitor household-days", temporary.visitorHouseholdDays);
      addDefinition(presence, "Peak visitors", temporary.peakVisitors);
      addDefinition(presence, "Arrivals / return departures", `${temporary.arrivals} / ${temporary.returnDepartures}`);
      container.append(presence);
      container.append(create("p", "Transit is reported in temporary-observability.json totals and has no invented per-cell location.", "muted"));
    }
  }
}

function renderMap(bundle, selectedCell = 1) {
  const select = byId("spatial-overlay");
  const overlay = select.value;
  const values = spatialMapValues(bundle, overlay);
  const finite = safeFiniteSpatialValues(values);
  const min = finite.length ? Math.min(...finite) : 0;
  const max = finite.length ? Math.max(...finite) : 0;
  byId("spatial-legend-min").textContent = finite.length ? formatValue(min) : "nodata";
  byId("spatial-legend-max").textContent = finite.length ? formatValue(max) : "nodata";
  byId("spatial-provenance").textContent = spatialOverlayDescription(bundle, overlay);

  const canvas = byId("spatial-map");
  const widthCells = bundle.landscape.width;
  const heightCells = bundle.landscape.height;
  canvas.width = 900;
  canvas.height = Math.max(240, Math.round(900 * heightCells / widthCells));
  const context = canvas.getContext("2d");
  context.clearRect(0, 0, canvas.width, canvas.height);
  const cellWidth = canvas.width / widthCells;
  const cellHeight = canvas.height / heightCells;
  const derived = overlay.startsWith("derived:");

  for (let row = 0; row < heightCells; row += 1) {
    for (let col = 0; col < widthCells; col += 1) {
      const index = row * widthCells + col;
      context.fillStyle = colorFor(values[index], min, max, derived);
      context.fillRect(col * cellWidth, row * cellHeight, Math.ceil(cellWidth), Math.ceil(cellHeight));
    }
  }

  const index = selectedCell - 1;
  if (index >= 0 && index < values.length) {
    const col = index % widthCells;
    const row = Math.floor(index / widthCells);
    context.strokeStyle = "white";
    context.lineWidth = Math.max(2, Math.min(cellWidth, cellHeight) * 0.12);
    context.strokeRect(col * cellWidth + 1, row * cellHeight + 1, cellWidth - 2, cellHeight - 2);
  }
  canvas.dataset.selectedCell = selectedCell;
  canvas.setAttribute("aria-label", `${widthCells} by ${heightCells} normalized landscape grid, ${select.selectedOptions[0]?.textContent ?? overlay}, selected cell ${selectedCell}`);
}

function bindMap(bundle) {
  const select = byId("spatial-overlay");
  select.replaceChildren();
  for (const option of spatialOverlayOptions(bundle)) {
    const element = create("option", option.label);
    element.value = option.value;
    select.append(element);
  }
  if (!select.options.length) return;

  const redraw = () => renderMap(bundle, Number(byId("spatial-map").dataset.selectedCell ?? 1));
  select.addEventListener("change", redraw);
  byId("spatial-map").addEventListener("click", (event) => {
    const canvas = byId("spatial-map");
    const rect = canvas.getBoundingClientRect();
    const col = Math.min(bundle.landscape.width - 1,
      Math.max(0, Math.floor((event.clientX - rect.left) / rect.width * bundle.landscape.width)));
    const row = Math.min(bundle.landscape.height - 1,
      Math.max(0, Math.floor((event.clientY - rect.top) / rect.height * bundle.landscape.height)));
    const cellId = row * bundle.landscape.width + col + 1;
    renderCell(bundle, cellId);
    renderMap(bundle, cellId);
  });
  renderMap(bundle, 1);
  renderCell(bundle, 1);
}

async function startSpatial() {
  try {
    const landscape = await fetchArtifact("landscape.json");
    if (!landscape) return;
    const [world, checkpoint, manifest, landscapeManifest, spatialMechanisms, spatialObservability, temporaryObservability] = await Promise.all([
      fetchArtifact("world.json", { optional: false }),
      fetchArtifact("checkpoint.json", { optional: false }),
      fetchArtifact("manifest.json"),
      fetchArtifact("landscape-manifest.json"),
      fetchArtifact("spatial-mechanisms.json"),
      fetchArtifact("spatial-observability.json"),
      fetchArtifact("temporary-observability.json"),
    ]);
    const bundle = {
      world,
      landscape,
      landscapeManifest,
      spatialMechanisms,
      spatialObservability,
      temporaryObservability,
    };
    const runInfo = {
      seed: checkpoint.experiment.seed,
      endTime: manifest?.endTime ?? checkpoint.time,
      stateDigest64: manifest?.stateDigest64 ?? checkpoint.stateDigest64,
    };
    const status = validateSpatialArtifacts(bundle, runInfo);
    if (!status.available) return;

    renderMetadata(bundle);
    renderLayerCatalogue(bundle);
    renderSummary(bundle);
    bindMap(bundle);
    const note = byId("spatial-status");
    note.textContent = `${status.transformed ? "Transformed landscape" : "Landscape-bound control"} · ${status.hasObservability ? "derived observability verified" : "input layers only"}`;
    note.classList.add("good");
    byId("spatial-m8").hidden = false;
  } catch (error) {
    byId("spatial-m8").hidden = false;
    byId("spatial-status").textContent = "Spatial artifacts invalid";
    byId("spatial-error").hidden = false;
    byId("spatial-error").textContent = error?.stack ?? String(error);
  }
}

startSpatial();
