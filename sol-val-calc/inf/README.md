# inf1-svc

The INF SOL Value Calculator interface and program.

This is not used by the main INF program but by other programs as a redemption rate for INF.

## Behavioural Differences

Given that pricing programs can impose arbitrary fees, even varying it depending on the token being redeemed, this SOL value calculator program does not account for redemption fees unlike the other stake pool programs.

To reduce the risk of losing SOL value, consumers of this program should account for this and possibly value INF lower than the values returned by the SOL value calculator interface instructions.
