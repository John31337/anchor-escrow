import * as anchor from "@project-serum/anchor";
import { Program, BN, IdlAccounts } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { assert } from "chai";
import { Escrow } from "../target/types/escrow";
import {TextEncoder} from 'text-encoder'
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import { createProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

type EscrowAccount = IdlAccounts<Escrow>["escrowAccount"];

describe("escrow", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;

  let mint: Token = null;
  let initializerTokenAccount: PublicKey = null;
  let initializerTokenAccount2: PublicKey = null;
  
  let pda: PublicKey = null;

  const initializerAmount = 500;

  const escrowAccount = Keypair.generate();
  const payer = Keypair.generate();
  const buyer = Keypair.generate();
  const mintAuthority = Keypair.generate();
  const mint_key = Keypair.generate();


  it("Initialise escrow state", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );
    
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(buyer.publicKey, 10000000000),
      "confirmed"
    );

    mint = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    initializerTokenAccount = await mint.createAccount(
      provider.wallet.publicKey
    );
    initializerTokenAccount2 = await mint.createAccount(
      provider.wallet.publicKey
    );

    console.log(initializerTokenAccount.toBase58());

    await mint.mintTo(
      initializerTokenAccount,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerAmount
    );

    let _initializerTokenAccount = await mint.getAccountInfo(
      initializerTokenAccount
    );
    assert.ok(_initializerTokenAccount.amount.toNumber() == initializerAmount);
  });

  it("Initialize escrow", async () => {

    await program.rpc.list(
      new BN(initializerAmount),
      {
        accounts: {
          initializer: provider.wallet.publicKey,
          initializerTokenAccount: initializerTokenAccount,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          mintKey: mint_key.publicKey,
        },
        signers: [escrowAccount],
      }
    );

    // Get the PDA that is assigned authority to token account.
    const [_pda, _nonce] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("escrow")), escrowAccount.publicKey.toBuffer()],
      program.programId
    );

    pda = _pda;

    let _initializerTokenAccount = await mint.getAccountInfo(
      initializerTokenAccount
    );

    let escrows = await program.account.escrowAccount.all();
    console.log(escrows);
    (await escrows).forEach(element => {
      console.log(element);
    });

    let _escrowAccount: EscrowAccount =
      await program.account.escrowAccount.fetch(escrowAccount.publicKey);
    console.log(_escrowAccount);
    console.log(_escrowAccount.tokenAccountPubkey.toBase58());
    // Check that the new owner is the PDA.
    assert.ok(_initializerTokenAccount.owner.equals(pda));

    // Check that the values in the escrow account match what we expect.
    assert.ok(_escrowAccount.seller.equals(provider.wallet.publicKey));
    assert.ok(_escrowAccount.amount.toNumber() == initializerAmount);
    assert.ok(
      _escrowAccount.seller.equals(
        provider.wallet.publicKey
      )
    );
  });

  it("Exchange escrow", async () => {
    await program.rpc.buy({
      accounts: {
        buyer: buyer.publicKey,
        pdaDepositTokenAccount: initializerTokenAccount,
        initializerMainAccount: provider.wallet.publicKey,
        escrowAccount: escrowAccount.publicKey,
        pdaAccount: pda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      },
      signers:[buyer]
    });
  });

  let newEscrow = Keypair.generate();

  it("Initialize escrow and cancel escrow", async () => {
    // Put back tokens into initializer token A account.
    await mint.mintTo(
      initializerTokenAccount2,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerAmount
    );

    await program.rpc.list(
      new BN(initializerAmount),
      {
        accounts: {
          initializer: provider.wallet.publicKey,
          initializerTokenAccount: initializerTokenAccount2,
          escrowAccount: newEscrow.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          mintKey: mint_key,
        },
        signers: [newEscrow],
      }
    );

    let _initializerTokenAccount = await mint.getAccountInfo(
      initializerTokenAccount2
    );

    // Check that the new owner is the PDA.
    // assert.ok(_initializerTokenAccount.owner.equals(pda));

    // Cancel the escrow.
    // Get the PDA that is assigned authority to token account.
    const [_pda, _nonce] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("escrow")), newEscrow.publicKey.toBuffer()],
      program.programId
    );
  
    pda = _pda;
    await program.rpc.cancel({
      accounts: {
        user: provider.wallet.publicKey,
        pdaTokenAccount: initializerTokenAccount2,
        pdaAccount: pda,
        escrowAccount: newEscrow.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    // Check the final owner should be the provider public key.
    _initializerTokenAccount = await mint.getAccountInfo(
      initializerTokenAccount2
    );
    assert.ok(
      _initializerTokenAccount.owner.equals(provider.wallet.publicKey)
    );

    // Check all the funds are still there.
    assert.ok(_initializerTokenAccount.amount.toNumber() == initializerAmount);
  });
});