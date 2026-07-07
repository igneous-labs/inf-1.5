//! Account types of other programs

use inf1_svc_core::buf_utils::csba;

pub const BPF_LOADER_V3_PROGRAMDATA_DISCM_SIZE: usize = 4;

/// AKA `UpgradeableLoaderState::ProgramData`. bincode discm.
pub const BPF_LOADER_V3_PROGRAMDATA_DISCM: [u8; BPF_LOADER_V3_PROGRAMDATA_DISCM_SIZE] =
    [3, 0, 0, 0];

pub const BPF_LOADER_V3_PROGRAMDATA_META_SIZE: usize = 41;

pub const BPF_LOADER_V3_PROGRAMDATA_HEADER_SIZE: usize =
    BPF_LOADER_V3_PROGRAMDATA_META_SIZE + BPF_LOADER_V3_PROGRAMDATA_DISCM_SIZE;

/// # Returns
///
/// `(last_upgrade_slot, upgrade_auth)` if successful
///
/// `None` if invalid programdata data
pub const fn parse_bpf_loader_v3_programdata_meta(
    data: &[u8; BPF_LOADER_V3_PROGRAMDATA_HEADER_SIZE],
) -> Option<(u64, Option<&[u8; 32]>)> {
    let (discm, meta) = csba::<
        BPF_LOADER_V3_PROGRAMDATA_HEADER_SIZE,
        BPF_LOADER_V3_PROGRAMDATA_DISCM_SIZE,
        BPF_LOADER_V3_PROGRAMDATA_META_SIZE,
    >(data);

    if u32::from_le_bytes(*discm) != u32::from_le_bytes(BPF_LOADER_V3_PROGRAMDATA_DISCM) {
        None
    } else {
        parse_no_discm_bpf_loader_v3_programdata_meta(meta)
    }
}

/// # Returns
///
/// `(last_upgrade_slot, upgrade_auth)` if successful
///
/// `None` if invalid programdata data
pub const fn parse_no_discm_bpf_loader_v3_programdata_meta(
    data: &[u8; BPF_LOADER_V3_PROGRAMDATA_META_SIZE],
) -> Option<(u64, Option<&[u8; 32]>)> {
    let (lus, rest) = csba::<BPF_LOADER_V3_PROGRAMDATA_META_SIZE, 8, 33>(data);
    let (&[discm], upgrade_auth) = csba::<33, 1, 32>(rest);

    let lus = u64::from_le_bytes(*lus);
    match discm {
        0 => Some((lus, None)),
        1 => Some((lus, Some(upgrade_auth))),
        _ => None,
    }
}
