#[derive(Clone, Debug)]
pub struct ResidualFlags {
    pub corridor_ok: bool,
    pub legal_ok: bool,
    pub gold_ok: bool,
}

#[derive(Clone, Debug)]
pub struct GateResult {
    pub safety_gate: bool,
    pub scaleup_gate: bool,
    pub deployment_gate: bool,
}

pub fn compute_gates(
    flags: &ResidualFlags,
    v_prev: f64,
    v_next: f64,
    eps: f64,
    lca_ok: bool,
    pilot_gates_ok: bool,
) -> GateResult {
    let safety = flags.corridor_ok && flags.legal_ok && (v_next <= v_prev + eps);
    let scaleup = safety && flags.gold_ok && lca_ok;
    let deploy = lca_ok && pilot_gates_ok;
    GateResult {
        safety_gate: safety,
        scaleup_gate: scaleup,
        deployment_gate: deploy,
    }
}
