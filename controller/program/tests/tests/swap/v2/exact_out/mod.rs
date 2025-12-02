use inf1_ctl_jiminy::{instructions::swap::v2::exact_out::SwapExactOutIxData, ID};
use inf1_pp_ag_core::instructions::PriceExactOutAccsAg;
use inf1_std::instructions::swap::v2::exact_out::{
    swap_exact_out_v2_ix_is_signer, swap_exact_out_v2_ix_is_writer, swap_exact_out_v2_ix_keys_owned,
};
use inf1_test_utils::{keys_signer_writable_to_metas, mock_prog_acc, AccountMap, ProgramDataAddr};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

mod add_liq;
mod rem_liq;
mod swap;

type Accs = super::super::Accs<PriceExactOutAccsAg>;
type Args = super::super::Args<PriceExactOutAccsAg>;

fn to_ix(args: &Args) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_out_v2_ix_keys_owned(&args.accs).seq(),
        swap_exact_out_v2_ix_is_signer(&args.accs).seq(),
        swap_exact_out_v2_ix_is_writer(&args.accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SwapExactOutIxData::new(&args.to_full()).as_buf().into(),
    }
}

fn add_prog_accs(
    am: &mut AccountMap,
    Accs {
        inp_calc_prog,
        out_calc_prog,
        pricing_prog,
        ..
    }: &Accs,
) {
    am.extend(
        [inp_calc_prog, out_calc_prog, pricing_prog]
            .into_iter()
            .map(|addr| {
                (
                    Pubkey::new_from_array(*addr),
                    // dont-care
                    mock_prog_acc(ProgramDataAddr::Raw(Default::default())),
                )
            }),
    );
}
