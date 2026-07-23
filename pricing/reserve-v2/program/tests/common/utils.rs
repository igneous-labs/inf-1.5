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

        let b = e.nanos.base_fee_nanos();
        let t = e.nanos.threshold_nanos();
        let tf = e.nanos.threshold_fee_nanos();
        let mf = e.nanos.max_fee_nanos();
        let of = e.nanos.output_fee_nanos();

        assert!(b.get() <= FeeNanos::MAX.get(), "base_fee out of range");
        assert!(
            tf.get() <= FeeNanos::MAX.get(),
            "threshold_fee out of range"
        );
        assert!(mf.get() <= FeeNanos::MAX.get(), "max_fee out of range");
        assert!(of.get() <= FeeNanos::MAX.get(), "output_fee out of range");
        assert!(t.get() >= ThresholdNanos::MIN.get(), "threshold too low");
        assert!(t.get() <= ThresholdNanos::MAX.get(), "threshold too high");

        assert!(b <= tf, "base_fee > threshold_fee");
        assert!(tf <= mf, "threshold_fee > max_fee");
    });
}
