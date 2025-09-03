use inf1_test_utils::mollusk_inf;
use mollusk_svm::Mollusk;

thread_local! {
    pub static SVM: Mollusk = mollusk_inf()
}
