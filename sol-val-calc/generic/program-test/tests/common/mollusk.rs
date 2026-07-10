use inf1_svc_generic_program_test::CONST_KEYS_OWNED;
use inf1_test_utils::{mollusk_add_so_files, workspace_root_dir};
use mollusk_svm::Mollusk;

fn mollusk() -> Mollusk {
    let mut svm = Mollusk::default();
    mollusk_add_so_files(
        &mut svm,
        [(
            workspace_root_dir()
                .join("target/deploy/inf1_svc_generic_program_test")
                .with_extension("so"),
            *CONST_KEYS_OWNED.program(),
        )],
    );
    svm
}

thread_local! {
    pub static SVM: Mollusk = mollusk();
}
