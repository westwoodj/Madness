#[cfg(test)]
mod tests {
    use crate::model::predictor::ModelWeights;
    use crate::fetch::odds::ml_to_implied_prob;

    #[test]
    fn test_sigmoid_midpoint() {
        // score = 0 → 50/50
        let prob = 1.0 / (1.0 + (-0.0_f64).exp());
        assert!((prob - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_sigmoid_strong_favorite() {
        // large positive score → near 1.0
        let prob = 1.0 / (1.0 + (-10.0_f64).exp());
        assert!(prob > 0.99);
    }

    #[test]
    fn test_ml_implied_prob_positive() {
        // +150 → 40%
        let p = ml_to_implied_prob(150);
        assert!((p - 0.40).abs() < 0.01, "expected ~0.40, got {}", p);
    }

    #[test]
    fn test_ml_implied_prob_negative() {
        // -200 → 66.7%
        let p = ml_to_implied_prob(-200);
        assert!((p - 0.6667).abs() < 0.01, "expected ~0.667, got {}", p);
    }

    #[test]
    fn test_ml_implied_prob_even() {
        // -110 (standard -vig line) → ~52.4%
        let p = ml_to_implied_prob(-110);
        assert!(p > 0.50 && p < 0.55);
    }

    #[test]
    fn test_default_weights_load() {
        let w = ModelWeights::default();
        assert!(w.net_eff_diff > 0.0);
        assert!(w.def_eff_diff > 0.0);
        assert!(w.seed_diff > 0.0);
        assert!(w.sportsbook_implied > 0.0);
    }

    #[test]
    fn test_seed_diff_normalization() {
        // 1-seed vs 16-seed → diff = (16 - 1) / 15.0 = 1.0
        let seed_diff = (16 - 1) as f64 / 15.0;
        assert!((seed_diff - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_confidence_from_prob() {
        let prob: f64 = 0.75;
        let confidence = (prob - 0.5).abs() * 2.0;
        assert!((confidence - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_confidence_at_50_50() {
        let prob: f64 = 0.5;
        let confidence = (prob - 0.5).abs() * 2.0;
        assert!(confidence < 1e-10); // zero confidence at 50/50
    }
}
