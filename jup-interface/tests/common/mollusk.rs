use mollusk_svm::Mollusk;
use solana_pubkey::Pubkey;

use crate::common::test_fixtures_dir;

pub const INF_PROGRAMS: [(&str, [u8; 32]); 6] = [
    (
        "flat-fee-pp",
        inf1_std::inf1_pp_ag_std::inf1_pp_flatfee_core::ID,
    ),
    ("inf", inf1_std::inf1_ctl_core::ID),
    (
        "lido-calc",
        inf1_std::inf1_svc_ag_std::inf1_svc_lido_core::ID,
    ),
    (
        "marinade-calc",
        inf1_std::inf1_svc_ag_std::inf1_svc_marinade_core::ID,
    ),
    (
        "sanctum-spl-multi-calc",
        inf1_std::inf1_svc_ag_std::inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
    ),
    (
        "wsol-calc",
        inf1_std::inf1_svc_ag_std::inf1_svc_wsol_core::ID,
    ),
];

pub fn mollusk_inf() -> Mollusk {
    let mut res = Mollusk::default();
    INF_PROGRAMS.into_iter().for_each(|(fname, key)| {
        let path = test_fixtures_dir()
            .join("programs")
            .join(fname)
            .with_extension("so");
        res.add_program_with_elf_and_loader(
            &key.into(),
            &std::fs::read(path).unwrap(),
            &Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111"),
        );
    });
    mollusk_svm_programs_token::token::add_program(&mut res);
    res
}
