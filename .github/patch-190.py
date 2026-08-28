from pathlib import Path
import re


def sub_once(path, pattern, replacement, flags=0):
    p = Path(path)
    text = p.read_text()
    new, count = re.subn(pattern, replacement, text, count=1, flags=flags)
    if count != 1:
        raise SystemExit(f"{path}: expected one regex match, got {count}: {pattern[:80]}")
    p.write_text(new)


def replace_once(path, old, new):
    p = Path(path)
    text = p.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"{path}: expected one exact match, got {text.count(old)}: {old[:80]!r}")
    p.write_text(text.replace(old, new, 1))

# --- M9.4 routing: retain all equal minima and route-hop metadata. ---
path = "crates/anthrosim-core/src/temporary_travel.rs"
replace_once(
    path,
    "use std::{cmp::Ordering, collections::BinaryHeap};",
    "use std::{\n    cmp::Ordering,\n    collections::{BTreeMap, BinaryHeap},\n};",
)
replace_once(
    path,
    "        TemporaryMobilityProgramError, TemporaryTravelResolution, TemporaryTravelTable,\n",
    "        TemporaryMobilityProgramError, TemporaryTravelDestinationCandidate,\n        TemporaryTravelResolution, TemporaryTravelTable,\n",
)
sub_once(
    path,
    r"    /// Derive one indexed M9\.4 travel table for every authoritative world origin\.\n    ///\n    /// One multi-source search is seeded by all focal-region cells, avoiding a global search per\n    /// household\. Equal-cost destinations choose the lower authoritative `CellId`\.\n    pub fn derive_table\(\n        &self,\n        region: &FocalRegion,\n        world: &World,\n    \) -> Result<TemporaryTravelTable, TemporaryTravelModelError> \{.*?\n    \}\n\}\n\nimpl Default",
    '''    /// Derive one indexed M9.4 travel table for every authoritative world origin.\n    ///\n    /// The public helper uses a zero tie seed for callers that only need static travel geometry.\n    /// Authoritative simulations call `derive_table_with_tie_seed` with the experiment seed.\n    pub fn derive_table(\n        &self,\n        region: &FocalRegion,\n        world: &World,\n    ) -> Result<TemporaryTravelTable, TemporaryTravelModelError> {\n        self.derive_table_with_tie_seed(region, world, 0)\n    }\n\n    /// Derive M9.4 travel geometry while preserving every exactly equal minimum destination.\n    ///\n    /// `destination_tie_seed` does not affect route cost or reachability. It is retained in the\n    /// table solely so M9 execution can resolve an equal-cost destination with the declared keyed\n    /// tie policy without consuming any sequential RNG stream.\n    pub fn derive_table_with_tie_seed(\n        &self,\n        region: &FocalRegion,\n        world: &World,\n        destination_tie_seed: u64,\n    ) -> Result<TemporaryTravelTable, TemporaryTravelModelError> {\n        self.validate()?;\n        region.validate(world)?;\n        for &cell in region.member_cells() {\n            if !self.is_traversable(world, cell) {\n                return Err(TemporaryTravelModelError::RegionCellImpassable { cell });\n            }\n        }\n\n        let labels = minimum_cost_labels(self, region, world)?;\n        let mut resolutions = Vec::with_capacity(world.cell_count());\n        let mut accumulated_costs = Vec::with_capacity(world.cell_count());\n        let mut equal_cost_destinations = Vec::with_capacity(world.cell_count());\n\n        for label in labels {\n            let Some(label) = label else {\n                resolutions.push(TemporaryTravelResolution::Unreachable);\n                accumulated_costs.push(None);\n                equal_cost_destinations.push(Vec::new());\n                continue;\n            };\n            let candidates = label\n                .destinations\n                .into_iter()\n                .map(|(destination, route_distance_edges)| TemporaryTravelDestinationCandidate {\n                    destination,\n                    route_distance_edges,\n                })\n                .collect::<Vec<_>>();\n            let destination = candidates\n                .first()\n                .expect("reachable M9.4 label must retain at least one destination")\n                .destination;\n            let travel_days = self.travel_days(label.cost)?;\n            resolutions.push(TemporaryTravelResolution::Reachable {\n                destination,\n                outbound_travel_days: travel_days,\n                return_travel_days: travel_days,\n            });\n            accumulated_costs.push(Some(label.cost));\n            equal_cost_destinations.push(candidates);\n        }\n\n        TemporaryTravelTable::new_m9_4(\n            resolutions,\n            accumulated_costs,\n            equal_cost_destinations,\n            destination_tie_seed,\n            self.clone(),\n            region,\n            world,\n        )\n        .map_err(TemporaryTravelModelError::TravelTable)\n    }\n}\n\nimpl Default''',
    flags=re.S,
)
sub_once(
    path,
    r"#\[derive\(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord\)\]\nstruct RouteLabel \{.*?\nfn cell_index\(",
    '''#[derive(Debug, Clone, PartialEq, Eq)]\nstruct RouteLabel {\n    cost: u64,\n    destinations: BTreeMap<CellId, u32>,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nstruct QueueState {\n    cost: u64,\n    cell: CellId,\n}\n\nimpl Ord for QueueState {\n    fn cmp(&self, other: &Self) -> Ordering {\n        // `BinaryHeap` is a max-heap; reverse cost and cell for deterministic minimum-first work.\n        other\n            .cost\n            .cmp(&self.cost)\n            .then_with(|| other.cell.cmp(&self.cell))\n    }\n}\n\nimpl PartialOrd for QueueState {\n    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {\n        Some(self.cmp(other))\n    }\n}\n\nfn minimum_cost_labels(\n    model: &TemporaryTravelModel,\n    region: &FocalRegion,\n    world: &World,\n) -> Result<Vec<Option<RouteLabel>>, TemporaryTravelModelError> {\n    let mut labels = vec![None; world.cell_count()];\n    let mut queue = BinaryHeap::new();\n\n    for &destination in region.member_cells() {\n        let index = cell_index(destination, world)?;\n        let mut destinations = BTreeMap::new();\n        destinations.insert(destination, 0);\n        labels[index] = Some(RouteLabel {\n            cost: 0,\n            destinations,\n        });\n        queue.push(QueueState {\n            cost: 0,\n            cell: destination,\n        });\n    }\n\n    while let Some(current) = queue.pop() {\n        let current_index = cell_index(current.cell, world)?;\n        let Some(current_label) = labels[current_index].as_ref() else {\n            continue;\n        };\n        if current_label.cost != current.cost {\n            continue;\n        }\n        let current_destinations = current_label.destinations.clone();\n\n        for neighbour in world.neighbours4(current.cell).into_iter().flatten() {\n            if !model.is_traversable(world, neighbour) {\n                continue;\n            }\n            let edge = temporary_travel_edge_cost(world, current.cell, neighbour)?;\n            let candidate_cost = current\n                .cost\n                .checked_add(edge)\n                .ok_or(TemporaryTravelModelError::AccumulatedCostOverflow)?;\n            let mut candidate_destinations = BTreeMap::new();\n            for (destination, hops) in &current_destinations {\n                candidate_destinations.insert(\n                    *destination,\n                    hops.checked_add(1)\n                        .ok_or(TemporaryTravelModelError::RouteDistanceOverflow)?,\n                );\n            }\n            let neighbour_index = cell_index(neighbour, world)?;\n            let mut changed = false;\n            match labels[neighbour_index].as_mut() {\n                None => {\n                    labels[neighbour_index] = Some(RouteLabel {\n                        cost: candidate_cost,\n                        destinations: candidate_destinations,\n                    });\n                    changed = true;\n                }\n                Some(existing) if candidate_cost < existing.cost => {\n                    *existing = RouteLabel {\n                        cost: candidate_cost,\n                        destinations: candidate_destinations,\n                    };\n                    changed = true;\n                }\n                Some(existing) if candidate_cost == existing.cost => {\n                    for (destination, candidate_hops) in candidate_destinations {\n                        match existing.destinations.get_mut(&destination) {\n                            None => {\n                                existing.destinations.insert(destination, candidate_hops);\n                                changed = true;\n                            }\n                            Some(existing_hops) if candidate_hops < *existing_hops => {\n                                *existing_hops = candidate_hops;\n                                changed = true;\n                            }\n                            _ => {}\n                        }\n                    }\n                }\n                Some(_) => {}\n            }\n            if changed {\n                queue.push(QueueState {\n                    cost: candidate_cost,\n                    cell: neighbour,\n                });\n            }\n        }\n    }\n\n    Ok(labels)\n}\n\nfn cell_index(''',
    flags=re.S,
)
replace_once(
    path,
    '    #[error("temporary travel accumulated cost overflowed u64")]\n    AccumulatedCostOverflow,\n',
    '    #[error("temporary travel accumulated cost overflowed u64")]\n    AccumulatedCostOverflow,\n    #[error("temporary travel minimum-cost route distance exceeds u32 edges")]\n    RouteDistanceOverflow,\n',
)
replace_once(
    path,
    "    fn equal_cost_destinations_choose_lower_cell_id() {\n",
    "    fn equal_cost_destinations_preserve_all_minima() {\n",
)
old_assert = '''        assert!(matches!(\n            table.resolution(CellId::new(2)),\n            Some(TemporaryTravelResolution::Reachable {\n                destination,\n                ..\n            }) if destination == CellId::new(1)\n        ));\n'''
new_assert = '''        let candidates = table.equal_cost_destinations(CellId::new(2)).unwrap();\n        assert_eq!(candidates.len(), 2);\n        assert_eq!(candidates[0].destination, CellId::new(1));\n        assert_eq!(candidates[1].destination, CellId::new(3));\n        assert_eq!(candidates[0].route_distance_edges, 1);\n        assert_eq!(candidates[1].route_distance_edges, 1);\n'''
replace_once(path, old_assert, new_assert)

# --- Travel table: versioned equal-minimum metadata and keyed tie resolution. ---
path = "crates/anthrosim-core/src/temporary_mobility.rs"
replace_once(
    path,
    "const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;\n",
    'const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;\nconst M9_DESTINATION_TIE_POLICY_ID: &str = "m9/equal-cost-destination-keyed-v1";\n',
)
replace_once(
    path,
    "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(tag = \"status\", rename_all = \"snake_case\")]\npub enum TemporaryTravelResolution {",
    '''#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = "camelCase")]\npub struct TemporaryTravelDestinationCandidate {\n    pub destination: CellId,\n    pub route_distance_edges: u32,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(tag = "status", rename_all = "snake_case")]\npub enum TemporaryTravelResolution {''',
)
replace_once(
    path,
    '''    #[serde(default, skip_serializing_if = "Option::is_none")]\n    accumulated_cost_units: Option<Vec<Option<u64>>>,\n}\n\nimpl TemporaryTravelTable {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;''',
    '''    #[serde(default, skip_serializing_if = "Option::is_none")]\n    accumulated_cost_units: Option<Vec<Option<u64>>>,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    equal_cost_destinations: Option<Vec<Vec<TemporaryTravelDestinationCandidate>>>,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    destination_tie_seed: Option<u64>,\n}\n\nimpl TemporaryTravelTable {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 3;''',
)
replace_once(
    path,
    '''            travel_model: None,\n            accumulated_cost_units: None,\n        };''',
    '''            travel_model: None,\n            accumulated_cost_units: None,\n            equal_cost_destinations: None,\n            destination_tie_seed: None,\n        };''',
)
replace_once(
    path,
    '''    pub(crate) fn new_m9_4(\n        resolutions: Vec<TemporaryTravelResolution>,\n        accumulated_cost_units: Vec<Option<u64>>,\n        travel_model: TemporaryTravelModel,\n        region: &FocalRegion,\n        world: &World,\n    ) -> Result<Self, TemporaryMobilityProgramError> {\n        let table = Self {\n            schema_version: Self::CURRENT_SCHEMA_VERSION,\n            resolutions,\n            travel_model: Some(travel_model),\n            accumulated_cost_units: Some(accumulated_cost_units),\n        };''',
    '''    pub(crate) fn new_m9_4(\n        resolutions: Vec<TemporaryTravelResolution>,\n        accumulated_cost_units: Vec<Option<u64>>,\n        equal_cost_destinations: Vec<Vec<TemporaryTravelDestinationCandidate>>,\n        destination_tie_seed: u64,\n        travel_model: TemporaryTravelModel,\n        region: &FocalRegion,\n        world: &World,\n    ) -> Result<Self, TemporaryMobilityProgramError> {\n        let table = Self {\n            schema_version: Self::CURRENT_SCHEMA_VERSION,\n            resolutions,\n            travel_model: Some(travel_model),\n            accumulated_cost_units: Some(accumulated_cost_units),\n            equal_cost_destinations: Some(equal_cost_destinations),\n            destination_tie_seed: Some(destination_tie_seed),\n        };''',
)
insert_after = '''    pub fn accumulated_cost_units(&self, origin: CellId) -> Option<u64> {\n        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;\n        self.accumulated_cost_units\n            .as_ref()?\n            .get(index)\n            .copied()\n            .flatten()\n    }\n'''
addition = '''\n    #[must_use]\n    pub fn equal_cost_destinations(\n        &self,\n        origin: CellId,\n    ) -> Option<&[TemporaryTravelDestinationCandidate]> {\n        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;\n        self.equal_cost_destinations.as_ref()?.get(index).map(Vec::as_slice)\n    }\n\n    #[must_use]\n    pub fn equal_cost_destination_count(&self, origin: CellId) -> Option<u32> {\n        self.equal_cost_destinations(origin)?.len().try_into().ok()\n    }\n\n    #[must_use]\n    pub fn route_distance_edges(&self, origin: CellId, destination: CellId) -> Option<u32> {\n        self.equal_cost_destinations(origin)?\n            .iter()\n            .find(|candidate| candidate.destination == destination)\n            .map(|candidate| candidate.route_distance_edges)\n    }\n\n    /// Resolve one household/trigger destination without consuming a mutable RNG stream.\n    #[must_use]\n    pub fn resolution_for(\n        &self,\n        origin: CellId,\n        household: HouseholdId,\n        trigger_index: u32,\n    ) -> Option<TemporaryTravelResolution> {\n        let base = self.resolution(origin)?;\n        let TemporaryTravelResolution::Reachable {\n            outbound_travel_days,\n            return_travel_days,\n            ..\n        } = base\n        else {\n            return Some(base);\n        };\n        let Some(candidates) = self.equal_cost_destinations(origin) else {\n            return Some(base);\n        };\n        if candidates.len() <= 1 {\n            return Some(base);\n        }\n        let mut hash = FNV_OFFSET_BASIS;\n        digest_str(&mut hash, M9_DESTINATION_TIE_POLICY_ID);\n        digest_u64(&mut hash, self.destination_tie_seed.unwrap_or(0));\n        digest_u64(&mut hash, origin.0);\n        digest_u64(&mut hash, household.0);\n        digest_u64(&mut hash, u64::from(trigger_index));\n        hash = avalanche64(hash);\n        let index = usize::try_from(hash % candidates.len() as u64).ok()?;\n        Some(TemporaryTravelResolution::Reachable {\n            destination: candidates[index].destination,\n            outbound_travel_days,\n            return_travel_days,\n        })\n    }\n'''
replace_once(path, insert_after, insert_after + addition)
replace_once(
    path,
    '''        let m9_4 = match (&self.travel_model, &self.accumulated_cost_units) {\n            (None, None) => None,\n            (Some(model), Some(costs)) => {''',
    '''        let m9_4 = match (\n            &self.travel_model,\n            &self.accumulated_cost_units,\n            &self.equal_cost_destinations,\n            self.destination_tie_seed,\n        ) {\n            (None, None, None, None) => None,\n            (Some(model), Some(costs), Some(candidates), Some(_tie_seed)) => {''',
)
replace_once(
    path,
    '''                if costs.len() != world.cell_count() {\n                    return Err(\n                        TemporaryMobilityProgramError::TravelCostTableShapeMismatch {\n                            table: costs.len(),\n                            world: world.cell_count(),\n                        },\n                    );\n                }''',
    '''                if costs.len() != world.cell_count() {\n                    return Err(\n                        TemporaryMobilityProgramError::TravelCostTableShapeMismatch {\n                            table: costs.len(),\n                            world: world.cell_count(),\n                        },\n                    );\n                }\n                if candidates.len() != world.cell_count() {\n                    return Err(\n                        TemporaryMobilityProgramError::TravelDestinationCandidateShapeMismatch {\n                            table: candidates.len(),\n                            world: world.cell_count(),\n                        },\n                    );\n                }''',
)
replace_once(path, "                Some((model, costs))\n", "                Some((model, costs, candidates))\n")
replace_once(
    path,
    "            if let Some((model, costs)) = m9_4 {\n                match (*resolution, costs[index]) {",
    "            if let Some((model, costs, candidates)) = m9_4 {\n                match (*resolution, costs[index], candidates[index].as_slice()) {",
)
replace_once(
    path,
    '''                    (TemporaryTravelResolution::Unreachable, None) => {}\n                    (\n                        TemporaryTravelResolution::Reachable {\n                            outbound_travel_days,\n                            return_travel_days,\n                            ..\n                        },\n                        Some(cost),\n                    ) => {''',
    '''                    (TemporaryTravelResolution::Unreachable, None, []) => {}\n                    (\n                        TemporaryTravelResolution::Reachable {\n                            destination,\n                            outbound_travel_days,\n                            return_travel_days,\n                        },\n                        Some(cost),\n                        candidates,\n                    ) if !candidates.is_empty() => {\n                        if candidates[0].destination != destination\n                            || candidates.windows(2).any(|pair| {\n                                pair[0].destination >= pair[1].destination\n                            })\n                            || candidates.iter().any(|candidate| {\n                                world.cell(candidate.destination).is_none()\n                                    || !region.contains(candidate.destination)\n                            })\n                        {\n                            return Err(\n                                TemporaryMobilityProgramError::InvalidTravelDestinationCandidates {\n                                    origin,\n                                },\n                            );\n                        }''',
)
# Digest the new M9.4 metadata after costs.
needle = '''        match &self.accumulated_cost_units {\n            None => digest_u64(hash, 0),\n            Some(costs) => {\n                digest_u64(hash, 1);\n                digest_u64(hash, costs.len() as u64);\n                for cost in costs {\n                    match cost {\n                        None => digest_u64(hash, 0),\n                        Some(cost) => {\n                            digest_u64(hash, 1);\n                            digest_u64(hash, *cost);\n                        }\n                    }\n                }\n            }\n        }\n'''
extra = '''        match &self.equal_cost_destinations {\n            None => digest_u64(hash, 0),\n            Some(rows) => {\n                digest_u64(hash, 1);\n                digest_u64(hash, rows.len() as u64);\n                for row in rows {\n                    digest_u64(hash, row.len() as u64);\n                    for candidate in row {\n                        digest_u64(hash, candidate.destination.0);\n                        digest_u64(hash, u64::from(candidate.route_distance_edges));\n                    }\n                }\n            }\n        }\n        match self.destination_tie_seed {\n            None => digest_u64(hash, 0),\n            Some(seed) => {\n                digest_u64(hash, 1);\n                digest_u64(hash, seed);\n            }\n        }\n'''
replace_once(path, needle, needle + extra)
replace_once(
    path,
    '''    pub fn derive_program(\n        &self,\n        world: &World,\n    ) -> Result<TemporaryMobilityProgram, TemporaryMobilityConfigError> {\n        self.validate()?;\n        self.region.validate(world)?;\n        let travel = self.travel_model.derive_table(&self.region, world)?;\n        Ok(TemporaryMobilityProgram::new(\n            self.region.clone(),\n            self.schedule.clone(),\n            travel,\n            world,\n        )?)\n    }''',
    '''    pub fn derive_program(\n        &self,\n        world: &World,\n    ) -> Result<TemporaryMobilityProgram, TemporaryMobilityConfigError> {\n        self.derive_program_with_seed(world, 0)\n    }\n\n    pub fn derive_program_with_seed(\n        &self,\n        world: &World,\n        destination_tie_seed: u64,\n    ) -> Result<TemporaryMobilityProgram, TemporaryMobilityConfigError> {\n        self.validate()?;\n        self.region.validate(world)?;\n        let travel = self.travel_model.derive_table_with_tie_seed(\n            &self.region,\n            world,\n            destination_tie_seed,\n        )?;\n        Ok(TemporaryMobilityProgram::new(\n            self.region.clone(),\n            self.schedule.clone(),\n            travel,\n            world,\n        )?)\n    }''',
)
replace_once(
    path,
    '''        let resolution = program\n            .travel\n            .resolution(residence)\n            .ok_or(TemporaryMobilityExecutionError::MissingTravelResolution { residence })?;''',
    '''        let resolution = program\n            .travel\n            .resolution_for(residence, household, trigger_index)\n            .ok_or(TemporaryMobilityExecutionError::MissingTravelResolution { residence })?;''',
)
# Add avalanche helper alongside digest helpers.
replace_once(
    path,
    '''fn digest_u64(hash: &mut u64, value: u64) {\n    for byte in value.to_le_bytes() {\n        *hash ^= u64::from(byte);\n        *hash = (*hash).wrapping_mul(FNV_PRIME);\n    }\n}\n''',
    '''fn digest_u64(hash: &mut u64, value: u64) {\n    for byte in value.to_le_bytes() {\n        *hash ^= u64::from(byte);\n        *hash = (*hash).wrapping_mul(FNV_PRIME);\n    }\n}\n\nfn avalanche64(mut value: u64) -> u64 {\n    value ^= value >> 30;\n    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);\n    value ^= value >> 27;\n    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);\n    value ^ (value >> 31)\n}\n''',
)
replace_once(
    path,
    '''    #[error("temporary travel-cost table has {table} entries but world has {world} cells")]\n    TravelCostTableShapeMismatch { table: usize, world: usize },\n''',
    '''    #[error("temporary travel-cost table has {table} entries but world has {world} cells")]\n    TravelCostTableShapeMismatch { table: usize, world: usize },\n    #[error(\n        "temporary equal-cost destination table has {table} entries but world has {world} cells"\n    )]\n    TravelDestinationCandidateShapeMismatch { table: usize, world: usize },\n''',
)
replace_once(
    path,
    '''    #[error("temporary travel M9.4 cost presence does not match resolution for {origin:?}")]\n    TravelCostResolutionMismatch { origin: CellId },\n''',
    '''    #[error("temporary travel M9.4 cost presence does not match resolution for {origin:?}")]\n    TravelCostResolutionMismatch { origin: CellId },\n    #[error("temporary travel equal-cost destination metadata is invalid for {origin:?}")]\n    InvalidTravelDestinationCandidates { origin: CellId },\n''',
)

# --- Simulation hosts must derive the run-specific keyed tie program. ---
path = "crates/anthrosim-core/src/simulation.rs"
replace_once(
    path,
    ".map(|definition| definition.derive_program(&world))\n            .transpose()?;",
    ".map(|definition| definition.derive_program_with_seed(&world, config.seed))\n            .transpose()?;",
)
replace_once(
    path,
    "    let expected = definition.derive_program(world)?;\n",
    "    let expected = definition.derive_program_with_seed(world, config.seed)?;\n",
)

path = "crates/anthrosim-core/src/spatial_simulation.rs"
replace_once(
    path,
    ".map(|definition| definition.derive_program(&world))\n            .transpose()?;",
    ".map(|definition| definition.derive_program_with_seed(&world, config.seed))\n            .transpose()?;",
)
replace_once(
    path,
    "        let expected = definition.derive_program(world)?;\n",
    "        let expected = definition.derive_program_with_seed(world, checkpoint.experiment.seed)?;\n",
)

# --- Observability: expose tie dependence and validate the exact keyed choice. ---
path = "crates/anthrosim-core/src/temporary_observability.rs"
replace_once(
    path,
    '''use std::{\n    cmp::Ordering,\n    collections::{BTreeMap, BTreeSet, BinaryHeap},\n};''',
    "use std::collections::{BTreeMap, BTreeSet};",
)
replace_once(
    path,
    '''    ids::{CellId, HouseholdId, TemporaryJourneyId},\n    temporary_travel::temporary_travel_edge_cost,\n};''',
    "    ids::{CellId, HouseholdId, TemporaryJourneyId},\n};",
)
replace_once(path, "    pub const CURRENT_SCHEMA_VERSION: u32 = 2;", "    pub const CURRENT_SCHEMA_VERSION: u32 = 3;")
replace_once(
    path,
    '''    pub route_distance_unavailable_journeys: u64,\n}\n''',
    '''    pub route_distance_unavailable_journeys: u64,\n    pub tied_destination_origin_cells: u64,\n    pub maximum_equal_cost_destination_count: u32,\n    pub journeys_started_from_tied_origins: u64,\n}\n''',
)
replace_once(
    path,
    '''            route_distance_unavailable_journeys: 0,\n        }''',
    '''            route_distance_unavailable_journeys: 0,\n            tied_destination_origin_cells: 0,\n            maximum_equal_cost_destination_count: 0,\n            journeys_started_from_tied_origins: 0,\n        }''',
)
replace_once(
    path,
    '''    pub one_way_route_distance_edges: Option<u32>,\n    pub people_at_departure: u32,''',
    '''    pub one_way_route_distance_edges: Option<u32>,\n    pub equal_cost_destination_count: u32,\n    pub people_at_departure: u32,''',
)
replace_once(
    path,
    '''    pub route_distance_unavailable_journeys: u64,\n}\n\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = "camelCase")]\npub struct TemporaryVisitDurationBin''',
    '''    pub route_distance_unavailable_journeys: u64,\n    pub equal_cost_destination_count: Option<u32>,\n}\n\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = "camelCase")]\npub struct TemporaryVisitDurationBin''',
)
replace_once(
    path,
    '''    summary: TemporaryMobilityObservabilitySummary,\n    current_visitors: u64,\n    route_distance_edges: Vec<Option<u32>>,\n}''',
    '''    summary: TemporaryMobilityObservabilitySummary,\n    current_visitors: u64,\n}''',
)
replace_once(
    path,
    '''        let expected = config.derive_program(world).map_err(|error| {''',
    '''        let expected = config\n            .derive_program_with_seed(world, checkpoint.experiment.seed)\n            .map_err(|error| {''',
)
# Fix indentation/closure changed by chained map_err replacement if needed below via rustfmt.
replace_once(
    path,
    '''    let route_distance_edges = derive_route_distances(program, world)?;\n    let mut households = Vec::with_capacity(initial_population.household_count());''',
    '''    let mut households = Vec::with_capacity(initial_population.household_count());''',
)
replace_once(
    path,
    '''        summary: TemporaryMobilityObservabilitySummary {\n            provenance: MetricProvenance::Derived,\n            observation_duration_days: end_day,\n            ..TemporaryMobilityObservabilitySummary::default()\n        },\n        current_visitors: 0,\n        route_distance_edges,\n    };''',
    '''        summary: TemporaryMobilityObservabilitySummary {\n            provenance: MetricProvenance::Derived,\n            observation_duration_days: end_day,\n            tied_destination_origin_cells: (1..=world.cell_count() as u64)\n                .filter(|raw| {\n                    program\n                        .travel\n                        .equal_cost_destination_count(CellId::new(*raw))\n                        .is_some_and(|count| count > 1)\n                })\n                .count()\n                .try_into()\n                .map_err(|_| invalid("tied destination origin count exceeds u64"))?,\n            maximum_equal_cost_destination_count: (1..=world.cell_count() as u64)\n                .filter_map(|raw| {\n                    program\n                        .travel\n                        .equal_cost_destination_count(CellId::new(raw))\n                })\n                .max()\n                .unwrap_or(0),\n            ..TemporaryMobilityObservabilitySummary::default()\n        },\n        current_visitors: 0,\n    };''',
)
replace_once(
    path,
    '''            route_distance_unavailable_journeys: row.route_distance_unavailable_journeys,\n        })''',
    '''            route_distance_unavailable_journeys: row.route_distance_unavailable_journeys,\n            equal_cost_destination_count: program\n                .travel\n                .equal_cost_destination_count(CellId::new(origin)),\n        })''',
)
# Departure validation gets household/trigger and route distance comes from authoritative candidate metadata.
replace_once(
    path,
    '''                validate_departure_against_program(\n                    replay,\n                    *residence,\n                    *destination,''',
    '''                validate_departure_against_program(\n                    replay,\n                    household,\n                    *trigger_index,\n                    *residence,\n                    *destination,''',
)
replace_once(
    path,
    '''                let route_distance =\n                    replay.route_distance_edges[cell_index(*residence, replay.cells.len())?];''',
    '''                let route_distance = replay\n                    .program\n                    .travel\n                    .route_distance_edges(*residence, *destination);\n                let equal_cost_destination_count = replay\n                    .program\n                    .travel\n                    .equal_cost_destination_count(*residence)\n                    .unwrap_or(1);''',
)
replace_once(
    path,
    '''                    one_way_route_distance_edges: route_distance,\n                    people_at_departure: *people_affected,''',
    '''                    one_way_route_distance_edges: route_distance,\n                    equal_cost_destination_count,\n                    people_at_departure: *people_affected,''',
)
replace_once(
    path,
    '''                replay.summary.journeys_started = add(replay.summary.journeys_started, 1)?;\n                replay.summary.people_at_departure = add(''',
    '''                replay.summary.journeys_started = add(replay.summary.journeys_started, 1)?;\n                if equal_cost_destination_count > 1 {\n                    replay.summary.journeys_started_from_tied_origins =\n                        add(replay.summary.journeys_started_from_tied_origins, 1)?;\n                }\n                replay.summary.people_at_departure = add(''',
)
replace_once(
    path,
    '''fn validate_departure_against_program(\n    replay: &Replay<'_>,\n    residence: CellId,''',
    '''fn validate_departure_against_program(\n    replay: &Replay<'_>,\n    household: HouseholdId,\n    trigger_index: u32,\n    residence: CellId,''',
)
replace_once(
    path,
    "    match replay.program.travel.resolution(residence) {\n",
    '''    match replay\n        .program\n        .travel\n        .resolution_for(residence, household, trigger_index)\n    {\n''',
)
# Remove obsolete route-distance Dijkstra block entirely.
sub_once(
    path,
    r"#\[derive\(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord\)\]\nstruct RouteLabel \{.*?\n\}\n\n#\[cfg\(test\)\]\nmod tests \{",
    "#[cfg(test)]\nmod tests {",
    flags=re.S,
)
# Test fixture struct needs the new field.
replace_once(
    path,
    '''            one_way_route_distance_edges: Some(4),\n            people_at_departure: 2,''',
    '''            one_way_route_distance_edges: Some(4),\n            equal_cost_destination_count: 1,\n            people_at_departure: 2,''',
)

# --- Provenance + public API. ---
path = "crates/anthrosim-core/src/provenance.rs"
replace_once(path, 'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v17";', 'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v18";')

path = "crates/anthrosim-core/src/lib.rs"
replace_once(
    path,
    '''    TemporaryMobilitySchedule, TemporaryMobilityState, TemporaryMobilityValidationError,\n    TemporaryTravelResolution, TemporaryTravelTable, TemporaryTriggerTiming,''',
    '''    TemporaryMobilitySchedule, TemporaryMobilityState, TemporaryMobilityValidationError,\n    TemporaryTravelDestinationCandidate, TemporaryTravelResolution, TemporaryTravelTable,\n    TemporaryTriggerTiming,''',
)

# Focused scientific contract.
Path("docs/research/m9-equal-cost-destination-choice-v1.md").write_text('''# M9 equal-cost destination choice semantics v1\n\n## Scientific problem\n\nM9.4 can find more than one focal-region destination with exactly the same minimum accumulated movement cost from an origin. Before model semantics v18, the routing table collapsed such a tie to the lower authoritative `CellId`. Because `CellId` follows grid storage order, that deterministic fallback created a causal spatial preference unrelated to the travel model.\n\n## v18 rule\n\nM9.4 now preserves the complete canonical set of exactly equal minimum-cost destinations for every origin. The set is sorted only for stable serialization; ordering is not the causal choice rule. Each candidate also retains the minimum route-edge count among paths achieving that same minimum cost.\n\nWhen a household actually evaluates a trigger from a tied origin, AnthroSim chooses one candidate using the versioned keyed policy `m9/equal-cost-destination-keyed-v1`. The key contains the experiment seed, origin cell, household identity and trigger index and is passed through a fixed integer avalanche before reduction to the candidate count.\n\nThis choice is deterministic and platform independent, but it consumes no sequential RNG draw. Therefore adding or removing a tied journey cannot shift M2, M3, M4 or other stochastic streams. Replaying the same experiment/household/trigger produces the same destination exactly.\n\nThe keyed policy is a neutral ambiguity resolver, not evidence that historical households chose destinations randomly. If evidence supports destination preference, that preference requires a separate explicit model.\n\n## Symmetry and interpretation\n\nNo candidate receives priority because it is north, west, first in row-major storage or lower in `CellId`. Across households/seeds, symmetric alternatives receive both outcomes under the keyed mapping. A single deterministic run can still contain sampling imbalance; that is stochastic/keyed realization, not a fixed directional rule.\n\nNon-tied minima are unchanged. Reachability, accumulated route cost and travel duration are unchanged by the tie key. Only which scientifically indistinguishable minimum-cost destination receives the visit can change.\n\n## Observability\n\nTemporary-mobility observability v3 reports:\n\n- the number of world origins with more than one equal-cost destination;\n- the maximum equal-cost destination count;\n- the number of started journeys whose origin was tied;\n- `equalCostDestinationCount` for each started journey and origin-catchment row.\n\nResearchers should inspect these fields before interpreting destination-level visitor concentration or resource pressure. A high tied-origin frequency means destination-level conclusions depend materially on the declared ambiguity policy even when total catchment participation is stable.\n\n## Provenance boundary\n\nThis changes authoritative M9 destination behavior and advances `MODEL_SEMANTICS_ID` from v17 to v18. It does not change travel-cost equations, travel-duration conversion, M4 migration decisions, mortality, resource allocation rules, or any sequential RNG stream.\n''')
