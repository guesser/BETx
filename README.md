# BETx Solana Program
Solana Program built with Anchor, in order to be able to work with it you would need to follow the instructions from [here](https://project-serum.github.io/anchor/getting-started/installation.html) to install it. Once you have that, ```anchor test``` is your friend to execute fast calls to the program functions.
If you want to use it in devnet do ```anchor deploy``` and then execute the client.js file in scripts directory to give you the desired addresses of the contract and outcomes. There are some things that are still hardcoded, you may change them if you want, the code is easy to understand.


### Functions:
- new(): Sets the "program state" with default values
- initialize(): Sets and fixes the "program state" with the desired values. That is, setting the following values: signer, nonce, oracle, collateral_token, vault, mint_authority (program address), expiration_time, outcome1 address and outcome2 address
- mint_complete_sets(): Receives a transaction in its vault, mints that amount of Tokens of outcome1 and outcome2 and sends them to the user
- redeem_complete_sets(): Burns N amount of tokens from the user account, from outcome1 and outcome2. Then, transacts N tokens from the collateral vault to the user in exchange for the tokens sent.
- resolve_market(): Checks a few things and sets the winner the oracle sends
- claim_profits(): Burns N amount of winner outcome tokens from the user account. Then transacts N tokens from the collateral vault to the user

### Utils
A helper that provides anyone the needed js features to interact with the program such us creating the needed accounts, deployments and helpers

### Tests
It deploys and tests thoroughly all the needed requirements set initially for the project. 

### Program Workflow
1. Deploy Program
2. Call the new() function
3. Create the outcomes setting the owner the program account
4. Call the initilize() function sending the arguments displayed in the functions section.
5. Mint complete sets with the function provided by sending the collateral token to the specified Vault
6. Create a Serum Market with WSOL and Outcome1 and WSOL Outcome2 (might be helpful Outcome1<>Outcome2 as well)
7. Provide liquidity with the complete sets you got
8. Trade permissionlessly in Serum, no need to interact with the program
9. Redeem the spared complete sets (same amount of Outcome1 and Outcome2) that you don't want to trade anymore. You'll get the appropriate collateral back
10. Repeat from point 5 as much as you want, anyone can participate, not only the oracle/signer 
11. Expiration time passes
12. The oracle sets the winner with the resolve_market()
13. People that have the winner outcome call the claim_profits() function to get the proper amount of the collateral token back
