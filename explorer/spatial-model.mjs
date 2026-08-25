const MAX_SAFE_BIGINT = BigInt(Number.MAX_SAFE_INTEGER);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function asNumber(value, label) {
  const number = Number(value);
  if (!Number.isFinite(number)) throw new Error(`${label} is not numeric`);
  if (Number.isInteger(number) && !Number.isSafeInteger(number)) {
    throw new Error(`${label} exceeds JavaScript's exact integer range`);
  }
  return number;
}

function exactJsonEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function validateSpatialArtifacts(bundle, runInfo) {
  const { landscape, landscapeManifest, spatialMechanisms, spatialObservability, temporaryObservability } = bundle;
  if (!landscape && !landscapeManifest && !spatialMechanisms && !spatialObservability) {
    return { available: false };
  }
  assert(landscape, "spatial artifacts require landscape.json");
  assert(landscape.schemaVersion === 1, `unsupported landscape schema ${landscape.schemaVersion}`);
  assert(landscape.width === bundle.world.width && landscape.height === bundle.world.height,
    "landscape dimensions disagree with authoritative world");
  assert(Array.isArray(landscape.layers), "landscape layers are missing");
  assert(landscape.geometry, "landscape geometry is missing");
  assert(landscape.geometry.spatialReference, "landscape spatial reference is missing");
  assert(landscape.geometry.coordinateUnit, "landscape coordinate unit is missing");
  const cellCount = landscape.width * landscape.height;
  const layerIds = new Set();
  for (const layer of landscape.layers) {
    assert(layer.layerId && !layerIds.has(layer.layerId), `invalid or duplicate landscape layer ${layer.layerId}`);
    layerIds.add(layer.layerId);
    assert(Array.isArray(layer.values) && layer.values.length === cellCount,
      `landscape layer ${layer.layerId} does not match grid cell count`);
  }

  if (spatialMechanisms) {
    assert(spatialMechanisms.schemaVersion === 1,
      `unsupported spatial mechanism schema ${spatialMechanisms.schemaVersion}`);
    for (const transform of spatialMechanisms.transforms ?? []) {
      assert(layerIds.has(transform.sourceLayerId),
        `spatial transform references missing landscape layer ${transform.sourceLayerId}`);
    }
  }

  if (landscapeManifest) {
    assert(landscapeManifest.landscape, "landscape manifest is missing landscape binding");
    assert(landscapeManifest.landscape.width === landscape.width &&
      landscapeManifest.landscape.height === landscape.height,
    "landscape manifest dimensions disagree with landscape.json");
    if (landscapeManifest.spatial && spatialMechanisms) {
      assert(exactJsonEqual(landscapeManifest.spatial.config, spatialMechanisms),
        "spatial-mechanisms.json disagrees with landscape manifest");
    }
  }

  if (spatialObservability) {
    assert(spatialObservability.schemaVersion === 2,
      `unsupported spatial observability schema ${spatialObservability.schemaVersion}`);
    assert(spatialObservability.provenance === "derived",
      "spatial observability is not labelled derived");
    assert(spatialObservability.semantics?.populationLocationBasis === "persistent_residence",
      "spatial population location basis must be persistent residence");
    assert(spatialObservability.semantics?.occupancyIncludesTemporaryVisitors === false,
      "spatial occupancy must explicitly exclude temporary visitors");
    assert(spatialObservability.semantics?.occupancyIncludesTransit === false,
      "spatial occupancy must explicitly exclude transit");
    assert(spatialObservability.semantics?.birthCellAttribution === "persistent_residence",
      "spatial birth attribution must be persistent residence");
    assert(spatialObservability.semantics?.deathCellAttribution === "persistent_residence",
      "spatial death attribution must be persistent residence");
    assert(spatialObservability.width === landscape.width &&
      spatialObservability.height === landscape.height,
    "spatial observability dimensions disagree with landscape");
    assert(spatialObservability.source.seed === runInfo.seed,
      "spatial observability seed disagrees with run");
    assert(spatialObservability.source.endDay === runInfo.endTime,
      "spatial observability end day disagrees with run boundary");
    assert(String(spatialObservability.source.runStateDigest64) === String(runInfo.stateDigest64),
      "spatial observability state digest disagrees with run");
    assert(Array.isArray(spatialObservability.cells) && spatialObservability.cells.length === cellCount,
      "spatial observability cell table does not match grid");
    for (let index = 0; index < spatialObservability.cells.length; index += 1) {
      const row = spatialObservability.cells[index];
      assert(Number(row.cell) === index + 1, `spatial observability cell order is invalid at ${index + 1}`);
      assert(row.derived?.provenance === "derived",
        `spatial observability cell ${index + 1} is not labelled derived`);
    }
    if (landscapeManifest?.landscape?.landscapeIdentity) {
      assert(spatialObservability.source.landscapeIdentity ===
        landscapeManifest.landscape.landscapeIdentity,
      "spatial observability landscape identity disagrees with landscape manifest");
    }
    const reportLayers = new Set((spatialObservability.normalizedLayers ?? []).map((layer) => layer.layerId));
    for (const layer of landscape.layers) {
      assert(reportLayers.has(layer.layerId),
        `spatial observability omits normalized layer ${layer.layerId}`);
    }
  }

  if (temporaryObservability) {
    assert(temporaryObservability.schemaVersion === 1,
      `unsupported temporary observability schema ${temporaryObservability.schemaVersion}`);
    assert(temporaryObservability.source.seed === runInfo.seed,
      "temporary observability seed disagrees with run");
    assert(temporaryObservability.source.endDay === runInfo.endTime,
      "temporary observability end day disagrees with run boundary");
    assert(String(temporaryObservability.source.runStateDigest64) === String(runInfo.stateDigest64),
      "temporary observability state digest disagrees with run");
    assert(spatialObservability?.semantics?.physicalPresenceCompanionArtifact === "temporary-observability.json",
      "spatial observability does not declare its M9 physical-presence companion");
  }

  return {
    available: true,
    cellCount,
    transformed: Boolean(spatialMechanisms || landscapeManifest?.spatial),
    hasObservability: Boolean(spatialObservability),
  };
}

export function spatialOverlayOptions(bundle) {
  const options = [];
  for (const layer of bundle.landscape?.layers ?? []) {
    options.push({
      value: `landscape:${layer.layerId}`,
      label: `Input · ${layer.layerId} (${layer.role})`,
      provenance: "normalized_input",
    });
  }
  if (bundle.spatialObservability) {
    options.push(
      { value: "derived:occupancyPersistence", label: "Derived · residence occupancy persistence", provenance: "derived" },
      { value: "derived:personDays", label: "Derived · resident living person-days", provenance: "derived" },
      { value: "derived:terminalPopulation", label: "Derived · terminal resident population", provenance: "derived" },
      { value: "derived:scarcityDeaths", label: "Derived · residence-attributed scarcity deaths", provenance: "derived" },
      { value: "derived:migrationPeopleOut", label: "Derived · migration people out", provenance: "derived" },
    );
  }
  return options;
}

export function spatialMapValues(bundle, overlay) {
  if (overlay.startsWith("landscape:")) {
    const layerId = overlay.slice("landscape:".length);
    const layer = bundle.landscape?.layers?.find((candidate) => candidate.layerId === layerId);
    if (!layer) throw new Error(`unknown landscape layer ${layerId}`);
    return layer.values.map((value, index) => value === null ? null : asNumber(value, `${layerId} cell ${index + 1}`));
  }
  const report = bundle.spatialObservability;
  if (!report) throw new Error(`spatial observability report is unavailable for ${overlay}`);
  if (overlay === "derived:occupancyPersistence") {
    return report.cells.map((row) => row.derived.occupancyFractionPermille ?? null);
  }
  if (overlay === "derived:personDays") {
    return report.cells.map((row, index) => asNumber(row.derived.livingPersonDays, `person-days cell ${index + 1}`));
  }
  if (overlay === "derived:terminalPopulation") {
    return report.cells.map((row, index) => asNumber(row.derived.terminalLivingPopulation, `terminal population cell ${index + 1}`));
  }
  if (overlay === "derived:scarcityDeaths") {
    return report.cells.map((row, index) => asNumber(row.derived.resourceScarcityDeaths, `scarcity deaths cell ${index + 1}`));
  }
  if (overlay === "derived:migrationPeopleOut") {
    return report.cells.map((row, index) => asNumber(row.derived.migrationPeopleOut, `migration people out cell ${index + 1}`));
  }
  throw new Error(`unknown spatial overlay ${overlay}`);
}

export function spatialOverlayDescription(bundle, overlay) {
  if (overlay.startsWith("landscape:")) {
    const layerId = overlay.slice("landscape:".length);
    const layer = bundle.landscape.layers.find((candidate) => candidate.layerId === layerId);
    const evidence = layer?.evidenceInputId ? ` · evidence input ${layer.evidenceInputId}` : "";
    return `Normalized input from landscape.json · ${layer?.role ?? "unknown role"} · unit ${layer?.unit ?? "unknown"}${evidence}. Nodata remains missing; this is not simulated output.`;
  }
  const labels = {
    "derived:occupancyPersistence": "fraction of observed run time that each cell had at least one persistent resident; temporary visitors and transit are excluded",
    "derived:personDays": "integral of persistent-resident living population over time in each cell; temporary visitors and transit are excluded",
    "derived:terminalPopulation": "living persistent-resident population in each cell at the terminal checkpoint",
    "derived:scarcityDeaths": "authoritative resource-scarcity death events attributed to persistent residence rather than physical death location",
    "derived:migrationPeopleOut": "people moved out of each origin cell across authoritative household-migration events",
  };
  return `Derived M8.5 observable from spatial-observability.json: ${labels[overlay]}. It is downstream analysis and cannot alter the simulation.`;
}

export function spatialCellDetails(bundle, cellId) {
  const id = asNumber(cellId, "cell id");
  const index = id - 1;
  if (index < 0 || index >= (bundle.landscape?.width ?? 0) * (bundle.landscape?.height ?? 0)) return null;
  const layers = (bundle.landscape?.layers ?? []).map((layer) => ({
    layerId: layer.layerId,
    role: layer.role,
    unit: layer.unit,
    evidenceInputId: layer.evidenceInputId ?? null,
    value: layer.values[index] ?? null,
  }));
  return {
    id,
    layers,
    report: bundle.spatialObservability?.cells?.[index] ?? null,
    temporaryReport: bundle.temporaryObservability?.cells?.find((row) => Number(row.cell) === id) ?? null,
  };
}

export function safeFiniteSpatialValues(values) {
  return values.filter((value) => value !== null && Number.isFinite(value));
}

export function isUnsafeSpatialIntegerToken(value) {
  if (typeof value !== "string" || !/^-?\d+$/.test(value)) return false;
  const integer = BigInt(value);
  return integer > MAX_SAFE_BIGINT || integer < -MAX_SAFE_BIGINT;
}
