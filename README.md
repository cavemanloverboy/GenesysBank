# GenesysBank

There are four anchor instructions:
![genesys_banking](https://user-images.githubusercontent.com/93507302/157431128-b1e9b1af-141e-4c28-899c-e2a6a147c040.png)

1) initialize: initializes an empty vault.
2) refreshReserve: lets the vaultAdmin top off the tokenVault (the reserve)
3) deposit: lets users deposit and specify the lockup time
4) withdraw: lets users withdraw after specified lockup time

There is a test script with 4 mocha tests:
1) initializes an empty vault
2) refreshes it (tops it off)
3) airdrops SOL + FEET token to user and deposits 100,000 FEET
4) waits 4 seconds and then withdraws tokens + interest

This is a mvp with some limitations, all which are easily fixable:
1) A user cannot have multiple deposit boxes.
2) A user cannot deposit more funds in the same box.
3) The admin is given mint authority, which means they can mint tokens themselves as well (my people would say that this is "no bueno")
4) There are a few lazy castings between signed/unsigned integers and f64's that I would want to think more about re: security.

I am aware a set of keypairs (admin, mint, user) are attached to this; this is something I am doing only for the purpose of this demo.
