const anchor = require('@project-serum/anchor')
const {
  createToken,
} = require('./utils')
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

// Execute the RPC.
const createMarket = async (creator, programId, connection, wallet, endTimestamp) => {
  await program.state.rpc.new({
    accounts: {}
  })
  const [mintAuthority, nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [creator.publicKey.toBuffer()],
    programId
  )

  let collateralToken = await createToken({ connection, wallet, mintAuthority: wallet.publicKey })
  let vault = await collateralToken.createAccount(mintAuthority)
  let creatorCollateralTokenAccount = await collateralToken.createAccount(creator.publicKey)

  let outcomeB = await createToken({ connection, wallet, mintAuthority })
  let outcomeA = await createToken({ connection, wallet, mintAuthority })

  await program.state.rpc.initialize(
    nonce, // Nonce
    creator.publicKey, // Signer
    wallet.publicKey, // Oracle
    collateralToken.publicKey, // Collateral Token
    vault, // Vault
    mintAuthority, // Mint Authority
    new anchor.BN(endTimestamp),
    outcomeA.publicKey,
    outcomeB.publicKey,
    {
      accounts: {}
    }
  )

  const data = {
    programId: programId._bn,
    outcomeTokenA: outcomeA.publicKey._bn,
    outcomeTokenB: outcomeB.publicKey._bn,
    collateralToken: collateralToken.publicKey._bn,
    vault: vault._bn,
    creatorCollateralTokenAccount: creatorCollateralTokenAccount._bn,
    creator: creator.publicKey._bn,
  }

  fs.writeFileSync(`market_${new Date().getTime()}.json`, JSON.stringify(data, null, 2))
}

const creator = new anchor.web3.Account() //User account
const endTimestamp = new Date().getTime() / 1000 //Set by the User

createMarket(creator, programId, connection, wallet, endTimestamp)
