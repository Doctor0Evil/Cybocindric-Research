use crate::types::LcaScenario;

pub fn lca_ok(
    base: &LcaScenario,
    cybo: &LcaScenario,
) -> bool {
    assert_eq!(base.region_id, cybo.region_id);
    assert_eq!(base.functional_unit, cybo.functional_unit);
    assert_eq!(base.mode, "STATUS_QUO");
    assert_eq!(cybo.mode, "CYBOCINDER");
    cybo.gwp_kg_co2eq < base.gwp_kg_co2eq
}
