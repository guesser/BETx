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
  const admin = wallet
  const systemProgram = anchor.workspace.System
  const signer = new anchor.web3.Account()
  let collateralToken
  let mintAuthority
  let collateralAccount
  let syntheticUsd
  let nonce
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
      collateralAccount = await collateralToken.createAccount(mintAuthority)
      syntheticUsd = await createToken({ connection, wallet, mintAuthority })

      let outcomes = [
        syntheticUsd,
        collateralToken,
      ]
      let outcomesName = [
        'USD',
        'DAI',
      ]

      await systemProgram.state.rpc.initialize(
        _nonce, // Nonce
        signer.publicKey, // Signer
        wallet.publicKey, // Admin
        collateralToken.publicKey, // Collateral Token
        collateralAccount, // Collateral Account
        syntheticUsd.publicKey, // USD Token
        mintAuthority, // Mint Authority
        outcomes,
        outcomesName,
        2,
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
    assert.ok(state.collateralAccount.equals(collateralAccount))
    // initaly we will have collateral and sythetic usd
    assert.ok(state.outcomes.length === 2)
    assert.ok(state.outcomes[0].address.equals(syntheticUsd.publicKey))
    // initial collateralBalance
    const collateralAccountInfo = await collateralToken.getAccountInfo(collateralAccount)
    assert.ok(collateralAccountInfo.amount.eq(new anchor.BN(0)))
  })

  describe('#mint()', () => {
    const firstMintAmount = new anchor.BN(1 * 1e8)
    it('1st mint', async () => {
      const { userWallet } = await createAccountWithCollateral({
        collateralAccount,
        collateralToken,
        mintAuthority: wallet,
        systemProgram,
        amount: new anchor.BN(100 * 1e8)
      })
      const userTokenAccount = await syntheticUsd.createAccount(userWallet.publicKey)
      await mintUsd({
        userWallet,
        systemProgram,
        userTokenAccount, // To
        mintAuthority,
        mintAmount: firstMintAmount
      })
      const info = await syntheticUsd.getAccountInfo(userTokenAccount)
      assert.ok(info.amount.eq(firstMintAmount))
      // const account = await systemProgram.account.userAccount(userSystemAccount.publicKey)
      // assert.ok(state.shares.eq(firstMintShares)) // Its first mint so shares will be 1e8
    })
    it('2nd mint', async () => {
      const { userWallet } = await createAccountWithCollateral({
        collateralAccount,
        collateralToken,
        mintAuthority: wallet,
        systemProgram,
        amount: new anchor.BN(100 * 1e8)
      })

      const userTokenAccount = await syntheticUsd.createAccount(userWallet.publicKey)
      // We mint same amount
      await mintUsd({
        userWallet,
        systemProgram,
        userTokenAccount,
        mintAuthority,
        mintAmount: firstMintAmount
      })
      const info = await syntheticUsd.getAccountInfo(userTokenAccount)
      assert.ok(info.amount.eq(firstMintAmount))
    })
    it('3rd mint', async () => {
      const mintAmount = firstMintAmount.div(new anchor.BN(3)) // Mint 1/3
      const { userWallet } = await createAccountWithCollateral({
        collateralAccount,
        collateralToken,
        mintAuthority: wallet,
        systemProgram,
        amount: new anchor.BN(100 * 1e8)
      })
      const userTokenAccount = await syntheticUsd.createAccount(userWallet.publicKey)
      // We mint same amount
      await mintUsd({
        userWallet,
        systemProgram,
        userTokenAccount,
        mintAuthority,
        mintAmount: mintAmount
      })
      const info = await syntheticUsd.getAccountInfo(userTokenAccount)
      assert.ok(info.amount.eq(mintAmount))
    })
  })
})
