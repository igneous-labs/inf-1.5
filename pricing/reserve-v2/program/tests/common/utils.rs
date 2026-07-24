use inf1_pp_reserve_v2_core::{
    keys::CONST_KEYS_OWNED,
    typedefs::{FeeEntry, FeeNanos, ThresholdNanos},
};
use solana_pubkey::Pubkey;

pub fn assert_valid_fee_entries(entries: &[FeeEntry]) {
    assert!(entries.is_sorted_by_key(|e| e.mint));

    // assert no duplicate mints
    for w in entries.windows(2) {
        if w[0].mint == w[1].mint {
            panic!("duplicate mint {}", Pubkey::new_from_array(w[0].mint));
        }
    }

    [CONST_KEYS_OWNED.lp_mint(), CONST_KEYS_OWNED.wsol_mint()]
        .iter()
        .for_each(|always_present| assert!(entries.iter().any(|e| e.mint == **always_present)));

    entries.iter().for_each(|e| {
        // TODO: when the validation functions are done, refactor to use those instead

        let b = e.fee_nanos.base_fee();
        let t = e.threshold_nanos;
        let tf = e.fee_nanos.threshold_fee();
        let mf = e.fee_nanos.max_fee();
        let of = e.fee_nanos.output_fee();

        assert!(*b <= FeeNanos::MAX.get(), "base_fee out of range");
        assert!(*tf <= FeeNanos::MAX.get(), "threshold_fee out of range");
        assert!(*mf <= FeeNanos::MAX.get(), "max_fee out of range");
        assert!(*of <= FeeNanos::MAX.get(), "output_fee out of range");
        assert!(t >= ThresholdNanos::MIN.get(), "threshold too low");
        assert!(t <= ThresholdNanos::MAX.get(), "threshold too high");

        assert!(b <= tf, "base_fee > threshold_fee");
        assert!(tf <= mf, "threshold_fee > max_fee");
    });
}
