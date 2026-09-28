mod pool;
mod portfolio;
mod protocol;
mod rebal_aux;

pub use pool::*;
pub use portfolio::*;
pub use protocol::*;
pub use rebal_aux::*;

// ideally this is in each indiv file,
// but we made accounts mod a barrel
#[cfg(kani)]
pub mod spec {
    use super::*;

    // pool

    /// A solvent pool has
    /// - pool_acc_lamports >= rent_exempt
    /// - pool_acc_lamports + outstanding >= rent_exempt + protocol_fee
    ///   (dep_due_checked should not panic)
    ///
    /// Preconditions:
    /// - `p.dep_due` must not overflow
    pub fn is_pool_solvent((p, pool_acc_lamports): (PoolV1LamportVals, u64)) -> bool {
        pool_acc_lamports >= *p.rent_exempt()
            && pool_acc_lamports + p.outstanding() >= p.rent_exempt() + p.protocol_fee()
    }

    /// a pool whose values will not result in overflow for the additions in [`is_pool_solvent`]
    fn pool() -> (PoolV1LamportVals, u64) {
        let [rent_exempt, outstanding, protocol_fee, pool_acc_lamports] =
            core::array::from_fn(|_| kani::any());

        // overflow guards
        kani::assume(u64::MAX - pool_acc_lamports >= outstanding);
        kani::assume(u64::MAX - rent_exempt >= protocol_fee);

        (
            PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
                rent_exempt,
                outstanding,
                protocol_fee,
            }),
            pool_acc_lamports,
        )
    }

    #[kani::proof]
    fn is_pool_solvent_iff_dep_due_checked_some() {
        // is_pool_solvent ->
        let x = pool();
        kani::assume(is_pool_solvent(x));
        assert!(x.0.dep_due_checked(x.1).is_some());

        // dep_due_checked_some ->
        let y = pool();
        kani::assume(y.0.dep_due_checked(y.1).is_some());
        assert!(is_pool_solvent(y));
    }

    /// Returns values that satisfy [`is_pool_solvent`]
    ///
    /// Returns `(PoolV1Lamports, pool_acc.lamports)`
    pub fn solvent_pool() -> (PoolV1LamportVals, u64) {
        let res = pool();
        kani::assume(is_pool_solvent(res));
        res
    }

    /// dep_due = liq_avail + outstanding - protocol_fee
    #[kani::proof]
    fn dep_due_formula_correct() {
        let (p, pool_acc_lamports) = solvent_pool();
        let dep_due = p.dep_due_checked(pool_acc_lamports).unwrap();
        let liq_avail = p.liq_avail_checked(pool_acc_lamports).unwrap();
        assert_eq!(dep_due, liq_avail + p.outstanding() - p.protocol_fee());
    }

    /// depositing N lamports increases dep_due by exactly N
    #[kani::proof]
    fn dep_due_increases_with_deposits() {
        let (p, pool_acc_lamports) = solvent_pool();
        let deposit: u64 = kani::any();

        let new_pool_acc_lamports = match pool_acc_lamports.checked_add(deposit) {
            Some(x) => x,
            None => return,
        };
        // overflow guard for is_pool_solvent's unchecked addition
        kani::assume(new_pool_acc_lamports <= u64::MAX - *p.outstanding());
        if !is_pool_solvent((p, new_pool_acc_lamports)) {
            return;
        }

        let dep_due_before = match p.dep_due_checked(pool_acc_lamports) {
            Some(x) => x,
            None => return,
        };
        let dep_due_after = match p.dep_due_checked(new_pool_acc_lamports) {
            Some(x) => x,
            None => return,
        };
        let expected = match dep_due_before.checked_add(deposit) {
            Some(x) => x,
            None => return,
        };

        assert_eq!(dep_due_after, expected);
    }

    /// increasing outstanding by N increases dep_due by exactly N
    #[kani::proof]
    fn dep_due_increases_with_outstanding() {
        let (p, pool_acc_lamports) = solvent_pool();
        let outstanding_inc: u64 = kani::any();

        kani::assume(outstanding_inc <= u64::MAX - *p.outstanding());
        let new_outstanding = p.outstanding() + outstanding_inc;
        // overflow guard for is_pool_solvent's unchecked addition
        kani::assume(new_outstanding <= u64::MAX - pool_acc_lamports);

        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: *p.rent_exempt(),
            outstanding: new_outstanding,
            protocol_fee: *p.protocol_fee(),
        });
        kani::assume(is_pool_solvent((new_p, pool_acc_lamports)));

        let dep_due_before = p.dep_due_checked(pool_acc_lamports).unwrap();
        let dep_due_after = new_p.dep_due_checked(pool_acc_lamports).unwrap();
        assert_eq!(dep_due_after - dep_due_before, outstanding_inc);
    }

    /// increasing protocol_fee by N decreases dep_due by exactly N
    #[kani::proof]
    fn dep_due_decreases_with_protocol_fee() {
        let (p, pool_acc_lamports) = solvent_pool();
        let fee_inc: u64 = kani::any();

        kani::assume(fee_inc <= u64::MAX - *p.protocol_fee());
        let new_protocol_fee = p.protocol_fee() + fee_inc;
        // overflow guard for is_pool_solvent's unchecked addition
        kani::assume(new_protocol_fee <= u64::MAX - *p.rent_exempt());

        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: *p.rent_exempt(),
            outstanding: *p.outstanding(),
            protocol_fee: new_protocol_fee,
        });
        kani::assume(is_pool_solvent((new_p, pool_acc_lamports)));

        let dep_due_before = p.dep_due_checked(pool_acc_lamports).unwrap();
        let dep_due_after = new_p.dep_due_checked(pool_acc_lamports).unwrap();
        assert_eq!(dep_due_before - dep_due_after, fee_inc);
    }

    /// claiming <= surplus preserves dep_due >= mint_supply
    #[kani::proof]
    fn claim_preserves_solvency() {
        let (p, pool_acc_lamports) = solvent_pool();
        let mint_supply: u64 = kani::any();

        let dep_due = p.dep_due_checked(pool_acc_lamports).unwrap();
        kani::assume(mint_supply <= dep_due);

        let surplus = dep_due - mint_supply;
        let claim_amt: u64 = kani::any();
        kani::assume(claim_amt <= surplus);

        let liq_avail = p.liq_avail_checked(pool_acc_lamports).unwrap();
        kani::assume(claim_amt <= liq_avail);

        let new_pool_acc_lamports = pool_acc_lamports - claim_amt;
        kani::assume(is_pool_solvent((p, new_pool_acc_lamports)));

        let new_dep_due = p.dep_due_checked(new_pool_acc_lamports).unwrap();
        assert_eq!(new_dep_due, dep_due - claim_amt);
        assert!(new_dep_due >= mint_supply);
    }

    /// rebalancing SOL out (lamports ↓, outstanding ↑) preserves dep_due
    #[kani::proof]
    fn rebal_sol_out_preserves_dep_due() {
        let (p, pool_acc_lamports) = solvent_pool();
        let rebal_amt: u64 = kani::any();

        let liq_avail = p.liq_avail_checked(pool_acc_lamports).unwrap();
        kani::assume(rebal_amt <= liq_avail);
        kani::assume(rebal_amt <= u64::MAX - *p.outstanding());

        let new_pool_acc_lamports = pool_acc_lamports - rebal_amt;
        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: *p.rent_exempt(),
            outstanding: p.outstanding() + rebal_amt,
            protocol_fee: *p.protocol_fee(),
        });
        kani::assume(is_pool_solvent((new_p, new_pool_acc_lamports)));

        let dep_due_before = p.dep_due_checked(pool_acc_lamports).unwrap();
        let dep_due_after = new_p.dep_due_checked(new_pool_acc_lamports).unwrap();
        assert_eq!(dep_due_after, dep_due_before);
    }

    /// rebalancing SOL in (lamports ↑, outstanding ↓) preserves dep_due
    #[kani::proof]
    fn rebal_sol_inp_preserves_dep_due() {
        let (p, pool_acc_lamports) = solvent_pool();
        let rebal_amt: u64 = kani::any();

        kani::assume(rebal_amt <= *p.outstanding());
        kani::assume(rebal_amt <= u64::MAX - pool_acc_lamports);

        let new_pool_acc_lamports = pool_acc_lamports + rebal_amt;
        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: *p.rent_exempt(),
            outstanding: p.outstanding() - rebal_amt,
            protocol_fee: *p.protocol_fee(),
        });
        kani::assume(is_pool_solvent((new_p, new_pool_acc_lamports)));

        let dep_due_before = p.dep_due_checked(pool_acc_lamports).unwrap();
        let dep_due_after = new_p.dep_due_checked(new_pool_acc_lamports).unwrap();
        assert_eq!(dep_due_after, dep_due_before);
    }

    /// sync_rent_pool: increasing rent_exempt by shortfall decreases dep_due by shortfall.
    /// If surplus >= shortfall, pool remains depositor-solvent after the change.
    #[kani::proof]
    fn sync_rent_pool_preserves_solvency() {
        let (p, pool_acc_lamports) = solvent_pool();
        let mint_supply: u64 = kani::any();

        let dep_due = p.dep_due_checked(pool_acc_lamports).unwrap();
        kani::assume(mint_supply <= dep_due);

        let surplus = dep_due - mint_supply;
        let new_rent_exempt: u64 = kani::any();

        kani::assume(new_rent_exempt >= *p.rent_exempt());
        kani::assume(pool_acc_lamports >= new_rent_exempt);

        let shortfall = new_rent_exempt - *p.rent_exempt();

        kani::assume(surplus >= shortfall);

        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: new_rent_exempt,
            outstanding: *p.outstanding(),
            protocol_fee: *p.protocol_fee(),
        });

        let new_dep_due = match new_p.dep_due_checked(pool_acc_lamports) {
            Some(x) => x,
            None => return,
        };

        assert_eq!(
            new_dep_due,
            dep_due - shortfall,
            "dep_due must decrease by shortfall"
        );

        assert!(
            new_dep_due >= mint_supply,
            "pool must remain depositor-solvent"
        );
    }

    /// sync_rent_pool: decreasing rent_exempt (rent decrease) increases surplus.
    /// Pool remains solvent and surplus strictly increases.
    #[kani::proof]
    fn sync_rent_pool_decrease_increases_surplus() {
        let (p, pool_acc_lamports) = solvent_pool();
        let mint_supply: u64 = kani::any();

        let dep_due = p.dep_due_checked(pool_acc_lamports).unwrap();
        kani::assume(mint_supply <= dep_due);

        let new_rent_exempt: u64 = kani::any();

        kani::assume(new_rent_exempt > 0);
        kani::assume(new_rent_exempt < *p.rent_exempt());

        let freed = *p.rent_exempt() - new_rent_exempt;

        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: new_rent_exempt,
            outstanding: *p.outstanding(),
            protocol_fee: *p.protocol_fee(),
        });

        let new_dep_due = match new_p.dep_due_checked(pool_acc_lamports) {
            Some(x) => x,
            None => return,
        };

        assert_eq!(
            new_dep_due,
            dep_due + freed,
            "dep_due must increase by freed amount"
        );

        assert!(
            new_dep_due >= mint_supply,
            "pool must remain depositor-solvent"
        );
        assert!(
            new_dep_due - mint_supply >= dep_due - mint_supply + freed,
            "surplus must increase by freed amount"
        );
    }

    /// swap_other out pool accounting: if outstanding and mint_supply both increase
    /// by the same delta, surplus (dep_due - mint_supply) is exactly preserved.
    #[kani::proof]
    fn swap_other_out_pool_surplus_preserved() {
        let (p, pool_acc_lamports) = solvent_pool();
        let mint_supply: u64 = kani::any();

        let dep_due = p.dep_due_checked(pool_acc_lamports).unwrap();
        kani::assume(mint_supply <= dep_due);

        let delta: u64 = kani::any();

        kani::assume(delta <= u64::MAX - *p.outstanding());
        let new_outstanding = p.outstanding() + delta;
        // overflow guard for is_pool_solvent's unchecked addition
        kani::assume(new_outstanding <= u64::MAX - pool_acc_lamports);
        kani::assume(delta <= u64::MAX - mint_supply);

        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: *p.rent_exempt(),
            outstanding: new_outstanding,
            protocol_fee: *p.protocol_fee(),
        });
        kani::assume(is_pool_solvent((new_p, pool_acc_lamports)));

        let new_dep_due = new_p.dep_due_checked(pool_acc_lamports).unwrap();
        let new_mint_supply = mint_supply + delta;

        assert_eq!(
            new_dep_due - dep_due,
            delta,
            "dep_due must increase by delta"
        );
        assert_eq!(
            new_dep_due - new_mint_supply,
            dep_due - mint_supply,
            "surplus must be preserved"
        );
        assert!(
            new_dep_due >= new_mint_supply,
            "pool must remain depositor-solvent"
        );
    }

    /// swap_other inp pool accounting: if outstanding decreases by outstanding_dec
    /// and protocol_fee increases by pf_total, and mint_supply decreases by claim_lamports,
    /// where outstanding_dec = claim_lamports - pf_total, then surplus is preserved.
    #[kani::proof]
    fn swap_other_inp_pool_surplus_preserved() {
        let (p, pool_acc_lamports) = solvent_pool();
        let mint_supply: u64 = kani::any();

        let dep_due = p.dep_due_checked(pool_acc_lamports).unwrap();
        kani::assume(mint_supply <= dep_due);

        let claim_lamports: u64 = kani::any();
        let pf_total: u64 = kani::any();

        kani::assume(claim_lamports <= mint_supply);
        kani::assume(pf_total <= claim_lamports);
        let outstanding_dec = claim_lamports - pf_total;

        kani::assume(outstanding_dec <= *p.outstanding());
        kani::assume(pf_total <= u64::MAX - *p.protocol_fee());
        let new_protocol_fee = p.protocol_fee() + pf_total;
        // overflow guard for is_pool_solvent's unchecked addition
        kani::assume(new_protocol_fee <= u64::MAX - *p.rent_exempt());

        let new_p = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: *p.rent_exempt(),
            outstanding: p.outstanding() - outstanding_dec,
            protocol_fee: new_protocol_fee,
        });
        kani::assume(is_pool_solvent((new_p, pool_acc_lamports)));

        let new_dep_due = new_p.dep_due_checked(pool_acc_lamports).unwrap();
        let new_mint_supply = mint_supply - claim_lamports;

        assert_eq!(
            dep_due - new_dep_due,
            claim_lamports,
            "dep_due must decrease by claim_lamports"
        );

        assert_eq!(
            new_dep_due - new_mint_supply,
            dep_due - mint_supply,
            "surplus must be preserved"
        );
    }
}
