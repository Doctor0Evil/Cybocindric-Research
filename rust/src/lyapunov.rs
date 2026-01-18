use crate::types::{Parameter, RiskCoordinateDef};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RiskCoord {
    pub r: f64,
    pub w: f64,
}

#[derive(Clone, Debug)]
pub struct ResidualState {
    pub coords: Vec<RiskCoord>,
    pub v: f64,
}

fn clip01(x: f64) -> f64 {
    if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
}

pub fn compute_risk_coord(
    param: &Parameter,
    rc_def: &RiskCoordinateDef,
    x: f64,
) -> RiskCoord {
    let denom = rc_def.r_max - rc_def.r_min;
    assert!(denom > 0.0, "invalid r_min/r_max");
    let raw = if param.direction_max {
        (x - rc_def.r_min) / denom
    } else {
        (rc_def.r_max - x) / denom
    };
    let r = clip01(raw);
    assert!(rc_def.weight_w >= 0.0, "negative weight");
    RiskCoord { r, w: rc_def.weight_w }
}

pub fn compute_residual(coords: &[RiskCoord]) -> ResidualState {
    assert!(!coords.is_empty(), "no risk coordinates (no corridor -> no deployment)");
    let mut v = 0.0;
    for c in coords {
        v += c.r * c.w;
    }
    ResidualState { coords: coords.to_vec(), v }
}

pub fn is_admissible(v_prev: f64, v_next: f64, eps: f64) -> bool {
    v_next <= v_prev + eps
}
