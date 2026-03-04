pub fn fmt_weight(kg: impl Into<f64>) -> String {
    let kg = kg.into();
    format!("{}", (kg * 100.0).round() / 100.0)
}
