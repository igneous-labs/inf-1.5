use core::array;

use proptest::prelude::*;

/// Given a mint supply, generate N token balances
/// s.t. the sum of these N token balances dont exceed the supply.
///
/// Returns (token balances, unconsumed supply)
pub fn bals_from_supply<const N: usize>(supply: u64) -> impl Strategy<Value = ([u64; N], u64)> {
    let end = array::from_fn(|_| Just(0u64));
    (0..N).fold((end, Just(supply)).boxed(), |tup, i| {
        tup.prop_flat_map(|(end, rem)| (bal_from_supply(rem), Just(end)))
            .prop_map(move |((bal, rem), mut end)| {
                end[i] = bal;
                (end, rem)
            })
            .boxed()
    })
}

/// Given a mint supply, generate a token balance
/// that doesnt exceed the supply.
///
/// Returns (token balance, unconsumed supply)
pub fn bal_from_supply(supply: u64) -> impl Strategy<Value = (u64, u64)> {
    (0..=supply).prop_map(move |bal| (bal, supply - bal))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strat<const N: usize>() -> impl Strategy<Value = (u64, ([u64; N], u64))> {
        any::<u64>().prop_flat_map(|supply| (Just(supply), bals_from_supply(supply)))
    }

    proptest! {
        #[test]
        fn bals_from_supply_sum_invariant((supply, (bals, rem)) in strat::<5>()) {
            let sum = bals.into_iter().sum::<u64>() + rem;
            prop_assert_eq!(supply, sum);
        }
    }
}
