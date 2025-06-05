import {
  address,
  getAddressDecoder,
  getBase64Encoder,
  getU64Decoder,
  type Address,
  type ReadonlyUint8Array,
} from "@solana/kit";
import { testFixturesAcc } from "./file";

const TOKEN_ACC_OWNER_OFFSET = 32;
const TOKEN_ACC_BALANCE_OFFSET = 64;

const MINT_SUPPLY_OFFSET = 36;

// mints
export const INF_MINT = address("5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm");
export const WSOL_MINT = address("So11111111111111111111111111111111111111112");
export const JUPSOL_MINT = address(
  "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v"
);

export function tokenAccOwner(accData: ReadonlyUint8Array): Address {
  return getAddressDecoder().decode(accData, TOKEN_ACC_OWNER_OFFSET);
}

export function tokenAccBalance(accData: ReadonlyUint8Array): bigint {
  return getU64Decoder().decode(accData, TOKEN_ACC_BALANCE_OFFSET);
}

export function mintSupply(accData: ReadonlyUint8Array): bigint {
  return getU64Decoder().decode(accData, MINT_SUPPLY_OFFSET);
}

export function testFixturesTokenAcc(tokenAccFname: string): {
  addr: Address;
  owner: Address;
} {
  const {
    pubkey,
    account: {
      data: [data],
    },
  } = testFixturesAcc(tokenAccFname);
  const owner = tokenAccOwner(getBase64Encoder().encode(data));
  return {
    addr: pubkey,
    owner,
  };
}
