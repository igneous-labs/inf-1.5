use crate::{
    errs::{ReserveV2ProgramErr, SameMintErr},
    keys::CONST_KEYS_OWNED,
    pricing::ReserveV2Swap,
};

pub type ReserveV2SwapKind = ReserveV2Swap<(), ()>;

#[inline]
pub const fn classify_route(
    input_mint: &[u8; 32],
    output_mint: &[u8; 32],
) -> Result<ReserveV2SwapKind, ReserveV2ProgramErr> {
    if bytes_eq(input_mint, output_mint) {
        return Err(ReserveV2ProgramErr::SameMint(SameMintErr {
            mint: *input_mint,
        }));
    }

    if !bytes_eq(input_mint, CONST_KEYS_OWNED.wsol_mint())
        && (bytes_eq(output_mint, CONST_KEYS_OWNED.wsol_mint())
            || bytes_eq(output_mint, CONST_KEYS_OWNED.lp_mint()))
    {
        return Ok(ReserveV2SwapKind::RangeOut(()));
    }

    Ok(ReserveV2SwapKind::Flat(()))
}

const fn bytes_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut i = 0;
    while i < 32 {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::keys::CONST_KEYS_OWNED;

    use super::*;

    const LP_MINT: [u8; 32] = *CONST_KEYS_OWNED.lp_mint();
    const WSOL_MINT: [u8; 32] = *CONST_KEYS_OWNED.wsol_mint();
    const LST_A: [u8; 32] = [2; 32];
    const LST_B: [u8; 32] = [3; 32];

    #[test]
    fn route_policy_matrix() {
        assert_eq!(
            classify_route(&LST_A, &WSOL_MINT),
            Ok(ReserveV2SwapKind::RangeOut(()))
        );
        assert_eq!(
            classify_route(&LST_A, &LP_MINT),
            Ok(ReserveV2SwapKind::RangeOut(()))
        );
        assert_eq!(
            classify_route(&LP_MINT, &WSOL_MINT),
            Ok(ReserveV2SwapKind::RangeOut(()))
        );
        assert_eq!(
            classify_route(&WSOL_MINT, &LP_MINT),
            Ok(ReserveV2SwapKind::Flat(()))
        );
        assert_eq!(
            classify_route(&LST_A, &LST_B),
            Ok(ReserveV2SwapKind::Flat(()))
        );
        assert_eq!(
            classify_route(&WSOL_MINT, &LST_A),
            Ok(ReserveV2SwapKind::Flat(()))
        );
        assert_eq!(
            classify_route(&LP_MINT, &LST_A),
            Ok(ReserveV2SwapKind::Flat(()))
        );
    }

    #[test]
    fn input_mint_eq_output_mint_rejected() {
        assert_eq!(
            classify_route(&LST_A, &LST_A),
            Err(ReserveV2ProgramErr::SameMint(SameMintErr { mint: LST_A }))
        );
        assert_eq!(
            classify_route(&WSOL_MINT, &WSOL_MINT),
            Err(ReserveV2ProgramErr::SameMint(SameMintErr {
                mint: WSOL_MINT
            }))
        );
        assert_eq!(
            classify_route(&LP_MINT, &LP_MINT),
            Err(ReserveV2ProgramErr::SameMint(SameMintErr { mint: LP_MINT }))
        );
    }
}
