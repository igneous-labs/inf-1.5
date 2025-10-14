use inf1_test_utils::mollusk_inf_local_ctl;
use mollusk_svm::Mollusk;

thread_local! {
    pub static SVM: Mollusk = mollusk_inf_local_ctl()
}
