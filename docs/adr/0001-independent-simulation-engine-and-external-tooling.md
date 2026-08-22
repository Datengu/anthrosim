# ADR 0001: Keep AnthroSim as an independent simulation engine and integrate mature external scientific tooling

- **Status:** Accepted
- **Date:** 2026-08-22

## Context

AnthroSim is developing from a deterministic agent-based simulation core toward a reusable framework for anthropological and archaeological hypothesis testing. Several mature simulation platforms already provide substantial generic infrastructure, including NetLogo, GAMA, Repast and Mesa. These platforms offer combinations of agent scheduling, spatial worlds, GIS integration, visualisation, batch experiments, checkpointing and analysis support.

AnthroSim therefore needs a clear boundary between capabilities that are scientifically distinctive to the project and capabilities that are already well served by mature external tools.

The current AnthroSim architecture already makes several deliberate research-oriented choices:

- authoritative state is owned by a headless deterministic engine;
- stochastic processes use explicit named RNG streams;
- authoritative numerical state favours exact integer/fixed-point representations where practical;
- simulation state, observation artifacts and visualisation are separated;
- checkpoints record sufficient state, including RNG positions, for deterministic resumption;
- versioned artifacts support causal inspection and downstream analysis;
- the local explorer is a read-only consumer and cannot mutate authoritative research state.

Re-platforming AnthroSim onto a general-purpose ABM framework would gain mature generic infrastructure, but would also require adapting or abandoning some of these explicit guarantees while leaving the project to implement its anthropological and archaeological scientific layer anyway.

## Decision

AnthroSim will remain an **independent deterministic simulation engine**, currently implemented in Rust.

AnthroSim will own the parts of the system that are central to its scientific purpose, including:

- authoritative simulation state and execution semantics;
- deterministic and reproducible stochastic processes;
- anthropological mechanisms such as demography, households, kinship, mobility, social interaction and future cultural processes;
- explicit alternative behavioural models that can be selected as experimental assumptions rather than one universal model of human behaviour;
- experiment definitions, provenance and versioned run artifacts;
- mechanisms needed to compare competing anthropological or archaeological hypotheses;
- future archaeological observation models that distinguish simulated past behaviour from the archaeological record likely to survive, be detected and be sampled.

AnthroSim will **not** attempt to replace mature scientific tooling by default. In particular, it should prefer integration with existing tools and libraries for:

- GIS editing and geospatial preprocessing;
- LiDAR and raster/vector processing;
- general statistical analysis;
- plotting and visualisation outside the dedicated AnthroSim explorer;
- notebooks and exploratory analysis;
- standard sensitivity-analysis and inference algorithms;
- general-purpose scientific data transformation and storage formats.

For example, real-world terrain and palaeolandscape data should normally be prepared using tools such as QGIS/GDAL and converted into a documented, versioned AnthroSim input format rather than requiring AnthroSim to become a GIS application.

Likewise, AnthroSim should produce reproducible ensemble and parameter-sweep outputs suitable for analysis in Python or R rather than reimplementing mature statistical ecosystems inside the simulation engine.

## Relationship to existing ABM platforms

NetLogo, GAMA, Repast, Mesa and similar platforms are not considered runtime dependencies or replacement targets for AnthroSim.

They should instead be used where valuable as:

- methodological references;
- sources of established modelling practices;
- independent implementations of published models;
- validation and reproduction targets;
- comparison points for performance, usability and scientific methodology.

Where a published anthropological or archaeological model exists in one of these platforms, AnthroSim should be able to reproduce an equivalent experiment when scientifically useful. Agreement and disagreement between independent implementations can then become part of AnthroSim's validation programme.

## Architectural rule for future work

Before implementing substantial new infrastructure, ask:

> Is this capability scientifically distinctive to AnthroSim, or is it a solved scientific-software problem that should be integrated rather than reinvented?

If the capability is generic and a mature interoperable implementation exists, the default decision is to integrate or consume it externally.

If the capability materially defines AnthroSim's scientific model, reproducibility guarantees, experiment semantics or anthropological/archaeological inference framework, it belongs in AnthroSim.

Exceptions should be justified by measurable requirements such as determinism, reproducibility, performance, provenance, interoperability or the inability of existing tooling to preserve a scientifically important boundary.

## Consequences

### Positive

- preserves the deterministic and inspectable Rust research core already implemented;
- avoids a disruptive re-platforming effort with limited scientific benefit;
- keeps AnthroSim focused on anthropological and archaeological modelling rather than generic scientific software;
- allows mature GIS, statistical and scientific-computing ecosystems to evolve independently of the simulator;
- makes external ABM platforms useful independent validation targets rather than direct dependencies;
- reduces the amount of infrastructure AnthroSim must maintain;
- encourages explicit, replaceable behavioural assumptions rather than embedding a single supposedly universal model of human behaviour.

### Negative / trade-offs

- AnthroSim remains responsible for maintaining its own simulation engine and execution semantics;
- some capabilities available out of the box in mature ABM environments will require integration work or lightweight project-specific equivalents;
- independent reproduction of published models may require translation between model semantics and platforms;
- interoperability boundaries and input/output formats must be designed and versioned carefully.

## Scientific direction implied by this decision

AnthroSim should not aim to become a simulator that simply contains the greatest possible number of human behaviours. Its scientific value should come from making assumptions explicit, modular, traceable and testable.

A particularly important long-term distinction is between:

1. the simulated past state and behaviour of agents; and
2. the material, preserved, detected and sampled archaeological record that those behaviours would plausibly produce.

This archaeological-observation layer is a project-specific scientific concern and is therefore within AnthroSim's intended domain, whereas the GIS and statistical tooling used to prepare inputs or analyse outputs should generally remain external.

## Revisit conditions

Reconsider this decision only if one or more of the following become true:

- an external framework can demonstrably provide AnthroSim's required deterministic execution, state provenance and checkpoint guarantees with substantially lower maintenance cost;
- the independent engine becomes a dominant barrier to scientific validation or collaboration;
- interoperability requirements from research partners strongly favour another execution environment;
- performance or scale requirements cannot reasonably be met by the current architecture.

Any future re-platforming proposal should compare scientific guarantees and reproducibility semantics, not only feature count or development convenience.
