use inf1_std::inf1_pp_ag_std::inf1_pp_flatslab_std;
use mollusk_svm::Mollusk;

use crate::common::{test_fixtures_dir, workspace_root_dir, BPF_LOADER_UPGRADEABLE_ADDR};

pub const FIXTURE_PROGRAMS: [(&str, [u8; 32]); 6] = [
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

pub const LOCAL_PROGRAMS: [(&str, [u8; 32]); 1] =
    [("inf1_pp_flatslab_program", inf1_pp_flatslab_std::ID)];

pub fn mollusk_inf() -> Mollusk {
    let mut res = Mollusk::default();
    let paths = FIXTURE_PROGRAMS
        .into_iter()
        .map(|(fname, key)| {
            (
                test_fixtures_dir()
                    .join("programs")
                    .join(fname)
                    .with_extension("so"),
                key,
            )
        })
        .chain(LOCAL_PROGRAMS.into_iter().map(|(fname, key)| {
            (
                workspace_root_dir()
                    .join("target/deploy")
                    .join(fname)
                    .with_extension("so"),
                key,
            )
        }));
    paths.for_each(|(path, key)| {
        res.add_program_with_elf_and_loader(
            &key.into(),
            &std::fs::read(path).unwrap(),
            &BPF_LOADER_UPGRADEABLE_ADDR,
        );
    });
    mollusk_svm_programs_token::token::add_program(&mut res);
    res
}
