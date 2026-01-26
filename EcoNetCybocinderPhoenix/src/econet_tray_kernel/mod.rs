// src/econet_tray_kernel/mod.rs
// Hex-stamp placeholder for QPU/ALN authorship anchoring:
// 0xeca1_7f9b_42de_9012_77aa_55cc_3399_1188

#![forbid(unsafe_code)]

use std::time::Duration;

/// Dimensionless risk coordinate r_x ∈ [0,1], plus metadata.
/// Aligned with ecosafety grammar: rx, corridor bands, Lyapunov channels.[file:18][file:27]
#[derive(Clone, Debug)]
pub struct RiskCoord {
    pub var_id: String,
    pub value: f64,        // 0.0 = fully safe, 1.0 = at hard limit
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,       // contribution weight into V_t
    pub lyap_channel: u16, // channel index for residual aggregation
}

/// Lyapunov-style residual V_t over a set of risk coordinates.[file:18][file:27]
#[derive(Clone, Debug)]
pub struct Residual {
    pub vt: f64,
    pub weights: Vec<f64>,
    pub rx: Vec<RiskCoord>,
}

impl Residual {
    /// Recompute V_t = Σ w_j * r_j from internal rx.[file:18]
    pub fn recompute(&mut self) {
        let mut v = 0.0;
        for (j, rc) in self.rx.iter().enumerate() {
            let w = self.weights.get(j).copied().unwrap_or(0.0);
            v += w * rc.value;
        }
        self.vt = v;
    }
}

/// Decision returned by ecosafety shell when a new operating point is proposed.[file:18]
#[derive(Clone, Debug)]
pub struct CorridorDecision {
    pub derate: bool,
    pub stop: bool,
    pub reason: String,
}

/// Hard ecosafety contract: no corridor, no build; violated corridor => derate/stop.[file:18][file:27]
pub fn enforce_safestep(prev: &Residual, next: &Residual) -> CorridorDecision {
    // Any r_x ≥ 1.0 violates a hard limit.
    if next.rx.iter().any(|rc| rc.value >= 1.0) {
        return CorridorDecision {
            derate: true,
            stop: true,
            reason: "hard corridor limit exceeded".to_string(),
        };
    }

    // Lyapunov residual must not increase outside the safe interior.[file:18]
    if next.vt > prev.vt {
        return CorridorDecision {
            derate: true,
            stop: false,
            reason: "Lyapunov residual increased".to_string(),
        };
    }

    CorridorDecision {
        derate: false,
        stop: false,
        reason: "within corridors".to_string(),
    }
}

/// RegionConfig: container for all Phoenix/region-specific parameters.[file:30][file:20]
pub trait RegionConfig {
    fn region_code(&self) -> &str;

    // Compost environment (Phoenix-first, globally portable).
    fn compost_temp_range_c(&self) -> (f64, f64);
    fn compost_moisture_frac(&self) -> (f64, f64);
    fn compost_oxygen_min_percent(&self) -> f64;

    // Water quality baselines (pH, TDS, PFBS, E. coli refs, etc.).[file:30][file:20]
    fn canal_ph_range(&self) -> (f64, f64);
    fn canal_tds_mg_l(&self) -> (f64, f64);
    fn cref_tray_residual_mg_l(&self) -> f64; // C_ref for TRAYRESIDUAL in CEIM K_n.[file:30][file:27]

    // Hydropower envelope coefficients for CAP canal segments.[file:20][file:30]
    fn hydro_water_density_kg_m3(&self) -> f64;
    fn hydro_cp_efficiency(&self) -> f64;
    fn hydro_default_area_m2(&self) -> f64;
    fn hydro_default_velocity_m_s(&self) -> f64;

    // Biodegradation target bands for t90.[file:30]
    fn t90_target_days(&self) -> f64;
    fn t90_hard_limit_days(&self) -> f64;

    // Toxicity corridor bands (r_tox ≤ 0.1).[file:30][file:27]
    fn rtox_safe(&self) -> f64;
    fn rtox_gold(&self) -> f64;
    fn rtox_hard(&self) -> f64;

    // Eco-impact weighting (e.g., Karma per kg plastic avoided / tray residual).[file:30][file:27]
    fn karma_per_kg_tray_residual_avoided(&self) -> f64;
}

/// Phoenix implementation of RegionConfig (phoenix_az profile).[file:30][file:20]
#[derive(Clone, Debug)]
pub struct PhoenixAzConfig;

impl RegionConfig for PhoenixAzConfig {
    fn region_code(&self) -> &str {
        "Phoenix-AZ-US"
    }

    fn compost_temp_range_c(&self) -> (f64, f64) {
        // Active compost in Phoenix ~45–60 °C.[file:30]
        (45.0, 60.0)
    }

    fn compost_moisture_frac(&self) -> (f64, f64) {
        (0.45, 0.65)
    }

    fn compost_oxygen_min_percent(&self) -> f64 {
        10.0
    }

    fn canal_ph_range(&self) -> (f64, f64) {
        (7.2, 8.3)
    }

    fn canal_tds_mg_l(&self) -> (f64, f64) {
        (500.0, 900.0)
    }

    fn cref_tray_residual_mg_l(&self) -> f64 {
        // Example C_ref for TRAYRESIDUAL mass-load impact in CEIM.[file:27]
        50.0
    }

    fn hydro_water_density_kg_m3(&self) -> f64 {
        1000.0
    }

    fn hydro_cp_efficiency(&self) -> f64 {
        0.4
    }

    fn hydro_default_area_m2(&self) -> f64 {
        2.0
    }

    fn hydro_default_velocity_m_s(&self) -> f64 {
        2.0
    }

    fn t90_target_days(&self) -> f64 {
        90.0
    }

    fn t90_hard_limit_days(&self) -> f64 {
        180.0
    }

    fn rtox_safe(&self) -> f64 {
        0.05
    }

    fn rtox_gold(&self) -> f64 {
        0.10
    }

    fn rtox_hard(&self) -> f64 {
        0.20
    }

    fn karma_per_kg_tray_residual_avoided(&self) -> f64 {
        6.7e5
    }
}

/// EnvFeed trait abstracts environmental data providers (static config vs telemetry).[file:30][file:18]
pub trait EnvFeed {
    fn region(&self) -> &dyn RegionConfig;

    fn compost_temp_c(&self) -> f64;
    fn compost_moisture_frac(&self) -> f64;
    fn compost_oxygen_percent(&self) -> f64;

    fn canal_velocity_m_s(&self) -> f64;
    fn canal_area_m2(&self) -> f64;
    fn canal_ph(&self) -> f64;
    fn canal_tds_mg_l(&self) -> f64;
}

/// StaticConfigFeed uses only RegionConfig; no live telemetry.[file:30]
#[derive(Clone, Debug)]
pub struct StaticConfigFeed<R: RegionConfig + Clone> {
    pub region_cfg: R,
}

impl<R: RegionConfig + Clone> EnvFeed for StaticConfigFeed<R> {
    fn region(&self) -> &dyn RegionConfig {
        &self.region_cfg
    }

    fn compost_temp_c(&self) -> f64 {
        let (tmin, tmax) = self.region_cfg.compost_temp_range_c();
        0.5 * (tmin + tmax)
    }

    fn compost_moisture_frac(&self) -> f64 {
        let (mmin, mmax) = self.region_cfg.compost_moisture_frac();
        0.5 * (mmin + mmax)
    }

    fn compost_oxygen_percent(&self) -> f64 {
        self.region_cfg.compost_oxygen_min_percent()
    }

    fn canal_velocity_m_s(&self) -> f64 {
        self.region_cfg.hydro_default_velocity_m_s()
    }

    fn canal_area_m2(&self) -> f64 {
        self.region_cfg.hydro_default_area_m2()
    }

    fn canal_ph(&self) -> f64 {
        let (pmin, pmax) = self.region_cfg.canal_ph_range();
        0.5 * (pmin + pmax)
    }

    fn canal_tds_mg_l(&self) -> f64 {
        let (tmin, tmax) = self.region_cfg.canal_tds_mg_l();
        0.5 * (tmin + tmax)
    }
}

/// Placeholder TelemetryFeed interface; in Phase 1 this can be left unimplemented.[file:30][file:20]
pub struct TelemetryFeed<R: RegionConfig + Clone> {
    pub region_cfg: R,
    // Future: sensor channels, message buses, etc.
}

impl<R: RegionConfig + Clone> EnvFeed for TelemetryFeed<R> {
    fn region(&self) -> &dyn RegionConfig {
        &self.region_cfg
    }

    fn compost_temp_c(&self) -> f64 {
        unimplemented!("Hook real compost temperature sensor here")
    }

    fn compost_moisture_frac(&self) -> f64 {
        unimplemented!("Hook real compost moisture sensor here")
    }

    fn compost_oxygen_percent(&self) -> f64 {
        unimplemented!("Hook real compost O2 sensor here")
    }

    fn canal_velocity_m_s(&self) -> f64 {
        unimplemented!("Hook canal flowmeter here")
    }

    fn canal_area_m2(&self) -> f64 {
        unimplemented!("Hook canal geometry model here")
    }

    fn canal_ph(&self) -> f64 {
        unimplemented!("Hook canal pH probe here")
    }

    fn canal_tds_mg_l(&self) -> f64 {
        unimplemented!("Hook canal TDS probe here")
    }
}

/// Basic tray material recipe descriptor; mirrored into qpudatashards.[file:30][file:27]
#[derive(Clone, Debug)]
pub struct TrayMaterialMix {
    pub id: String,                // e.g., "AR-PHX-LAB-01"
    pub description: String,       // "70% bagasse 25% starch 5% clay"
    pub fiber_frac: f64,           // 0–1
    pub starch_frac: f64,          // 0–1
    pub protein_frac: f64,         // 0–1
    pub mineral_frac: f64,         // 0–1
}

/// Result of a biodegradation + toxicity simulation for one recipe.[file:30]
#[derive(Clone, Debug)]
pub struct TraySimResult {
    pub material_id: String,
    pub region_code: String,

    // Time-to-90% mass loss.
    pub modeled_t90_days: f64,

    // Relative toxicity r_tox; dimensionless.
    pub r_tox: f64,

    // EcoNet-aligned scores K, E, R in 0–1.[file:27]
    pub knowledge_factor: f64,
    pub ecoimpact_score: f64,
    pub risk_of_harm: f64,

    // Derived eco-metrics.
    pub waste_reduced_kg_per_cycle: f64,
    pub energy_kwh_per_cycle: f64,
}

/// Trait for anything that can be evaluated into eco-impact scores.[file:27][file:18]
pub trait EcoScorable {
    fn score(&self, region: &dyn RegionConfig) -> TraySimResult;
}

/// Simple biodegradation kinetics: first-order with k fitted to Phoenix compost band.[file:30][file:18]
fn estimate_t90_days_from_mix<E: EnvFeed>(mix: &TrayMaterialMix, env: &E) -> f64 {
    let temp = env.compost_temp_c();

    // Baseline k at 25 °C for starch-rich blends ~0.05 d⁻¹, adjusted via Q10.[file:30]
    let k_base = 0.05;
    let q10 = 2.0;
    let delta_t = temp - 25.0;
    let k = k_base * q10.powf(delta_t / 10.0);

    // t90 ≈ ln(10) / k.[file:30]
    let t90 = (10.0_f64.ln()) / k;

    // Minor adjustment: more minerals -> slower decay.
    let mineral_penalty = 1.0 + 0.5 * mix.mineral_frac;
    t90 * mineral_penalty
}

/// Crude toxicity proxy using mineral / binder fractions; real LC-MS will replace this.[file:30][file:18]
fn estimate_rtox_from_mix(mix: &TrayMaterialMix, region: &dyn RegionConfig) -> f64 {
    let base = 0.02
        + 0.05 * mix.mineral_frac
        + 0.01 * mix.protein_frac.max(0.0);
    let safe = region.rtox_safe();
    let gold = region.rtox_gold();
    let hard = region.rtox_hard();

    if base <= safe {
        0.0
    } else if base >= hard {
        1.0
    } else {
        // Linear scaling between safe and hard.
        (base - safe) / (hard - safe)
    }
}

/// A candidate Phoenix tray recipe that can be eco-scored.[file:30][file:27]
#[derive(Clone, Debug)]
pub struct PhoenixTrayCandidate<E: EnvFeed> {
    pub mix: TrayMaterialMix,
    pub env: E,
    pub waste_reduced_kg_per_cycle: f64,
    pub energy_kwh_per_cycle: f64,
    pub knowledge_factor: f64,
}

impl<E: EnvFeed> EcoScorable for PhoenixTrayCandidate<E> {
    fn score(&self, region: &dyn RegionConfig) -> TraySimResult {
        let modeled_t90 = estimate_t90_days_from_mix(&self.mix, &self.env);
        let r_tox = estimate_rtox_from_mix(&self.mix, region);

        // Primary gates: t90 ≤ hard limit, r_tox ≤ 0.1 corridor.[file:30]
        let mut risk_of_harm = 0.0;
        if modeled_t90 > region.t90_hard_limit_days() {
            risk_of_harm = 1.0;
        }
        if r_tox > region.rtox_gold() {
            risk_of_harm = risk_of_harm.max(1.0);
        }

        // Eco-impact E: scaled by waste avoided, clamped to [0,1].[file:30][file:27]
        let e_raw = self.waste_reduced_kg_per_cycle / 300.0; // 300 kg/cycle ~ typical Phoenix tray node.[file:30]
        let ecoimpact_score = e_raw.clamp(0.0, 1.0);

        // If any hard gate fails, ecoimpact collapses to zero.[file:27]
        let ecoimpact_final = if risk_of_harm >= 1.0 { 0.0 } else { ecoimpact_score };

        TraySimResult {
            material_id: self.mix.id.clone(),
            region_code: region.region_code().to_string(),
            modeled_t90_days: modeled_t90,
            r_tox,
            knowledge_factor: self.knowledge_factor,
            ecoimpact_score: ecoimpact_final,
            risk_of_harm,
            waste_reduced_kg_per_cycle: self.waste_reduced_kg_per_cycle,
            energy_kwh_per_cycle: self.energy_kwh_per_cycle,
        }
    }
}

/// Hydropower P = 0.5 ρ A v³ C_p; used to check canal-powered tray lines.[file:20][file:30]
pub fn canal_hydro_power_kw<E: EnvFeed>(env: &E) -> f64 {
    let region = env.region();
    let rho = region.hydro_water_density_kg_m3();
    let a = env.canal_area_m2();
    let v = env.canal_velocity_m_s();
    let cp = region.hydro_cp_efficiency();

    let p_watts = 0.5 * rho * a * v.powi(3) * cp;
    p_watts / 1000.0
}

/// One row of a qpudatashard for AntRecyclingBioPackPhoenix2026v1.csv-like outputs.[file:30][file:27]
#[derive(Clone, Debug)]
pub struct QpuTrayShardRow {
    pub machine_id: String,
    pub facility: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,
    pub materialmix: String,
    pub target_t90_days: f64,
    pub modeled_t90_days: f64,
    pub iso14851_class: String,
    pub ecoimpact_score: f64,
    pub waste_reduced_kg_per_cycle: f64,
    pub tox_risk_corridor: f64,
    pub energy_kwh_per_cycle: f64,
}

/// Map a TraySimResult into a shard row (Phoenix profile).[file:30][file:27]
pub fn to_qpu_tray_shard_row(
    machine_id: &str,
    facility: &str,
    lat: f64,
    lon: f64,
    mix: &TrayMaterialMix,
    region: &dyn RegionConfig,
    sim: &TraySimResult,
) -> QpuTrayShardRow {
    let iso_class = if sim.modeled_t90_days <= region.t90_target_days() {
        "Phoenix-ISO14851-StrongPass".to_string()
    } else if sim.modeled_t90_days <= region.t90_hard_limit_days() {
        "Phoenix-ISO14851-Pass".to_string()
    } else {
        "Phoenix-ISO14851-Fail".to_string()
    };

    QpuTrayShardRow {
        machine_id: machine_id.to_string(),
        facility: facility.to_string(),
        region: region.region_code().to_string(),
        lat,
        lon,
        materialmix: mix.description.clone(),
        target_t90_days: region.t90_target_days(),
        modeled_t90_days: sim.modeled_t90_days,
        iso14851_class: iso_class,
        ecoimpact_score: sim.ecoimpact_score,
        waste_reduced_kg_per_cycle: sim.waste_reduced_kg_per_cycle,
        tox_risk_corridor: sim.r_tox,
        energy_kwh_per_cycle: sim.energy_kwh_per_cycle,
    }
}

/// Simple batch simulation harness for Phoenix recipes.
/// Phase 1: pure static-config runs, writing CSV-compatible shard rows.[file:30][file:18]
pub fn simulate_tray_recipes_phoenix(
    recipes: Vec<(TrayMaterialMix, String, f64, f64)>, // (mix, facility, lat, lon)
    waste_reduced_kg_per_cycle: f64,
    energy_kwh_per_cycle: f64,
    knowledge_factor: f64,
) -> Vec<QpuTrayShardRow> {
    let region_cfg = PhoenixAzConfig;
    let env = StaticConfigFeed { region_cfg };
    let region_ref: &dyn RegionConfig = env.region();

    recipes
        .into_iter()
        .map(|(mix, facility, lat, lon)| {
            let candidate = PhoenixTrayCandidate {
                mix: mix.clone(),
                env: env.clone(),
                waste_reduced_kg_per_cycle,
                energy_kwh_per_cycle,
                knowledge_factor,
            };

            let sim = candidate.score(region_ref);

            to_qpu_tray_shard_row(
                &mix.id,
                &facility,
                lat,
                lon,
                &mix,
                region_ref,
                &sim,
            )
        })
        .collect()
}

/// Utility to render QpuTrayShardRows as CSV lines (header + rows).
/// Caller is responsible for writing to filesystem.[file:30]
pub fn qpu_tray_shard_to_csv(rows: &[QpuTrayShardRow]) -> String {
    let mut out = String::new();
    out.push_str("machineid,facility,region,lat,lon,materialmix,target_t90_days,modeled_t90_days,iso14851_class,ecoimpact_score,waste_reduced_kg_per_cycle,tox_risk_corridor,energy_kwh_per_cycle\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{:.4},{:.4},\"{}\",{:.1},{:.1},{},{:.3},{:.1},{:.3},{:.2}\n",
            r.machine_id,
            r.facility,
            r.region,
            r.lat,
            r.lon,
            r.materialmix.replace('"', "'"),
            r.target_t90_days,
            r.modeled_t90_days,
            r.iso14851_class,
            r.ecoimpact_score,
            r.waste_reduced_kg_per_cycle,
            r.tox_risk_corridor,
            r.energy_kwh_per_cycle
        ));
    }
    out
}
