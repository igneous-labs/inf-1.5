use generic_array_struct::generic_array_struct;

macro_rules! const_map {
    ($DEFAULT:expr, $from_arr:expr, $const_fn:expr) => {{
        let mut res = [$DEFAULT; _];
        let mut i = 0;
        while i < res.len() {
            res[i] = $const_fn(&$from_arr[i]);
            i += 1;
        }
        res
    }};
}
pub(crate) use const_map;

/// Program-specific const addrs
#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstAccs<T> {
    /// This program's (generic SOL Value Calculator) program ID
    pub program: T,

    /// The stake pool program that this SVC program works with
    pub pool_prog: T,

    /// initial manager to set to
    pub init_manager: T,
}

impl ConstAccs<&str> {
    pub const fn const_keys(self) -> ConstAccs<[u8; 32]> {
        ConstAccs(const_map!(
            [0; 32],
            self.0,
            const_crypto::bs58::decode_pubkey
        ))
    }
}

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalConstAccs<T> {
    pub sys_prog: T,

    pub bpf_loader_v3: T,
}

pub const GLOBAL_CONST_KEY_STRS: GlobalConstAccs<&str> =
    GlobalConstAccs::const_from_destr(GlobalConstAccsDestr {
        sys_prog: "11111111111111111111111111111111",
        bpf_loader_v3: "BPFLoaderUpgradeab1e11111111111111111111111",
    });

pub const GLOBAL_CONST_KEYS_OWNED: GlobalConstAccs<[u8; 32]> = GlobalConstAccs(const_map!(
    [0; 32],
    GLOBAL_CONST_KEY_STRS.0,
    const_crypto::bs58::decode_pubkey
));
