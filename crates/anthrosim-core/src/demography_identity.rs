use crate::config::{AgeProbabilityBand, DemographyConfig};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const SCHEDULE_IDENTITY_DOMAIN: &[u8] = b"anthrosim-demography-schedule-v1";
const SCHEDULE_ID_PREFIX: &str = "anthrosim-demography-schedule-v1-";

impl DemographyConfig {
    /// Deterministic identity of the complete executable demographic schedule.
    ///
    /// The identity deliberately excludes `schedule_id` itself to avoid circularity and excludes
    /// provenance because provenance describes the epistemic status of the schedule rather than
    /// its executable content. Every field consumed by the demographic engine is included.
    ///
    /// This is a compact reproducibility/integrity identity, not a cryptographic authenticity seal.
    #[must_use]
    pub fn schedule_content_digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_bytes(&mut hash, SCHEDULE_IDENTITY_DOMAIN);
        digest_u32(&mut hash, self.schema_version);
        digest_bands(&mut hash, &self.mortality_bands);
        digest_bands(&mut hash, &self.fertility_bands);
        digest_u32(&mut hash, self.minimum_birth_spacing_days);
        digest_u16(&mut hash, self.male_birth_permille);
        digest_u32(&mut hash, self.male_parent_min_age_years);
        digest_u32(&mut hash, self.male_parent_max_age_years_exclusive);
        hash
    }

    /// Canonical whole-schedule identifier accepted by the research evidence-closure gate.
    #[must_use]
    pub fn content_bound_schedule_id(&self) -> String {
        format!("{SCHEDULE_ID_PREFIX}{:016x}", self.schedule_content_digest64())
    }

    /// Bind `schedule_id` to the current executable schedule contents.
    ///
    /// Call this after all schedule fields have been configured. Later mutation of any executable
    /// schedule field makes the identifier stale and therefore fails research evidence closure.
    #[must_use]
    pub fn with_content_bound_schedule_id(mut self) -> Self {
        self.schedule_id = self.content_bound_schedule_id();
        self
    }

    /// Whether the persisted identifier exactly matches the current executable schedule contents.
    #[must_use]
    pub fn has_content_bound_schedule_id(&self) -> bool {
        self.schedule_id == self.content_bound_schedule_id()
    }
}

fn digest_bands(hash: &mut u64, bands: &[AgeProbabilityBand]) {
    digest_u64(
        hash,
        u64::try_from(bands.len()).expect("demographic schedule band count must fit u64"),
    );
    for band in bands {
        digest_u32(hash, band.start_age_years);
        digest_u32(hash, band.end_age_years_exclusive);
        digest_u32(hash, band.annual_probability_per_million);
    }
}

fn digest_bytes(hash: &mut u64, bytes: &[u8]) {
    digest_u64(
        hash,
        u64::try_from(bytes.len()).expect("demographic identity byte length must fit u64"),
    );
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn digest_u16(hash: &mut u64, value: u16) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn digest_u32(hash: &mut u64, value: u32) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_bound_schedule_id_is_deterministic() {
        let first = DemographyConfig::synthetic_validation_v1();
        let second = DemographyConfig::synthetic_validation_v1();
        assert_eq!(
            first.content_bound_schedule_id(),
            second.content_bound_schedule_id()
        );
    }

    #[test]
    fn executable_schedule_changes_invalidate_bound_identity() {
        let bound = DemographyConfig::synthetic_validation_v1().with_content_bound_schedule_id();
        assert!(bound.has_content_bound_schedule_id());

        let mut mortality = bound.clone();
        mortality.mortality_bands[0].annual_probability_per_million -= 1;
        assert!(!mortality.has_content_bound_schedule_id());

        let mut fertility = bound.clone();
        fertility.fertility_bands[1].annual_probability_per_million -= 1;
        assert!(!fertility.has_content_bound_schedule_id());

        let mut spacing = bound.clone();
        spacing.minimum_birth_spacing_days += 1;
        assert!(!spacing.has_content_bound_schedule_id());

        let mut sex_ratio = bound.clone();
        sex_ratio.male_birth_permille -= 1;
        assert!(!sex_ratio.has_content_bound_schedule_id());

        let mut parent_age = bound;
        parent_age.male_parent_max_age_years_exclusive += 1;
        assert!(!parent_age.has_content_bound_schedule_id());
    }

    #[test]
    fn legacy_synthetic_label_is_not_accidentally_content_bound() {
        let synthetic = DemographyConfig::synthetic_validation_v1();
        assert!(!synthetic.has_content_bound_schedule_id());
    }
}
