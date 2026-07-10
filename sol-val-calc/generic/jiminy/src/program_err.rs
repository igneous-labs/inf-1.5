use inf1_svc_generic::errs::GenSvcErr;
use jiminy_account::program_error::ProgramError;
use jiminy_log::sol_log;

pub struct GenSvcProgErr(pub GenSvcErr);

impl From<GenSvcErr> for GenSvcProgErr {
    #[inline]
    fn from(e: GenSvcErr) -> Self {
        Self(e)
    }
}

impl From<GenSvcProgErr> for ProgramError {
    // Note: to_string() + log adds around 15kb to binsize
    /// Also `sol_msg` logs the error string.
    #[inline]
    fn from(GenSvcProgErr(e): GenSvcProgErr) -> Self {
        let msg = e.to_string();
        sol_log(&msg);
        ProgramError::custom(e.into())
    }
}
