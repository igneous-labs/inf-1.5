use inf1_std::instructions::swap::IxAccs;

use crate::tests::swap::{V1Args, V2Args};

mod exact_in;
// mod exact_out;
// mod test_utils;

fn args_to_v2<P>(
    V1Args {
        inp_lst_index,
        out_lst_index,
        limit,
        amount,
        accs:
            IxAccs {
                ix_prefix,
                inp_calc_prog,
                inp_calc,
                out_calc_prog,
                out_calc,
                pricing_prog,
                pricing,
            },
    }: V1Args<P>,
) -> V2Args<P> {
    V2Args {
        inp_lst_index,
        out_lst_index,
        limit,
        amount,
        accs: IxAccs {
            ix_prefix: ix_prefix.into(),
            inp_calc_prog,
            inp_calc,
            out_calc_prog,
            out_calc,
            pricing_prog,
            pricing,
        },
    }
}
