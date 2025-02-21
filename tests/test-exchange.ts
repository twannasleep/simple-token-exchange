import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
} from "@solana/spl-token";
import BN from "bn.js";

// Program ID from our deployment
const PROGRAM_ID = new PublicKey("F11d9Ct1MHaQhyRGR7TgyPavoCSPkrzhGf6Wh9g41JS");
const TEST_TOKEN = new PublicKey(
  "FKr7f2zhbjPyuyUoPw8uFyLYfQbaoGHwDA8a5ehr769B"
);

async function main() {
  // Connect to local test validator
  const connection = new Connection("http://localhost:8899", "confirmed");

  // Create test accounts
  const payer = Keypair.generate();
  const airdropSignature = await connection.requestAirdrop(
    payer.publicKey,
    2 * 1000000000 // 2 SOL
  );
  await connection.confirmTransaction(airdropSignature);

  // Create LP token mint
  const lpMint = await createMint(connection, payer, payer.publicKey, null, 9);

  // Create pool state account
  const poolState = Keypair.generate();
  const createPoolStateIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: poolState.publicKey,
    lamports: await connection.getMinimumBalanceForRentExemption(165),
    space: 165,
    programId: PROGRAM_ID,
  });

  // Initialize pool
  const initPoolIx = new Transaction().add({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: poolState.publicKey, isSigner: false, isWritable: true },
      { pubkey: TEST_TOKEN, isSigner: false, isWritable: false },
      { pubkey: lpMint, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: Buffer.from([
      0, // Initialize pool instruction
      ...new BN(1000000000).toArray("le", 8), // 1 SOL
      ...new BN(1000000000).toArray("le", 8), // 1 Token
      ...new BN(30).toArray("le", 8), // 0.3% fee
    ]),
  });

  // Send transaction
  const tx = new Transaction().add(createPoolStateIx, initPoolIx);

  try {
    const signature = await sendAndConfirmTransaction(connection, tx, [
      payer,
      poolState,
    ]);
    console.log("Pool initialized! Signature:", signature);
  } catch (error) {
    console.error("Error:", error);
  }
}

main();
