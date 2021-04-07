/* eslint-disable new-cap */
const anchor = require('@project-serum/anchor')
const assert = require('assert')
const TokenInstructions = require('@project-serum/serum').TokenInstructions
// const { u64, Token } = require('@solana/spl-token')

const {
  createToken,
  createAccountWithCollateral,
  mintUsd,
} = require('../scripts/utils')

const fs = require('fs')

// Read the generated IDL.
const idl = JSON.parse(fs.readFileSync('./target/idl/system.json', 'utf8'));

// Address of the deployed program.
const programId = new anchor.web3.PublicKey(idl.metadata.address);

// Generate the program client from IDL.
const program = new anchor.Program(idl, programId);

const provider = anchor.Provider.local()
anchor.setProvider(provider)

const connection = provider.connection
const wallet = provider.wallet.payer

const systemProgram = program
const signer = new anchor.web3.Account()
let collateralToken
let mintAuthority
let vault
let outcomeA
let outcomeB
let nonce
const firstMintAmount = new anchor.BN(1 * 1e8)

const initProgram = async () => {

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

  const data = {
    programId: program.programId._bn,
    outcomeTokenA: outcomeA.publicKey._bn,
    outcomeTokenB: outcomeB.publicKey._bn,
    collateralToken: collateralToken.publicKey._bn,
    vault: vault._bn,
    creatorCollateralTokenAccount: userCollateralTokenAccount._bn,
    creator: userWallet.publicKey._bn,
    userTokenAccountA: userTokenAccountA.publicKey,
    userTokenAccountB: userTokenAccountB.publicKey,
  }

  const fileName = `market_${new Date().getTime()}.json`
  fs.writeFileSync(fileName, JSON.stringify(data, null, 2))

  console.log('Market data saved in ' + fileName);
}

initProgram()


