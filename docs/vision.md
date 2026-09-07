# AnthroSim vision

AnthroSim is a modular agent-based simulation framework for exploring how human demographic and social patterns can emerge from lower-level causes.

The long-term ambition is to support increasingly rich simulations of human populations interacting with geography, ecology, kinship, knowledge, culture, language, exchange, disease, institutions, and conflict. The project is not intended to predict a single true alternative human history. It is intended to provide an inspectable experimental system in which assumptions can be isolated, varied, and tested across many reproducible runs.

## North star

**Build enough defensible rules that history-like structure can emerge without scripting the history.**

AnthroSim should eventually support two complementary modes:

- **Exploration:** inspect a single artificial history in depth, from population-scale patterns down to the people and events that produced them.
- **Experimentation:** execute many controlled runs to measure how outcomes change when assumptions, parameters, environments, or random seeds vary.

## Development strategy

After the v0.1 experiment-engine baseline, development should become increasingly **question-led rather than feature-led**. New mechanisms and infrastructure should normally be introduced because they are needed to test a declared hypothesis, null model, validation target or methodological question—not simply because the behaviour exists in real societies or would make the simulation richer.

The public core roadmap remains case-study-neutral. Reusable scientific capabilities, assumptions and validation boundaries belong in the repository; the general engine architecture should not depend on any particular locality, dataset or research question.

Evidence-grounded spatial experiments are now an established M8 capability, and temporary mobility / controlled aggregation is an established M9 capability. Future substantive mechanisms should be prioritised in response to declared research or methodological questions and to what controlled experiments show is missing or inadequately explained, rather than by extending the engine for its own sake.

See [`roadmap.md`](roadmap.md) for the current development strategy, completed milestone/release history, and the present question-led boundary.

## What AnthroSim is not

AnthroSim is not a grand-strategy game, a procedural lore generator, or an LLM role-play system. It may eventually be visually engaging, but research inspectability takes precedence over game balance or dramatic storytelling.

It is also not automatically a scientifically valid model merely because it simulates humans. Scientific credibility must be earned module by module through explicit assumptions, empirical grounding, calibration where appropriate, sensitivity analysis, verification, validation, and external review.

## Long-term research possibilities

Potential questions include:

- population persistence, fragmentation, bottlenecks, and migration;
- cultural transmission and loss of knowledge;
- interaction between ecology, demography, and mobility;
- language divergence and contact;
- settlement emergence and persistence;
- exchange networks and specialisation;
- inequality and institutional emergence;
- the archaeological signatures produced by known simulated histories;
- how reliably archaeological inference can recover simulated ground truth.

These are directions, not promises for the current release line.
