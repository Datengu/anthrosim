from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


# explorer/model.mjs
path = ROOT / "explorer/model.mjs"
text = path.read_text()
text = replace_once(
    text,
    '''function assert(condition, message) {
  if (!condition) throw new Error(message);
}
''',
    '''function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function asBigInt(value, label) {
  try {
    return BigInt(value);
  } catch {
    throw new Error(`${label} is not an exact integer`);
  }
}
''',
    "model bigint helper",
)
text = replace_once(
    text,
    '''  const { manifest, world, initialPopulation, events, metrics, checkpoint } = bundle;
''',
    '''  const { manifest, world, initialPopulation, events, metrics, checkpoint, temporaryObservability } = bundle;
''',
    "model destructure temporary report",
)
text = replace_once(
    text,
    '''  for (const snapshot of metrics.snapshots) {
    assert(snapshot.provenance === "derived", `metric snapshot at day ${snapshot.day} is not marked derived`);
  }

  const personRecords = manifest?.population.personRecords ?? checkpoint.population.birthDays.length;
''',
    '''  for (const snapshot of metrics.snapshots) {
    assert(snapshot.provenance === "derived", `metric snapshot at day ${snapshot.day} is not marked derived`);
  }

  if (temporaryObservability) validateTemporaryObservability(bundle, endTime);

  const personRecords = manifest?.population.personRecords ?? checkpoint.population.birthDays.length;
''',
    "model report validation call",
)
text = replace_once(
    text,
    '''    stateDigest64: manifest?.stateDigest64 ?? checkpoint.stateDigest64,
    eventCounts: counts,
  };
}

export function countEvents(records) {
''',
    '''    stateDigest64: manifest?.stateDigest64 ?? checkpoint.stateDigest64,
    eventCounts: counts,
    hasTemporaryObservability: Boolean(temporaryObservability),
  };
}

export function validateTemporaryObservability(bundle, endTime = runEndTime(bundle)) {
  const report = bundle.temporaryObservability;
  assert(report, "missing temporary observability report");
  const { checkpoint } = bundle;
  assert(report.schemaVersion === 1, `unsupported temporary observability schema ${report.schemaVersion}`);
  assert(report.provenance === "derived", "temporary observability report is not marked derived");
  assert(report.source?.modelVersion === checkpoint.modelVersion,
    "temporary observability model version disagrees with checkpoint");
  assert(report.source?.modelSemanticsId === checkpoint.modelSemanticsId,
    "temporary observability model semantics disagree with checkpoint");
  assert((report.source?.gitCommit ?? null) === (checkpoint.gitCommit ?? null),
    "temporary observability Git identity disagrees with checkpoint");
  assert(report.source?.seed === checkpoint.experiment.seed,
    "temporary observability seed disagrees with checkpoint");
  assert(report.source?.endDay === endTime,
    "temporary observability end day disagrees with run boundary");
  assert(report.source?.runStateDigest64 === checkpoint.stateDigest64,
    "temporary observability state digest disagrees with checkpoint");
  assert(report.source?.worldDigest64 === checkpoint.worldDigest64,
    "temporary observability world digest disagrees with checkpoint");
  assert(report.summary?.provenance === "derived",
    "temporary observability summary is not marked derived");
  assert(report.summary?.observationDurationDays === endTime,
    "temporary observability duration disagrees with run boundary");

  const total = asBigInt(report.summary.totalLivingPersonDays, "total living person-days");
  const persistent = asBigInt(report.summary.persistentResidencePersonDays,
    "persistent residence person-days");
  const atResidence = asBigInt(report.summary.atResidencePersonDays, "at-residence person-days");
  const visitors = asBigInt(report.summary.visitorPersonDays, "visitor person-days");
  const outbound = asBigInt(report.summary.outboundTransitPersonDays,
    "outbound transit person-days");
  const returning = asBigInt(report.summary.returnTransitPersonDays,
    "return transit person-days");
  const transit = asBigInt(report.summary.transitPersonDays, "transit person-days");
  assert(persistent === total,
    "temporary observability persistent-residence person-days do not equal total living person-days");
  assert(outbound + returning === transit,
    "temporary observability transit partition does not reconcile");
  assert(atResidence + visitors + transit === total,
    "temporary observability physical person-day partition does not reconcile");
  return report;
}

export function countEvents(records) {
''',
    "model report validator",
)
path.write_text(text)


# explorer/app.mjs
path = ROOT / "explorer/app.mjs"
text = path.read_text()
text = replace_once(
    text,
    '''  return {
    ...Object.fromEntries(entries),
    manifest: await fetchArtifact("manifest.json", { optional: true }),
  };
}
''',
    '''  return {
    ...Object.fromEntries(entries),
    manifest: await fetchArtifact("manifest.json", { optional: true }),
    temporaryObservability: await fetchArtifact("temporary-observability.json", { optional: true }),
  };
}
''',
    "app load temporary report",
)
insert_before = '''function renderMap() {
'''
render = '''function renderTemporaryMobility() {
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

'''
text = replace_once(text, insert_before, render + insert_before, "app temporary renderer")
old = '''  if (event.type === "householdMigration") {
    const peopleMoved = event.people_moved ?? event.peopleMoved;
    return `migration · household ${event.household} · ${event.origin} → ${event.destination} · ${peopleMoved} people`;
  }
  return event.type;
}
'''
new = '''  if (event.type === "householdMigration") {
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
'''
text = replace_once(text, old, new, "app event summaries")
text = replace_once(
    text,
    '''    renderTimeline();
    renderMap();
    renderEvents();
    inspect("cell", 1);
''',
    '''    renderTimeline();
    renderMap();
    renderTemporaryMobility();
    renderEvents();
    inspect("cell", 1);
''',
    "app startup temporary render",
)
path.write_text(text)


# explorer/index.html
path = ROOT / "explorer/index.html"
text = path.read_text()
insert_before = '''    <section class="panel inspector-panel" aria-labelledby="inspector-heading">
'''
panel = '''    <section id="temporary-m9" class="panel" aria-labelledby="temporary-heading" hidden>
      <div class="panel-heading">
        <div>
          <p class="eyebrow">M9.6 / TEMPORARY MOBILITY</p>
          <h2 id="temporary-heading">Temporary presence observability</h2>
        </div>
        <span class="badge derived">Derived report</span>
      </div>
      <p id="temporary-provenance" class="provenance-note"></p>
      <div id="temporary-summary" class="summary-grid"></div>
      <h3>Mobility provenance</h3>
      <dl id="temporary-metadata"></dl>
      <details class="raw-details">
        <summary>Trace this panel to temporary-observability.json</summary>
        <pre id="temporary-raw"></pre>
      </details>
    </section>

'''
text = replace_once(text, insert_before, panel + insert_before, "explorer temporary panel")
text = replace_once(
    text,
    '''          <option value="householdMigration">Household migrations</option>
''',
    '''          <option value="householdMigration">Household migrations</option>
          <option value="temporaryJourneyNotStarted">Temporary journeys not started</option>
          <option value="temporaryJourneyDeparted">Temporary departures</option>
          <option value="temporaryJourneyArrived">Temporary arrivals</option>
          <option value="temporaryReturnDeparted">Temporary return departures</option>
          <option value="temporaryJourneyCompleted">Temporary journey completions</option>
''',
    "explorer M9 event filters",
)
path.write_text(text)


# explorer/model.test.mjs
path = ROOT / "explorer/model.test.mjs"
text = path.read_text()
helper_marker = '''function pausedFixture() {
'''
helper = '''function withTemporaryObservability(bundle = fixture()) {
  bundle.checkpoint.modelSemanticsId = "anthrosim-model-semantics-v5";
  bundle.checkpoint.gitCommit = null;
  bundle.checkpoint.worldDigest64 = "777";
  bundle.temporaryObservability = {
    schemaVersion: 1,
    provenance: "derived",
    source: {
      modelVersion: bundle.checkpoint.modelVersion,
      modelSemanticsId: bundle.checkpoint.modelSemanticsId,
      gitCommit: null,
      seed: bundle.checkpoint.experiment.seed,
      endDay: bundle.checkpoint.time,
      runStateDigest64: bundle.checkpoint.stateDigest64,
      worldDigest64: bundle.checkpoint.worldDigest64,
      temporaryMobilityConfigIdentity: "temporary-config-test",
      temporaryMobilityProgramIdentity: "temporary-program-test",
      regionId: "temporary-test-region",
      regionIdentity: "temporary-region-test",
      travelModelIdentity: "temporary-travel-test",
    },
    summary: {
      provenance: "derived",
      observationDurationDays: bundle.checkpoint.time,
      totalLivingPersonDays: "100",
      persistentResidencePersonDays: "100",
      atResidencePersonDays: "70",
      visitorPersonDays: "20",
      outboundTransitPersonDays: "5",
      returnTransitPersonDays: "5",
      transitPersonDays: "10",
      journeysStarted: 2,
      notStartedTotal: 1,
      arrivals: 2,
      journeysCompleted: 2,
      peakVisitors: 3,
      meanVisitorsMillipersons: 18,
    },
  };
  return bundle;
}

'''
text = replace_once(text, helper_marker, helper + helper_marker, "explorer temporary test fixture")
insert_before = '''test("paused bundle validation uses checkpoint as the authoritative boundary without a manifest", () => {
'''
tests = '''test("optional temporary observability validates its provenance and person-day partition", () => {
  const bundle = withTemporaryObservability();
  const result = validateBundle(bundle);
  assert.equal(result.hasTemporaryObservability, true);

  bundle.temporaryObservability.summary.visitorPersonDays = "21";
  assert.throws(() => validateBundle(bundle), /physical person-day partition does not reconcile/);
});

test("temporary observability preserves unsafe person-day integers exactly", () => {
  const bundle = withTemporaryObservability();
  bundle.temporaryObservability.summary.totalLivingPersonDays = "18446744073709551615";
  bundle.temporaryObservability.summary.persistentResidencePersonDays = "18446744073709551615";
  bundle.temporaryObservability.summary.atResidencePersonDays = "18446744073709551605";
  bundle.temporaryObservability.summary.visitorPersonDays = "0";
  bundle.temporaryObservability.summary.outboundTransitPersonDays = "5";
  bundle.temporaryObservability.summary.returnTransitPersonDays = "5";
  bundle.temporaryObservability.summary.transitPersonDays = "10";
  assert.equal(validateBundle(bundle).hasTemporaryObservability, true);
});

'''
text = replace_once(text, insert_before, tests + insert_before, "explorer temporary tests")
path.write_text(text)

print("patched M9.6 read-only Explorer support")
