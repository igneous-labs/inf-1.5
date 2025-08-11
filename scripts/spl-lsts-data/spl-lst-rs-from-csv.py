"""
Creates a jup-interface/src/consts/spl_lsts.rs file
from a `lst_mint,stake_pool_addr` csv created by csv-from-db.py
"""

import base58
import csv

CSV = "data.csv"

OUTFILE = "spl_lsts.rs"

# Pubkey::from_str_const

HEADER = """
#![cfg_attr(rustfmt, rustfmt_skip)] // else it'll crash rust-analyzer

// `static` instead of `const` because clippy says so:
// https://rust-lang.github.io/rust-clippy/master/index.html#large_const_arrays
/// TODO: figure out how to make this dynamic so that we can add spl stake pools
/// without updating the crate. Put this data onchain?
///
/// Currently only contains data for all the LSTs in INF
///
/// Array of `(spl_lst_mints, spl_stake_pool_addr)`
"""

def bs58str_to_arr(s):
    return [int(i) for i in base58.b58decode(s)]

if __name__ == "__main__":
    with open(CSV, "r") as c, open(OUTFILE, "w") as f:
        csvreader = csv.reader(c)
        header = next(csvreader)
        if header != ["mint", "pool"]:
            raise Exception(f"unexpected header {header}")
        
        lines = []
        total = 0
        for [mint, pool] in csvreader:
            lines.append(f'({bs58str_to_arr(mint)}, {bs58str_to_arr(pool)}),\n')
            total += 1

        f.write(HEADER)
        f.write(f'pub static SPL_LSTS: [([u8; 32], [u8; 32]); {total}] = [\n')
        f.writelines(lines)
        f.write("];\n")
