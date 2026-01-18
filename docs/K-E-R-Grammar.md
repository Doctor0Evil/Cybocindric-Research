## K layer – Knowledge objects

### K.1 Core entities

**K.1.1 Parameter**

A *parameter* is any physically measurable or derived quantity relevant to safety, performance, or LCA (e.g., NOx, PM₂.₅, furnace temperature, grid carbon intensity). Each parameter has:

- `name`: globally unique string identifier (e.g., `"NOx_stack"`)  
- `unit`: UCUM‑compatible string (e.g., `"mg/Nm3"`, `"degC"`)  
- `domain`: continuous or discrete value space (e.g., \([0,+\infty)\))  
- `legal_limit`: scalar upper bound required by regulation (may be `null` if N/A)  
- `gold_limit`: scalar upper bound from health/LCA standard (e.g., WHO guideline)  
- `direction`: `"MAX"` (safety if ≤ limit) or `"MIN"` (safety if ≥ limit; e.g., residence time)  

These fields are **mandatory** for any parameter used in corridors or LCA; missing fields mean the parameter cannot be used in control or deployment logic. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

**K.1.2 Risk coordinate**

For each safety‑critical parameter \(x_j\), define a *risk coordinate*:

- `id`: integer index \(j\)  
- `param_name`: reference to `Parameter.name`  
- `r_min`: lower bound of “safe” physical range for normalization  
- `r_max`: upper bound of “corridor” physical range for normalization  
- `weight_w`: non‑negative scalar weight \(w_j\) in the residual  
- `channel`: integer index used by controllers (e.g., `lyap_channel`)  

The normalized risk coordinate at time \(t\) is:

- If `direction = "MAX"`:  
  \[
  r_{x_j}(t) = \mathrm{clip}_{[0,1]}\left(\frac{x_j(t) - r_{\min}}{r_{\max} - r_{\min}}\right)
  \]
- If `direction = "MIN"`:  
  \[
  r_{x_j}(t) = \mathrm{clip}_{[0,1]}\left(\frac{r_{\max} - x_j(t)}{r_{\max} - r_{\min}}\right)
  \]

`clip` enforces the \([0,1]\) range. By convention:

- \(r_{x_j} = 0\) means *ideal* within corridor.  
- \(r_{x_j} = 1\) means *boundary violation* (or worse). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

**K.1.3 Lyapunov residual**

The *Lyapunov residual* \(V_t\) is defined on the set of risk coordinates:

- State vector: \(R(t) = [r_{x_0}(t), \ldots, r_{x_{n-1}}(t)]\)  
- Weight vector: \(W = [w_0, \ldots, w_{n-1}]\), \(w_j \ge 0\)  

Residual:

\[
V_t = \sum_{j=0}^{n-1} w_j\, r_{x_j}(t)
\]

This is a dimensionless scalar in \([0, +\infty)\); in practice \(V_t \in [0, \sum_j w_j]\). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/97c69cfa-ed31-402b-935d-aa24257e605f/1-which-specific-topics-or-ris-eHRAYKP7Qg.OEhXKF519cg.md)

### K.2 LCA kernel entities

**K.2.1 Scenario**

An *LCA scenario* is defined for a given region and functional unit:

- `scenario_id`: unique string  
- `region_id`: code for geographical / grid region  
- `functional_unit`: enum `{ "MSW_TON", "ENERGY_MWH", "RESOURCE_KG" }`  
- `description`: free text  
- `GWP_kgCO2eq`: scalar total GWP per functional unit  
- `other_impacts`: optional additional metrics (e.g., acidification, PM, ecotoxicity)  

Two scenarios must exist for any deployment decision:

- `status_quo` (baseline)  
- `cybocinder` (candidate) [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

**K.2.2 Avoided‑burden parameters**

Avoided burdens are explicit parameters in the shard:

- `grid_gCO2_per_kWh`  
- `landfill_ref_GWP_kgCO2eq_per_ton`  
- `avoided_virgin_metal_kgCO2eq_per_kg`  
- `energy_recovery_efficiency` (net)  
- `recycling_rate`  

They are used by the LCA model to derive `GWP_kgCO2eq` but are stored explicitly so they can be audited and updated as grids or markets change. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

***

## E layer – Execution / behavior

### E.1 Control moves and timescales

Define three execution layers:

- **Fast layer** (combustion, 10 ms – minutes): controls air, fuel feed, dampers  
- **Medium layer** (scheduling, minutes – hours): controls feed mix, load setpoint  
- **Slow layer** (maintenance/fouling, weeks – seasons): controls cleaning schedules, inspections  

Each layer has:

- `state`: set of internal state variables (layer‑specific)  
- `inputs`: current measured or estimated values of \(x_j(t)\)  
- `outputs`: proposed control moves (e.g., new setpoints)  

All layers must compute a candidate next residual \(V_{t+1}^\mathrm{cand}\) using the **same** K‑layer definition of \(r_{x_j}\) and \(w_j\). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

### E.2 Lyapunov admissibility contract

A *control move* \(\Delta u\) is admissible **only if** it satisfies:

1. Corridor consistency:  
   All updated risk coordinates \(r_{x_j}(t+1)\) are computed from current or predicted \(x_j(t+1)\) via the normalization rules above.  

2. Lyapunov non‑increase:  
   \[
   V_{t+1} \le V_t + \varepsilon
   \]
   where \(\varepsilon \ge 0\) is a small numerical tolerance.

3. Hard violation handling:  
   If any \(r_{x_j}(t+1) > 1\) or `LyapunovNonIncrease` is false, the move is rejected and the furnace enters a derate or shutdown path defined in the corridor spec. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/97c69cfa-ed31-402b-935d-aa24257e605f/1-which-specific-topics-or-ris-eHRAYKP7Qg.OEhXKF519cg.md)

In implementation, the PLC (or C++ control core) exposes a function:

```text
bool is_admissible_move(State s_t, Move delta_u, double V_t) -> bool
```

which must internally compute \(V_{t+1}\) and return `true` only if all invariants hold.

### E.3 Dual‑threshold behavior

For parameters with both legal and gold limits:

- Legal constraint (always):  
  \[
  x_j(t) \le C_{\text{reg},j}
  \]
- Gold constraint (for scale‑up / bonus modes):  
  When `Mode ∈ { SCALE_UP, BONUS }`:
  \[
  x_j(t) \le C_{\text{gold},j}
  \]

Execution layer rules:

- If legal bound is violated, immediate derate/stop and alarm.  
- If gold bound is violated while in `SCALE_UP` or `BONUS`, the mode must be downgraded and scale‑up actions blocked. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

### E.4 LCA gate contract

For each region and functional unit:

- Let
  - \( \text{GWP}_{\text{base}} = \text{GWP}_{\text{status\_quo}} \)  
  - \( \text{GWP}_{\text{cybo}} = \text{GWP}_{\text{cybocinder}} \)

The deployment contract is:

- **Primary LCA invariant:**  
  \[
  \text{GWP}_{\text{cybo}} < \text{GWP}_{\text{base}}
  \]
- **Optional stronger condition:**  
  For each additional impact metric \(I\) marked *must‑improve*:
  \[
  I_{\text{cybo}} \le I_{\text{base}}
  \]

If any required invariant fails for any functional unit, deployment and replication are disallowed for that region. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

***

## R layer – Risk / residuals / gates

### R.1 Residual definition

The **residual** is the tuple:

- `R_state`: current normalized vector \(R(t)\)  
- `V_state`: scalar \(V_t\)  
- `corridor_ok`: boolean flag (all \(r_{x_j} \le 1\))  
- `legal_ok`: boolean (all legal thresholds met)  
- `gold_ok`: boolean (all gold thresholds met, where active)  

These fields are computed at each control step and logged to qpudatashards at a configurable frequency (e.g., 1–10 seconds). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/97c69cfa-ed31-402b-935d-aa24257e605f/1-which-specific-topics-or-ris-eHRAYKP7Qg.OEhXKF519cg.md)

### R.2 Gate semantics

Define three top‑level gate predicates:

1. **SafetyGate** – *operation allowed*  

   - `SafetyGate = corridor_ok ∧ legal_ok ∧ (V_{t+1} ≤ V_t + ε)`  

2. **ScaleUpGate** – *scale‑up / bonus allowed*  

   - `ScaleUpGate = SafetyGate ∧ gold_ok ∧ LCA_ok`  

   where `LCA_ok` comes from the LCA gate (E.4).

3. **DeploymentGate** – *initial deployment allowed*  

   - `DeploymentGate = LCA_ok ∧ PilotGates_ok`  

   where `PilotGates_ok` aggregates structural, treatment/social, fouling/maintenance, and social‑governance predicates proven over at least one seasonal cycle. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/97c69cfa-ed31-402b-935d-aa24257e605f/1-which-specific-topics-or-ris-eHRAYKP7Qg.OEhXKF519cg.md)

If any gate evaluates to `false`, the corresponding action (operate, scale up, deploy) must be blocked in software and surfaced as an explicit violation in logs and dashboards.

***

## qpudatashard binding

### Shard type 1: Corridor specification shard

Example schema row:

```csv
node_id,parameter,unit,legal_limit,gold_limit,r_min,r_max,weight_w,channel,ecoimpactscore,notes
PHX-CYBO-NOX,NOx_stack,mg/Nm3,150,40,0,150,0.22,0,0.92,"EU IED + WHO AQG anchored"
```

Semantics:

- `node_id`: unique furnace or stack segment identifier  
- `parameter`..`channel`: bound to K‑layer definitions  
- `ecoimpactscore`: priority 0–1, used for governance dashboards, not for control math  
- **Constraint**: missing `legal_limit` or `r_max` for any safety parameter causes CI failure. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_3d4de815-b635-4b21-b2a0-98997e218102/d1fdddb2-6e63-4831-97d0-8aa8254914c7/econet-alignment-response-ariz-CcGf9wVDSv2yZ5z34LGftA.md)

### Shard type 2: LCA scenario shard

Example rows:

```csv
scenario_id,region_id,functional_unit,mode,GWP_kgCO2eq,grid_gCO2_per_kWh,landfill_ref_GWP_kgCO2eq_per_ton,avoided_virgin_metal_kgCO2eq_per_kg,energy_recovery_efficiency,recycling_rate,notes
PHX-BASE-MSW,Phoenix,MSW_TON,STATUS_QUO,480,420,520,0,0.0,0.25,"Baseline landfill + current recycling"
PHX-CYBO-MSW,Phoenix,MSW_TON,CYBOCINDER,320,420,520,1500,0.25,0.25,"Cybocinder with energy recovery and metal recovery"
PHX-BASE-MWH,Phoenix,ENERGY_MWH,STATUS_QUO,650,420,0,0,0.0,0.25,"Gas-fired baseline"
PHX-CYBO-MWH,Phoenix,ENERGY_MWH,CYBOCINDER,420,420,0,0,0.25,0.25,"Cybocinder electricity/heat"
```

- CI rule: for each `region_id` and `functional_unit`, a STATUS_QUO and CYBOCINDER pair must exist and satisfy the LCA invariants. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/1dd8dd6b-dd04-4477-894b-aac4648763f3/e-co-lab-o-rated-is-the-study-uwDFimgZRLuBik9k7Qw.iA.md)

### Shard type 3: Telemetry / residual shard

Example row (high‑frequency logging):

```csv
timestamp,node_id,channel,param_name,value,unit,risk_r,weight_w,V_t,mode,legal_ok,gold_ok,gate_safety_ok
2026-01-17T19:27:00Z,PHX-CYBO-1,0,NOx_stack,95,mg/Nm3,0.63,0.22,0.41,OPERATE,true,true,true
```

- This shard provides a tamper‑evident history of risk coordinates, residual values, and gate status. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_3d4de815-b635-4b21-b2a0-98997e218102/d1fdddb2-6e63-4831-97d0-8aa8254914c7/econet-alignment-response-ariz-CcGf9wVDSv2yZ5z34LGftA.md)

***

## Knowledge / eco‑impact / risk scoring (K, E, R scores)

For each **input** (new design choice, policy change, waste mix) and **output** (spec, code, deployment):

- **Knowledge factor (K‑score)**  
  - Measures how fully the input/output is tied into K‑layer definitions and shards (0 = undocumented, 1 = fully typed, shard‑backed, and referenced).  

- **Eco‑impact value (E‑score)**  
  - Derived from LCA deltas, residual history, and ecoimpactscore fields (e.g., normalized net reduction in GWP and high‑priority pollutants).  

- **Risk factor (R‑score)**  
  - Combines residual statistics (mean/peak \(V_t\)), gate violation frequency, and unresolved corridor gaps (higher R‑score = higher residual risk).  

These three scores can be kept as separate structured fields in governance shards to prioritize work and halt changes that lower K or E or increase R beyond thresholds. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/97c69cfa-ed31-402b-935d-aa24257e605f/1-which-specific-topics-or-ris-eHRAYKP7Qg.OEhXKF519cg.md)

***

This grammar gives you:

- A **precise vocabulary** (K) for parameters, corridors, residuals, and LCA scenarios.  
- A **behavioral contract layer** (E) that defines exactly when moves, scale‑ups, and deployments are allowed.  
- A **risk/gate layer** (R) that aggregates residuals and decisions into simple booleans and scores for CI, dashboards, and governance.

From here you can:

- Lift these blocks directly into an RFC.  
- Auto‑generate C++ `struct`s and `static_assert`s from the shard schemas.  
- Auto‑generate ALN contracts from the K/E/R definitions and gate semantics.
