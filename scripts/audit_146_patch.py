from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one exact replacement target, found {count}")
    target.write_text(text.replace(old, new, 1))


spatial = "crates/anthrosim-core/src/spatial_observability.rs"
replace_once(
    spatial,
    """    pub source: SpatialObservabilitySource,\n    pub geometry: GridGeometry,\n""",
    """    pub source: SpatialObservabilitySource,\n    pub semantics: SpatialObservabilitySemantics,\n    pub geometry: GridGeometry,\n""",
)
replace_once(
    spatial,
    """impl SpatialObservabilityReport {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 1;\n}\n\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"camelCase\")]\npub struct SpatialObservabilitySource {\n""",
    """impl SpatialObservabilityReport {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"snake_case\")]\npub enum SpatialLocationAttribution {\n    PersistentResidence,\n}\n\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"camelCase\")]\npub struct SpatialObservabilitySemantics {\n    pub population_location_basis: SpatialLocationAttribution,\n    pub occupancy_includes_temporary_visitors: bool,\n    pub occupancy_includes_transit: bool,\n    pub birth_cell_attribution: SpatialLocationAttribution,\n    pub death_cell_attribution: SpatialLocationAttribution,\n    pub physical_presence_companion_artifact: Option<String>,\n}\n\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"camelCase\")]\npub struct SpatialObservabilitySource {\n""",
)
replace_once(
    spatial,
    """            spatial_config_identity: spatial.map(|binding| binding.config_identity.clone()),\n        },\n        geometry: landscape.geometry.clone(),\n""",
    """            spatial_config_identity: spatial.map(|binding| binding.config_identity.clone()),\n        },\n        semantics: SpatialObservabilitySemantics {\n            population_location_basis: SpatialLocationAttribution::PersistentResidence,\n            occupancy_includes_temporary_visitors: false,\n            occupancy_includes_transit: false,\n            birth_cell_attribution: SpatialLocationAttribution::PersistentResidence,\n            death_cell_attribution: SpatialLocationAttribution::PersistentResidence,\n            physical_presence_companion_artifact: checkpoint\n                .experiment\n                .temporary_mobility\n                .as_ref()\n                .map(|_| \"temporary-observability.json\".to_owned()),\n        },\n        geometry: landscape.geometry.clone(),\n""",
)
replace_once(
    spatial,
    """            \"historical per-person condition between authoritative death/checkpoint observations is not recorded\"\n                .to_owned(),\n        ],\n""",
    """            \"historical per-person condition between authoritative death/checkpoint observations is not recorded\"\n                .to_owned(),\n            \"spatial population, occupancy, person-day, birth and death cell observables use persistent residence and exclude temporary visitors and transit; use temporary-observability.json for M9 physical presence\"\n                .to_owned(),\n            \"Death.cell and spatial death counts are attributed to persistent residence, not necessarily the physical location of death while a household is away\"\n                .to_owned(),\n        ],\n""",
)

events = "crates/anthrosim-core/src/events.rs"
replace_once(
    events,
    """    Death {\n        person: PersonId,\n        household: HouseholdId,\n        cell: CellId,\n""",
    """    Death {\n        person: PersonId,\n        household: HouseholdId,\n        /// Persistent residence cell used for demographic/spatial attribution. Under M9 this is\n        /// not necessarily the person's physical location at death while the household is away.\n        cell: CellId,\n""",
)

lib = "crates/anthrosim-core/src/lib.rs"
replace_once(
    lib,
    """    SpatialMigrationDistanceBin, SpatialMigrationFlow, SpatialModelFacingCell,\n    SpatialObservabilityError, SpatialObservabilityReport, SpatialObservabilitySource,\n    SpatialObservabilitySummary, derive_spatial_observability,\n""",
    """    SpatialLocationAttribution, SpatialMigrationDistanceBin, SpatialMigrationFlow,\n    SpatialModelFacingCell, SpatialObservabilityError, SpatialObservabilityReport,\n    SpatialObservabilitySemantics, SpatialObservabilitySource, SpatialObservabilitySummary,\n    derive_spatial_observability,\n""",
)

model = "explorer/spatial-model.mjs"
replace_once(
    model,
    """  const { landscape, landscapeManifest, spatialMechanisms, spatialObservability } = bundle;\n""",
    """  const { landscape, landscapeManifest, spatialMechanisms, spatialObservability, temporaryObservability } = bundle;\n""",
)
replace_once(
    model,
    """    assert(spatialObservability.schemaVersion === 1,\n      `unsupported spatial observability schema ${spatialObservability.schemaVersion}`);\n    assert(spatialObservability.provenance === \"derived\",\n      \"spatial observability is not labelled derived\");\n""",
    """    assert(spatialObservability.schemaVersion === 2,\n      `unsupported spatial observability schema ${spatialObservability.schemaVersion}`);\n    assert(spatialObservability.provenance === \"derived\",\n      \"spatial observability is not labelled derived\");\n    assert(spatialObservability.semantics?.populationLocationBasis === \"persistent_residence\",\n      \"spatial population location basis must be persistent residence\");\n    assert(spatialObservability.semantics?.occupancyIncludesTemporaryVisitors === false,\n      \"spatial occupancy must explicitly exclude temporary visitors\");\n    assert(spatialObservability.semantics?.occupancyIncludesTransit === false,\n      \"spatial occupancy must explicitly exclude transit\");\n    assert(spatialObservability.semantics?.birthCellAttribution === \"persistent_residence\",\n      \"spatial birth attribution must be persistent residence\");\n    assert(spatialObservability.semantics?.deathCellAttribution === \"persistent_residence\",\n      \"spatial death attribution must be persistent residence\");\n""",
)
replace_once(
    model,
    """  return {\n    available: true,\n""",
    """  if (temporaryObservability) {\n    assert(temporaryObservability.schemaVersion === 1,\n      `unsupported temporary observability schema ${temporaryObservability.schemaVersion}`);\n    assert(temporaryObservability.source.seed === runInfo.seed,\n      \"temporary observability seed disagrees with run\");\n    assert(temporaryObservability.source.endDay === runInfo.endTime,\n      \"temporary observability end day disagrees with run boundary\");\n    assert(String(temporaryObservability.source.runStateDigest64) === String(runInfo.stateDigest64),\n      \"temporary observability state digest disagrees with run\");\n    assert(spatialObservability?.semantics?.physicalPresenceCompanionArtifact === \"temporary-observability.json\",\n      \"spatial observability does not declare its M9 physical-presence companion\");\n  }\n\n  return {\n    available: true,\n""",
)
replace_once(
    model,
    """      { value: \"derived:occupancyPersistence\", label: \"Derived · occupancy persistence\", provenance: \"derived\" },\n      { value: \"derived:personDays\", label: \"Derived · living person-days\", provenance: \"derived\" },\n      { value: \"derived:terminalPopulation\", label: \"Derived · terminal population\", provenance: \"derived\" },\n      { value: \"derived:scarcityDeaths\", label: \"Derived · scarcity deaths\", provenance: \"derived\" },\n""",
    """      { value: \"derived:occupancyPersistence\", label: \"Derived · residence occupancy persistence\", provenance: \"derived\" },\n      { value: \"derived:personDays\", label: \"Derived · resident living person-days\", provenance: \"derived\" },\n      { value: \"derived:terminalPopulation\", label: \"Derived · terminal resident population\", provenance: \"derived\" },\n      { value: \"derived:scarcityDeaths\", label: \"Derived · residence-attributed scarcity deaths\", provenance: \"derived\" },\n""",
)
replace_once(
    model,
    """    \"derived:occupancyPersistence\": \"fraction of observed run time that each cell contained at least one living simulated person\",\n    \"derived:personDays\": \"integral of living simulated population over time in each cell\",\n    \"derived:terminalPopulation\": \"living simulated population in each cell at the terminal checkpoint\",\n    \"derived:scarcityDeaths\": \"authoritative resource-scarcity death events aggregated by cell\",\n""",
    """    \"derived:occupancyPersistence\": \"fraction of observed run time that each cell had at least one persistent resident; temporary visitors and transit are excluded\",\n    \"derived:personDays\": \"integral of persistent-resident living population over time in each cell; temporary visitors and transit are excluded\",\n    \"derived:terminalPopulation\": \"living persistent-resident population in each cell at the terminal checkpoint\",\n    \"derived:scarcityDeaths\": \"authoritative resource-scarcity death events attributed to persistent residence rather than physical death location\",\n""",
)
replace_once(
    model,
    """    report: bundle.spatialObservability?.cells?.[index] ?? null,\n  };\n""",
    """    report: bundle.spatialObservability?.cells?.[index] ?? null,\n    temporaryReport: bundle.temporaryObservability?.cells?.find((row) => Number(row.cell) === id) ?? null,\n  };\n""",
)

view = "explorer/spatial.mjs"
replace_once(
    view,
    """  if (bundle.spatialObservability?.source?.spatialModelSemanticsId) {\n    addDefinition(metadata, \"Spatial semantics\", bundle.spatialObservability.source.spatialModelSemanticsId);\n  }\n""",
    """  if (bundle.spatialObservability?.source?.spatialModelSemanticsId) {\n    addDefinition(metadata, \"Spatial semantics\", bundle.spatialObservability.source.spatialModelSemanticsId);\n  }\n  if (bundle.spatialObservability?.semantics) {\n    addDefinition(metadata, \"Population location basis\", \"persistent residence\");\n    addDefinition(metadata, \"Death cell attribution\", \"persistent residence (not necessarily physical death location)\");\n    if (bundle.spatialObservability.semantics.physicalPresenceCompanionArtifact) {\n      addDefinition(metadata, \"Physical-presence companion\", bundle.spatialObservability.semantics.physicalPresenceCompanionArtifact);\n    }\n  }\n""",
)
replace_once(
    view,
    """    [\"Terminal population\", summary.terminalLivingPopulation],\n    [\"Terminal occupied cells\", summary.terminalOccupiedCells],\n    [\"Cell-time occupied\", formatPermille(summary.cellTimeOccupiedPermille)],\n    [\"Largest-cell share\", formatPermille(summary.terminalLargestCellSharePermille)],\n""",
    """    [\"Terminal resident population\", summary.terminalLivingPopulation],\n    [\"Terminal residence-occupied cells\", summary.terminalOccupiedCells],\n    [\"Residence cell-time occupied\", formatPermille(summary.cellTimeOccupiedPermille)],\n    [\"Largest residence-cell share\", formatPermille(summary.terminalLargestCellSharePermille)],\n""",
)
replace_once(
    view,
    """  for (const [label, value] of values) {\n    const card = create(\"div\", null, \"summary-card\");\n    card.append(create(\"span\", label, \"label\"), create(\"strong\", formatValue(value), \"value\"));\n    cards.append(card);\n  }\n}\n""",
    """  if (bundle.temporaryObservability?.summary) {\n    values.push(\n      [\"M9 visitor person-days\", bundle.temporaryObservability.summary.visitorPersonDays],\n      [\"M9 peak visitors\", bundle.temporaryObservability.summary.peakVisitors],\n    );\n  }\n  for (const [label, value] of values) {\n    const card = create(\"div\", null, \"summary-card\");\n    card.append(create(\"span\", label, \"label\"), create(\"strong\", formatValue(value), \"value\"));\n    cards.append(card);\n  }\n}\n""",
)
replace_once(
    view,
    """    container.append(create(\"h4\", \"Derived spatial observables\"));\n    const derived = create(\"dl\");\n    addDefinition(derived, \"Occupancy duration\", `${formatValue(report.derived.occupiedDurationDays)} days`);\n    addDefinition(derived, \"Occupancy persistence\", formatPermille(report.derived.occupancyFractionPermille));\n    addDefinition(derived, \"Living person-days\", report.derived.livingPersonDays);\n    addDefinition(derived, \"Terminal population\", report.derived.terminalLivingPopulation);\n    addDefinition(derived, \"Births / deaths\", `${report.derived.births} / ${report.derived.deaths}`);\n    addDefinition(derived, \"Scarcity deaths\", report.derived.resourceScarcityDeaths);\n""",
    """    container.append(create(\"h4\", \"Derived persistent-residence observables\"));\n    const derived = create(\"dl\");\n    addDefinition(derived, \"Residence occupancy duration\", `${formatValue(report.derived.occupiedDurationDays)} days`);\n    addDefinition(derived, \"Residence occupancy persistence\", formatPermille(report.derived.occupancyFractionPermille));\n    addDefinition(derived, \"Resident living person-days\", report.derived.livingPersonDays);\n    addDefinition(derived, \"Terminal resident population\", report.derived.terminalLivingPopulation);\n    addDefinition(derived, \"Residence-attributed births / deaths\", `${report.derived.births} / ${report.derived.deaths}`);\n    addDefinition(derived, \"Residence-attributed scarcity deaths\", report.derived.resourceScarcityDeaths);\n""",
)
replace_once(
    view,
    """    addDefinition(derived, \"Migration people in / out\", `${report.derived.migrationPeopleIn} / ${report.derived.migrationPeopleOut}`);\n    container.append(derived);\n  }\n}\n""",
    """    addDefinition(derived, \"Migration people in / out\", `${report.derived.migrationPeopleIn} / ${report.derived.migrationPeopleOut}`);\n    container.append(derived);\n\n    if (details.temporaryReport) {\n      const temporary = details.temporaryReport;\n      container.append(create(\"h4\", \"M9 physical-presence companion\"));\n      const presence = create(\"dl\");\n      addDefinition(presence, \"Persistent-residence person-days\", temporary.persistentResidencePersonDays);\n      addDefinition(presence, \"At-residence person-days\", temporary.atResidencePersonDays);\n      addDefinition(presence, \"Visitor person-days\", temporary.visitorPersonDays);\n      addDefinition(presence, \"Visitor household-days\", temporary.visitorHouseholdDays);\n      addDefinition(presence, \"Peak visitors\", temporary.peakVisitors);\n      addDefinition(presence, \"Arrivals / return departures\", `${temporary.arrivals} / ${temporary.returnDepartures}`);\n      container.append(presence);\n      container.append(create(\"p\", \"Transit is reported in temporary-observability.json totals and has no invented per-cell location.\", \"muted\"));\n    }\n  }\n}\n""",
)
replace_once(
    view,
    """    const [world, checkpoint, manifest, landscapeManifest, spatialMechanisms, spatialObservability] = await Promise.all([\n""",
    """    const [world, checkpoint, manifest, landscapeManifest, spatialMechanisms, spatialObservability, temporaryObservability] = await Promise.all([\n""",
)
replace_once(
    view,
    """      fetchArtifact(\"spatial-observability.json\"),\n    ]);\n""",
    """      fetchArtifact(\"spatial-observability.json\"),\n      fetchArtifact(\"temporary-observability.json\"),\n    ]);\n""",
)
replace_once(
    view,
    """      spatialObservability,\n    };\n""",
    """      spatialObservability,\n      temporaryObservability,\n    };\n""",
)

test = "explorer/spatial-model.test.mjs"
replace_once(test, """    schemaVersion: 1,\n    provenance: \"derived\",\n""", """    schemaVersion: 2,\n    provenance: \"derived\",\n""")
replace_once(
    test,
    """    source: {\n      seed: 7,\n      endDay: 730,\n      runStateDigest64: \"18446744073709551615\",\n      landscapeIdentity: \"landscape-v1-example\",\n    },\n    width: 2,\n""",
    """    source: {\n      seed: 7,\n      endDay: 730,\n      runStateDigest64: \"18446744073709551615\",\n      landscapeIdentity: \"landscape-v1-example\",\n    },\n    semantics: {\n      populationLocationBasis: \"persistent_residence\",\n      occupancyIncludesTemporaryVisitors: false,\n      occupancyIncludesTransit: false,\n      birthCellAttribution: \"persistent_residence\",\n      deathCellAttribution: \"persistent_residence\",\n      physicalPresenceCompanionArtifact: \"temporary-observability.json\",\n    },\n    width: 2,\n""",
)
replace_once(
    test,
    """    spatialMechanisms: null,\n    spatialObservability: report,\n  };\n""",
    """    spatialMechanisms: null,\n    spatialObservability: report,\n    temporaryObservability: {\n      schemaVersion: 1,\n      source: { seed: 7, endDay: 730, runStateDigest64: \"18446744073709551615\" },\n      summary: { visitorPersonDays: 12, peakVisitors: 3 },\n      cells: [\n        { cell: 1, persistentResidencePersonDays: 900, atResidencePersonDays: 888, visitorPersonDays: 4, visitorHouseholdDays: 2, arrivals: 1, returnDepartures: 1, peakVisitors: 2 },\n        { cell: 2, persistentResidencePersonDays: 365, atResidencePersonDays: 365, visitorPersonDays: 8, visitorHouseholdDays: 4, arrivals: 2, returnDepartures: 2, peakVisitors: 3 },\n      ],\n    },\n  };\n""",
)
replace_once(
    test,
    """  assert.match(spatialOverlayDescription(bundle, \"derived:personDays\"), /downstream analysis/);\n});\n""",
    """  assert.match(spatialOverlayDescription(bundle, \"derived:personDays\"), /temporary visitors and transit are excluded/);\n});\n""",
)
replace_once(
    test,
    """  assert.equal(details.report.derived.terminalLivingPopulation, 1);\n});\n""",
    """  assert.equal(details.report.derived.terminalLivingPopulation, 1);\n  assert.equal(details.temporaryReport.visitorPersonDays, 8);\n});\n\ntest(\"spatial semantics reject ambiguous location-basis metadata\", () => {\n  const bundle = fixture();\n  bundle.spatialObservability.semantics.populationLocationBasis = \"physical_presence\";\n  assert.throws(() => validateSpatialArtifacts(bundle, runInfo), /persistent residence/);\n});\n""",
)
