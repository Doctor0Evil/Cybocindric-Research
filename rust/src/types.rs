use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub unit: String,
    pub domain_min: f64,
    pub domain_max: f64,
    pub legal_limit: Option<f64>,
    pub gold_limit: Option<f64>,
    pub direction_max: bool, // true = MAX, false = MIN
}

#[derive(Clone, Debug)]
pub struct RiskCoordinateDef {
    pub id: u32,
    pub param_name: String,
    pub r_min: f64,
    pub r_max: f64,
    pub weight_w: f64,
    pub channel: u32,
}

#[derive(Clone, Debug)]
pub struct LcaScenario {
    pub scenario_id: String,
    pub region_id: String,
    pub functional_unit: String, // "MSW_TON", "ENERGY_MWH", "RESOURCE_KG"
    pub mode: String,            // "STATUS_QUO" or "CYBOCINDER"
    pub gwp_kg_co2eq: f64,
    pub grid_gco2_per_kwh: f64,
    pub landfill_ref_gwp_kgco2_per_ton: f64,
    pub avoided_virgin_metal_kgco2eq_per_kg: f64,
    pub energy_recovery_efficiency: f64,
    pub recycling_rate: f64,
}
