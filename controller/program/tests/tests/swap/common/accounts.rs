use inf1_core::instructions::swap::IxAccs;
use inf1_test_utils::{fill_mock_prog_accs, AccountMap};

pub fn fill_swap_prog_accs<I, C, D, P>(
    am: &mut AccountMap,
    IxAccs {
        inp_calc_prog,
        out_calc_prog,
        pricing_prog,
        ..
    }: &IxAccs<[u8; 32], I, C, D, P>,
) {
    fill_mock_prog_accs(am, [*inp_calc_prog, *out_calc_prog, *pricing_prog]);
}
