use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;
use sanctum_marinade_liquid_staking_core::{FeeCents, StakeSystem, State, ValidatorSystem};
use sanctum_token_ratio_compat::{
    fee_floor_ratio_u32_u32_reverse_from_rem, floor_ratio_u64_u64_reverse,
};
use sanctum_u64_ratio::{Floor, Ratio};

/// Parameters from MarinadeState required to calculate SOL value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarinadeCalc {
    pub available_reserve_balance: u64,
    pub circulating_ticket_balance: u64,
    pub delayed_unstake_cooling_down: u64,
    pub emergency_cooling_down: u64,
    pub msol_supply: u64,
    pub total_active_balance: u64,
    pub withdraw_stake_account_fee_cents: u32,
    pub withdraw_stake_account_enabled: bool,
    pub paused: bool,
}

/// Constructors
impl MarinadeCalc {
    #[inline]
    pub const fn new(
        State {
            available_reserve_balance,
            circulating_ticket_balance,
            emergency_cooling_down,
            msol_supply,
            withdraw_stake_account_enabled,
            paused,
            stake_system:
                StakeSystem {
                    delayed_unstake_cooling_down,
                    ..
                },
            validator_system:
                ValidatorSystem {
                    total_active_balance,
                    ..
                },
            withdraw_stake_account_fee:
                FeeCents {
                    bp_cents: withdraw_stake_account_fee_cents,
                },
            ..
        }: &State,
    ) -> Self {
        Self {
            available_reserve_balance: *available_reserve_balance,
            circulating_ticket_balance: *circulating_ticket_balance,
            delayed_unstake_cooling_down: *delayed_unstake_cooling_down,
            emergency_cooling_down: *emergency_cooling_down,
            msol_supply: *msol_supply,
            total_active_balance: *total_active_balance,
            withdraw_stake_account_fee_cents: *withdraw_stake_account_fee_cents,
            withdraw_stake_account_enabled: *withdraw_stake_account_enabled,
            paused: *paused,
        }
    }
}

/// Accessors
impl MarinadeCalc {
    #[inline]
    pub const fn total_cooling_down(&self) -> u64 {
        // overflow-safety: qty should never exceed sol supply < u64::MAX
        self.delayed_unstake_cooling_down + self.emergency_cooling_down
    }

    #[inline]
    pub const fn total_lamports_under_control(&self) -> u64 {
        // overflow-safety: qty should never exceed sol supply < u64::MAX
        let tcd = self.total_cooling_down();
        self.total_active_balance + tcd + self.available_reserve_balance
    }

    #[inline]
    pub const fn total_virtual_staked_lamports(&self) -> u64 {
        self.total_lamports_under_control()
            .saturating_sub(self.circulating_ticket_balance)
    }

    #[inline]
    pub const fn lamports_over_supply(&self) -> Floor<Ratio<u64, u64>> {
        Floor(Ratio {
            n: self.total_virtual_staked_lamports(),
            d: self.msol_supply,
        })
    }

    #[inline]
    pub const fn withdraw_stake_account_fee(&self) -> FeeCents {
        FeeCents {
            bp_cents: self.withdraw_stake_account_fee_cents,
        }
    }

    #[inline]
    pub const fn can_withdraw_stake(&self) -> Result<(), MarinadeCalcErr> {
        if self.paused {
            Err(MarinadeCalcErr::Paused)
        } else if !self.withdraw_stake_account_enabled {
            Err(MarinadeCalcErr::StakeWithdrawDisabled)
        } else {
            Ok(())
        }
    }
}

/// SolValCalc
impl MarinadeCalc {
    #[inline]
    pub const fn svc_lst_to_sol(
        &self,
        msol_amount: u64,
    ) -> Result<RangeInclusive<u64>, MarinadeCalcErr> {
        if let Err(e) = self.can_withdraw_stake() {
            return Err(e);
        }
        let sol_value_of_msol_burned = match self.lamports_over_supply().apply(msol_amount) {
            Some(v) => v,
            None => return Err(MarinadeCalcErr::Ratio),
        };
        let fee = match self.withdraw_stake_account_fee().to_fee_floor() {
            Some(f) => f,
            None => return Err(MarinadeCalcErr::Ratio),
        };
        let aaf = match fee.apply(sol_value_of_msol_burned) {
            Some(v) => v,
            None => return Err(MarinadeCalcErr::Ratio),
        };
        Ok(aaf.rem()..=aaf.rem())
    }

    #[inline]
    pub const fn svc_sol_to_lst(
        &self,
        lamports_amount: u64,
    ) -> Result<RangeInclusive<u64>, MarinadeCalcErr> {
        if let Err(e) = self.can_withdraw_stake() {
            return Err(e);
        }
        let fee = match self.withdraw_stake_account_fee().to_fee_floor() {
            Some(f) => f,
            None => return Err(MarinadeCalcErr::Ratio),
        };
        let r = match fee_floor_ratio_u32_u32_reverse_from_rem(fee, lamports_amount) {
            Some(a) => a,
            None => return Err(MarinadeCalcErr::Ratio),
        };
        let los = self.lamports_over_supply();
        let (min, max) = match (
            floor_ratio_u64_u64_reverse(los, *r.start()),
            floor_ratio_u64_u64_reverse(los, *r.end()),
        ) {
            (Some(min), Some(max)) => (*min.start(), *max.end()),
            _ => return Err(MarinadeCalcErr::Ratio),
        };
        Ok(min..=max)
    }
}

impl SolValCalc for MarinadeCalc {
    type Error = MarinadeCalcErr;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports_amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarinadeCalcErr {
    Ratio,
    Paused,
    StakeWithdrawDisabled,
}

impl Display for MarinadeCalcErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Ratio => "ratio math error",
            Self::Paused => "marinade program paused",
            Self::StakeWithdrawDisabled => "stake withdrawals disabled for marinade",
        })
    }
}

impl Error for MarinadeCalcErr {}
