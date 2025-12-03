use inf1_test_utils::{fill_mock_prog_accs, AccountMap};

use crate::tests::swap::Accs;

pub fn add_swap_prog_accs<P>(
    am: &mut AccountMap,
    Accs {
        inp_calc_prog,
        out_calc_prog,
        pricing_prog,
        ..
    }: &Accs<P>,
) {
    fill_mock_prog_accs(am, [*inp_calc_prog, *out_calc_prog, *pricing_prog]);
}
