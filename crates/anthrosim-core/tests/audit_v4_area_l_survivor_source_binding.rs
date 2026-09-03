use anthrosim_core::StudyProtocol;
use serde_json::json;

#[test]
fn study_protocol_accepts_free_form_fake_mortality_source() {
    let mut protocol: serde_json::Value = serde_json::from_str(include_str!(
        "../../../examples/study-protocol-v1.json"
    ))
    .expect("example protocol parses");

    protocol["observables"] = json!([
        {
            "id": "survivor_condition",
            "role": "primary",
            "source": "metrics.resources.meanLivingConditionPermille",
            "analysisWindowId": "primary_window",
            "interpretation": "estimand=survivor_condition_at_boundary conditioning=survival death_handling=no_post_death_imputation"
        },
        {
            "id": "fake_joint_survival",
            "role": "secondary",
            "source": "derived.not_a_real_mortality_observable",
            "analysisWindowId": "primary_window",
            "interpretation": "This is intentionally not bound to a real survival or population output."
        }
    ]);
    protocol["comparisons"][0]["observableIds"] =
        json!(["survivor_condition", "fake_joint_survival"]);

    let decoded: StudyProtocol =
        serde_json::from_value(protocol).expect("free-form source remains schema-valid");
    decoded
        .validate()
        .expect("StudyProtocol validation accepts non-empty free-form source labels");

    println!("audit_v4_area_l_fake_source_study_protocol_valid=true");
}
