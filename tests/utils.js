const { Token, u64 } = require('@solana/spl-token')
const TokenInstructions = require('@project-serum/serum').TokenInstructions
const anchor = require('@project-serum/anchor')

const createToken = async ({ connection, wallet, mintAuthority }) =>
  Token.createMint(
    connection,
    wallet,
    mintAuthority,
    null,
    8,
    TokenInstructions.TOKEN_PROGRAM_ID
  )

const createAccountWithCollateral = async ({
  systemProgram,
  mintAuthority,
  collateralToken,
  amount = new anchor.BN(100 * 1e8)
}) => {
  const userWallet = await newAccountWithLamports(systemProgram.provider.connection)

  const userCollateralTokenAccount = await collateralToken.createAccount(userWallet.publicKey)
  await collateralToken.mintTo(
    userCollateralTokenAccount,
    mintAuthority,
    [],
    tou64(amount.toString())
  )

  /*
  await systemProgram.state.rpc.deposit({
    accounts: {
      userAccount: userAccount.publicKey,
      collateralAccount: collateralAccount
    },
    signers: [userWallet],
    instructions: [
      Token.createTransferInstruction(
        collateralToken.programId,
        userCollateralTokenAccount,
        collateralAccount,
        userWallet.publicKey,
        [],
        tou64(amount.toString())
      )
    ]
  })
  */
  return { userWallet, userCollateralTokenAccount }
}

const mintUsd = async ({
  userWallet,
  systemProgram,
  mintAmount,
  userTokenAccount,
  mintAuthority,
  vault,
  collateralToken,
  userCollateralTokenAccount
}) => {
  let amount = new anchor.BN(10 * 1e8)
  const state = await systemProgram.state()
  await systemProgram.state.rpc.mintCompleteSets(mintAmount, {
    accounts: {
      authority: mintAuthority,
      mint: state.outcomes[0].address,
      to: userTokenAccount,
      tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      owner: userWallet.publicKey,
      collateralAccount: vault
    },
    signers: [userWallet],
    instructions: [
      Token.createTransferInstruction(
        collateralToken.programId,
        userCollateralTokenAccount,
        vault,
        userWallet.publicKey,
        [],
        tou64(amount.toString())
      )
    ]
  })
}
const tou64 = (amount) => {
  // eslint-disable-next-line new-cap
  return new u64(amount.toString())
}
const newAccountWithLamports = async (connection, lamports = 1e10) => {
  const account = new anchor.web3.Account()

  let retries = 30
  await connection.requestAirdrop(account.publicKey, lamports)
  for (; ;) {
    await sleep(500)
    // eslint-disable-next-line eqeqeq
    if (lamports == (await connection.getBalance(account.publicKey))) {
      return account
    }
    if (--retries <= 0) {
      break
    }
  }
  throw new Error(`Airdrop of ${lamports} failed`)
}

const sleep = (ms) => {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

module.exports = {
  createToken,
  createAccountWithCollateral,
  mintUsd,
  tou64,
  newAccountWithLamports
}
