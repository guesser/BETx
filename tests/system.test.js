/* eslint-disable new-cap */
const anchor = require('@project-serum/anchor')
const assert = require('assert')
// const TokenInstructions = require('@project-serum/serum').TokenInstructions
// const { u64, Token } = require('@solana/spl-token')

const {
  createToken,
  createAccountWithCollateral,
  mintUsd,
  // newAccountWithLamports
} = require('./utils')

describe('system', () => {
  // const provider = anchor.Provider.local('https://devnet.solana.com', {
  //   commitment: 'max',
  //   preflightCommitment: 'max',
  //   skipPreflight: true
  // })
  const provider = anchor.Provider.local()
  anchor.setProvider(provider)
  const connection = provider.connection
  const wallet = provider.wallet.payer
  const systemProgram = anchor.workspace.System
  const signer = new anchor.web3.Account()
  let collateralToken
  let mintAuthority
  let vault
  let outcomeA
  let outcomeB
  let nonce
  let outcomes
  before(async () => {
    try {
      await systemProgram.state.rpc.new({
        accounts: {}
      })
      const [_mintAuthority, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
        [signer.publicKey.toBuffer()],
        systemProgram.programId
      )
      nonce = _nonce
      mintAuthority = _mintAuthority

      collateralToken = await createToken({ connection, wallet, mintAuthority: wallet.publicKey })
      vault = await collateralToken.createAccount(mintAuthority)

      outcomeB = await createToken({ connection, wallet, mintAuthority })
      outcomeA = await createToken({ connection, wallet, mintAuthority })

      await systemProgram.state.rpc.initialize(
        _nonce, // Nonce
        signer.publicKey, // Signer
        wallet.publicKey, // Oracle
        collateralToken.publicKey, // Collateral Token
        vault, // Vault
        mintAuthority, // Mint Authority
        new anchor.BN(1617100690),
        outcomeA.publicKey,
        outcomeB.publicKey,
        {
          accounts: {}
        }
      )
    } catch (error) {
      console.log(error)
    }
  })

  it('Check initialState', async () => {
    const state = await systemProgram.state()
    assert.ok(state.nonce === nonce)
    assert.ok(state.signer.equals(signer.publicKey))
    assert.ok(state.collateralToken.equals(collateralToken.publicKey))
    assert.ok(state.vault.equals(vault))
    // initaly we will have collateral and sythetic usd
    assert.ok(state.outcome1.address.equals(outcomeA.publicKey))
    assert.ok(state.outcome2.address.equals(outcomeB.publicKey))
    // initial collateralBalance
    const vaultInfo = await collateralToken.getAccountInfo(vault)
    assert.ok(vaultInfo.amount.eq(new anchor.BN(0)))
  })

  /*
  it('Try to re-initialState', async () => {
    try {
      await systemProgram.state.rpc.initialize(
        0, // Nonce
        signer.publicKey, // Signer
        wallet.publicKey, // Oracle
        collateralToken.publicKey, // Collateral Token
        vault, // Vault
        mintAuthority, // Mint Authority
        outcomes,
        // outcomesName,
        4,
        new anchor.BN(1917100690),
        {
          accounts: {}
        }
      )
      assert.ok(true === false)
    } catch (error) {
      const state = await systemProgram.state()
      assert.ok(state.nonce === nonce)
      assert.ok(state.signer.equals(signer.publicKey))
      assert.ok(state.collateralToken.equals(collateralToken.publicKey))
      assert.ok(state.vault.equals(vault))
      // initaly we will have collateral and sythetic usd
      assert.ok(state.outcomes.length === 2)
      assert.ok(state.outcomes[0].address.equals(outcomeA.publicKey))
      // initial collateralBalance
      const vaultInfo = await collateralToken.getAccountInfo(vault)
      assert.ok(vaultInfo.amount.eq(new anchor.BN(0)))
    }
  })
  */

  describe('#mint()', () => {
    const firstMintAmount = new anchor.BN(1 * 1e8)
    it('1st mint', async () => {
      // We give the user an account with USD
      const { userWallet, userCollateralTokenAccount } = await createAccountWithCollateral({
        vault,
        collateralToken,
        mintAuthority: wallet,
        systemProgram,
        amount: new anchor.BN(100 * 1e8)
      })
      const userTokenAccountA = await outcomeA.createAccount(userWallet.publicKey)
      const userTokenAccountB = await outcomeB.createAccount(userWallet.publicKey)
      await mintUsd({
        userWallet,
        systemProgram,
        userTokenAccountA, // To
        userTokenAccountB, // To
        mintAuthority,
        mintAmount: firstMintAmount,
        vault,
        collateralToken,
        userCollateralTokenAccount,
        outcomeA,
        outcomeB,
      })
      const info = await outcomeA.getAccountInfo(userTokenAccountA)
      assert.ok(info.amount.eq(firstMintAmount))
      // const account = await systemProgram.account.userAccount(userSystemAccount.publicKey)
      // assert.ok(state.shares.eq(firstMintShares)) // Its first mint so shares will be 1e8
    })
    it('2nd mint', async () => {
      const { userWallet, userCollateralTokenAccount } = await createAccountWithCollateral({
        vault,
        collateralToken,
        mintAuthority: wallet,
        systemProgram,
        amount: new anchor.BN(100 * 1e8)
      })

      const userTokenAccountA = await outcomeA.createAccount(userWallet.publicKey)
      const userTokenAccountB = await outcomeB.createAccount(userWallet.publicKey)
      // We mint same amount
      await mintUsd({
        userWallet,
        systemProgram,
        userTokenAccountA,
        userTokenAccountB,
        mintAuthority,
        mintAmount: firstMintAmount,
        vault,
        collateralToken,
        userCollateralTokenAccount,
        outcomeA,
        outcomeB,
      })
      const info = await outcomeA.getAccountInfo(userTokenAccountA)
      assert.ok(info.amount.eq(firstMintAmount))
    })
    it('3rd mint', async () => {
      const mintAmount = firstMintAmount.div(new anchor.BN(3)) // Mint 1/3
      const { userWallet, userCollateralTokenAccount } = await createAccountWithCollateral({
        vault,
        collateralToken,
        mintAuthority: wallet,
        systemProgram,
        amount: new anchor.BN(100 * 1e8)
      })
      const userTokenAccountA = await outcomeA.createAccount(userWallet.publicKey)
      const userTokenAccountB = await outcomeB.createAccount(userWallet.publicKey)
      // We mint same amount
      await mintUsd({
        userWallet,
        systemProgram,
        userTokenAccountA,
        userTokenAccountB,
        mintAuthority,
        mintAmount: mintAmount,
        vault,
        collateralToken,
        userCollateralTokenAccount,
        outcomeA,
        outcomeB,
      })
      const info = await outcomeA.getAccountInfo(userTokenAccountA)
      assert.ok(info.amount.eq(mintAmount))
    })
  })
})
