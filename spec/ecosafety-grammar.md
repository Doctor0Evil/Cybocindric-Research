# EcoNetCybocinderPhoenix Ecosafety Grammar

## 1. K layer – Knowledge objects

### 1.1 Parameters
- Definition of `Parameter` (name, unit, domain, legal_limit, gold_limit, direction).
- Constraint: parameters used in corridors or LCA must have all fields set.

### 1.2 Risk coordinates
- Definition of `RiskCoordinate` (id, param_name, r_min, r_max, weight_w, channel).
- Normalization rules for \(r_{x_j}(t)\) for direction = MAX/MIN, with clip to \([0,1]\).

### 1.3 Lyapunov residual
- State vector \(R(t)\), weight vector \(W\), residual
  \[
  V_t = \sum_{j=0}^{n-1} w_j r_{x_j}(t)
  \]

### 1.4 LCA kernel entities
- `Scenario` (scenario_id, region_id, functional_unit, description, GWP_kgCO2eq, other_impacts).
- Avoided‑burden parameters (grid_gCO2_per_kWh, landfill_ref_GWP_kgCO2eq_per_ton, etc.).

## 2. E layer – Execution / behavior

### 2.1 Control layers
- Fast/medium/slow layers, shared use of \(r_{x_j}\) and \(V_t\).
- Signature of `is_admissible_move(State, Move, V_t) -> bool`.

### 2.2 Lyapunov admissibility contract
- Corridor consistency.
- \(V_{t+1} \le V_t + \varepsilon\) condition.
- Hard violation rule: any \(r_{x_j}(t+1) > 1\) or residual increase → reject + derate/stop.

### 2.3 Dual‑threshold behavior
- Legal constraint: \(x_j(t) \le C_{\text{reg},j}\).
- Gold constraint in SCALE_UP/BONUS modes: \(x_j(t) \le C_{\text{gold},j}\).

### 2.4 LCA gate
- For each region + functional unit:
  \[
  \text{GWP}_{\text{cybo}} < \text{GWP}_{\text{base}}
  \]
  plus optional must‑improve co‑impacts.

## 3. R layer – Residuals / gates

### 3.1 Residual state
- R_state, V_state, corridor_ok, legal_ok, gold_ok.

### 3.2 Gate predicates
- `SafetyGate`, `ScaleUpGate`, `DeploymentGate` definitions and semantics.

## 4. Shard bindings

- Corridor spec shard schema.
- LCA scenario shard schema.
- Telemetry/residual shard schema.

## 5. K/E/R scores

- Definitions and formulas for K, E, R, and how they are logged in governance shards.
