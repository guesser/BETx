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

  return { userWallet, userCollateralTokenAccount }
}

const claimProfits = async ({
  userWallet,
  systemProgram,
  mintAmount,
  winnerFrom,
  mintAuthority,
  vault,
  winnerOutcome,
  userCollateralTokenAccount,
}) => {
  const approveTx1 = Token.createApproveInstruction(
    winnerOutcome.programId,
    winnerFrom,
    mintAuthority,
    userWallet.publicKey,
    [],
    tou64(mintAmount)
  )

  await systemProgram.state.rpc.claimProfits(mintAmount, {
    accounts: {
      to: userCollateralTokenAccount,
      authority: mintAuthority,
      winnerFrom: winnerFrom,
      tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      owner: userWallet.publicKey,
      collateralAccount: vault,
      winnerOutcome: winnerOutcome.publicKey,
    },
    signers: [userWallet],
    instructions: [approveTx1]
  })
}

const redeemCompleteSets = async ({
  userWallet,
  systemProgram,
  mintAmount,
  userTokenAccountA,
  userTokenAccountB,
  mintAuthority,
  vault,
  outcomeA,
  outcomeB,
  userCollateralTokenAccount,
}) => {
  const approveTx1 = Token.createApproveInstruction(
    outcomeA.programId,
    userTokenAccountA,
    mintAuthority,
    userWallet.publicKey,
    [],
    tou64(mintAmount)
  )
  const approveTx2 = Token.createApproveInstruction(
    outcomeB.programId,
    userTokenAccountB,
    mintAuthority,
    userWallet.publicKey,
    [],
    tou64(mintAmount)
  )
  let result = await systemProgram.state.rpc.redeemCompleteSets(mintAmount, {
    accounts: {
      to: userCollateralTokenAccount,
      authority: mintAuthority,
      to1: userTokenAccountA,
      to2: userTokenAccountB,
      tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      owner: userWallet.publicKey,
      collateralAccount: vault,
      outcome1: outcomeA.publicKey,
      outcome2: outcomeB.publicKey,
    },
    signers: [userWallet],
    instructions: [approveTx1, approveTx2]
  })
}

const mintUsd = async ({
  userWallet,
  systemProgram,
  mintAmount,
  userTokenAccountA,
  userTokenAccountB,
  mintAuthority,
  vault,
  collateralToken,
  userCollateralTokenAccount,
  outcomeA,
  outcomeB,
}) => {
  await systemProgram.state.rpc.mintCompleteSets(mintAmount, {
    accounts: {
      authority: mintAuthority,
      to1: userTokenAccountA,
      to2: userTokenAccountB,
      tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      // clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      owner: userWallet.publicKey,
      collateralAccount: vault,
      outcome1: outcomeA.publicKey,
      outcome2: outcomeB.publicKey,
    },
    signers: [userWallet],
    instructions: [
      Token.createTransferInstruction(
        collateralToken.programId,
        userCollateralTokenAccount,
        vault,
        userWallet.publicKey,
        [],
        tou64(mintAmount.toString())
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
  newAccountWithLamports,
  redeemCompleteSets,
  claimProfits,
}
