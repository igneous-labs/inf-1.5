"""
Creates a `lst_mint,stake_pool_addr` csv from sanctum's LST postgres DB.
All pubkeys base58-encoded
"""

import csv
import psycopg

DB = "postgres://sanctum:sanctum@localhost:5432/sanctum?sslmode=disable"
QUERY = "SELECT mint, pool FROM lst_metadata"

OUTFILE = "data.csv"

if __name__ == "__main__":
    conn = psycopg.connect(DB)
    cur = conn.cursor()
    cur.execute(QUERY)
    rows = cur.fetchall()

    with open(OUTFILE, "w") as f:
        w = csv.writer(f)
        w.writerow(["mint", "pool"])
        for (mint, data) in rows:
            program = data.get("program")
            if program != "SanctumSpl" and program != "SanctumSplMulti" and program != "Spl":
                continue
            w.writerow([mint, data["pool"]])
