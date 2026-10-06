use super::ComputeBudget;

#[test]
fn reserves_capacity_on_larger_machines() {
    let plan = ComputeBudget::from_logical(12);
    assert_eq!(plan.model_threads, 4);
}

#[test]
fn keeps_small_machines_within_budget() {
    let plan = ComputeBudget::from_logical(4);
    assert_eq!(plan.model_threads, 2);
}
