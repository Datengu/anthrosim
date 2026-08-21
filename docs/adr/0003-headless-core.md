# ADR 0003: Headless authoritative core

**Status:** Accepted  
**Date:** 2026-08-21

## Decision

The simulation engine must run without graphics, a database server, network access, or AI services. Explorers and analytical tools read versioned simulation outputs and must not mutate authoritative state during research runs.

## Rationale

This keeps batch experiments cheap, enables CI/research clusters, makes performance measurable, prevents UI requirements from contaminating model design, and allows multiple visualisation/analysis front ends to evolve independently.
