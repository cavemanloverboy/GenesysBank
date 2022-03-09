const assert = require("assert");
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { GenesysBanking } from "../target/types/genesys_banking";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  mintTo,
  transfer,
  createAccount,
} from "@solana/spl-token";
import { rpc, token } from "@project-serum/anchor/dist/cjs/utils";
const fs = require("fs");

const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: anchor.web3.PublicKey =
  new anchor.web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const debug = false;
if (!debug) {
  console.log = function () {};
}

async function findAssociatedTokenAddress(
  walletAddress: anchor.web3.PublicKey,
  tokenMintAddress: anchor.web3.PublicKey
): Promise<anchor.web3.PublicKey> {
  return (
    await anchor.web3.PublicKey.findProgramAddress(
      [
        walletAddress.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        tokenMintAddress.toBuffer(),
      ],
      SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    )
  )[0];
}

describe("genesys-banking", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GenesysBanking as Program<GenesysBanking>;
  let programConstants = Object.assign(
    {},
    ...program.idl.constants.map((x) => ({ [x.name]: x.value.slice(1, -1) }))
  );

  // Get admin key
  const vaultAdmin = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(
      fs
        .readFileSync("FRANKC3ibsaBW1o2qRuu3kspyaV4gHBuUfZ5uq9SXsqa.json", {
          encoding: "utf8",
          flag: "r",
        })
        .slice(1, -1)
        .split(",")
    )
  );
  // Get mint key
  const tokenMint = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(
      fs
        .readFileSync("FEETa25ux7dDxeuJCJnxWkREpFBy7RMNUA24Gyi4ZPp1.json", {
          encoding: "utf8",
          flag: "r",
        })
        .slice(1, -1)
        .split(",")
    )
  );
  // Get user key
  const user = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(
      fs
        .readFileSync("CAVEYgsWyeEAkAwXSA3tnqNhvFnztQkuwW1ZKfDBf9Za.json", {
          encoding: "utf8",
          flag: "r",
        })
        .slice(1, -1)
        .split(",")
    )
  );

  it("Vault is initialized!", async () => {
    let [vaultInfo, infoBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(
          anchor.utils.bytes.utf8.encode(programConstants["VAULT_INFO_SEED"])
        ),
      ],
      program.programId
    );

    let [tokenVault, vaultBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(programConstants["TOKEN_VAULT_SEED"])
          ),
        ],
        program.programId
      );

    console.log("admin:", vaultAdmin.publicKey.toString());
    console.log("mint:", tokenMint.publicKey.toString());
    console.log("user:", user.publicKey.toString());
    console.log("token program:", TOKEN_PROGRAM_ID.toString());
    console.log(
      "system program:",
      anchor.web3.SystemProgram.programId.toString()
    );
    console.log(
      "vaultInfo(seed=",
      programConstants["VAULT_INFO_SEED"],
      "):",
      vaultInfo.toString()
    );
    console.log(
      "tokenVault(seed =",
      programConstants["TOKEN_VAULT_SEED"],
      "):",
      tokenVault.toString()
    );

    // Add your test here.
    const tx = await program.rpc.initialize({
      accounts: {
        vaultInfo: vaultInfo,
        tokenVault: tokenVault,
        tokenMint: tokenMint.publicKey,
        vaultAdmin: vaultAdmin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [vaultAdmin, tokenMint],
    });
    console.log("Your transaction signature", tx);
  });

  it("Vault admin refreshed FEET reserve account!", async () => {
    // Grab Vault Info PDA + bump
    let [vaultInfo, infoBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(
          anchor.utils.bytes.utf8.encode(programConstants["VAULT_INFO_SEED"])
        ),
      ],
      program.programId
    );

    // Grab token vault/reseve PDA + bump
    let [tokenVault, reserveBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(programConstants["TOKEN_VAULT_SEED"])
          ),
        ],
        program.programId
      );

    let prevTokenVaultBalance =
      await provider.connection.getTokenAccountBalance(tokenVault);
    const tx = await program.rpc.refreshReserve(infoBump, reserveBump, {
      accounts: {
        vaultInfo: vaultInfo,
        tokenVault: tokenVault,
        tokenMint: tokenMint.publicKey,
        vaultAdmin: vaultAdmin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        program: program.programId,
      },
      signers: [vaultAdmin],
    });
    console.log("Your transaction signature", tx);

    let finalTokenVaultBalance =
      await provider.connection.getTokenAccountBalance(tokenVault);
    console.log("prev token Balance =", prevTokenVaultBalance.value.amount);
    console.log("post token Balance =", finalTokenVaultBalance.value.amount);
  });

  it("User deposits FEET with 3 sec lockout!", async () => {
    // airdrop user some SOL
    const transfer_tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: vaultAdmin.publicKey,
        toPubkey: user.publicKey,
        lamports: 1 * anchor.web3.LAMPORTS_PER_SOL,
      })
    );

    const signature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      transfer_tx,
      [vaultAdmin]
    );
    console.log("SOL xfer to user signature:", signature);
    let solBalance = await provider.connection.getBalance(user.publicKey);
    console.log("user SOL lamports balance is ", solBalance);

    // now airdrop some SPL
    // find user ATA
    let userATA = await findAssociatedTokenAddress(
      user.publicKey,
      tokenMint.publicKey
    );
    console.log("user FEET ATA is", userATA.toString());
    // create ATA
    let created_ATA = await createAccount(
      provider.connection,
      user,
      tokenMint.publicKey,
      user.publicKey
    );
    console.log("account created (should match):", created_ATA.toString());
    let airdrop_tx = await mintTo(
      provider.connection,
      user,
      tokenMint.publicKey,
      userATA, //user.publicKey,
      vaultAdmin,
      100000
    );
    console.log("airdrop_tx =", airdrop_tx);

    let tokenBalance = await provider.connection.getTokenAccountBalance(
      userATA
    );
    console.log("user FEET balance is ", tokenBalance.value.amount);

    let [vaultInfo, infoBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(
          anchor.utils.bytes.utf8.encode(programConstants["VAULT_INFO_SEED"])
        ),
      ],
      program.programId
    );

    let [tokenVault, reserveBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(programConstants["TOKEN_VAULT_SEED"])
          ),
        ],
        program.programId
      );

    let [depositInfo, depositInfoBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(
              programConstants["USER_DEPOSIT_INFO"]
            )
          ),
          user.publicKey.toBuffer(),
        ],
        program.programId
      );
    let [userVault, userVaultBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(programConstants["USER_VAULT_SEED"])
          ),
          user.publicKey.toBuffer(),
        ],
        program.programId
      );
    console.log("user vault is", userVault.toString());

    let tx = await program.rpc.deposit(
      reserveBump,
      infoBump,
      depositInfoBump,
      userVaultBump,
      new anchor.BN(3),
      new anchor.BN(100000),
      {
        accounts: {
          depositInfo: depositInfo,
          vaultInfo: vaultInfo,
          userVault: userVault,
          tokenMint: tokenMint.publicKey,
          vaultAdmin: vaultAdmin.publicKey,
          depositor: user.publicKey,
          depositorTokenAccount: userATA,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        },
        signers: [user],
      }
    );
    let vaultBalance = await provider.connection.getTokenAccountBalance(
      userVault
    );
    console.log("Asserting vault balance equals deposited amount");
    assert(vaultBalance.value.amount == "100000");
  });

  it("User withdraws FEET after 4 sec of waiting!", async () => {
    await new Promise((f) => setTimeout(f, 4000));

    let userATA = await findAssociatedTokenAddress(
      user.publicKey,
      tokenMint.publicKey
    );

    let [vaultInfo, infoBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(
          anchor.utils.bytes.utf8.encode(programConstants["VAULT_INFO_SEED"])
        ),
      ],
      program.programId
    );

    let [tokenVault, reserveBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(programConstants["TOKEN_VAULT_SEED"])
          ),
        ],
        program.programId
      );

    let [depositInfo, depositInfoBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(
              programConstants["USER_DEPOSIT_INFO"]
            )
          ),
          user.publicKey.toBuffer(),
        ],
        program.programId
      );
    let [userVault, userVaultBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(
            anchor.utils.bytes.utf8.encode(programConstants["USER_VAULT_SEED"])
          ),
          user.publicKey.toBuffer(),
        ],
        program.programId
      );

    let tx = await program.rpc.withdraw(
      reserveBump,
      infoBump,
      depositInfoBump,
      userVaultBump,
      {
        accounts: {
          depositInfo: depositInfo,
          vaultInfo: vaultInfo,
          userVault: userVault,
          tokenVault: tokenVault,
          tokenMint: tokenMint.publicKey,
          vaultAdmin: vaultAdmin.publicKey,
          depositor: user.publicKey,
          depositorTokenAccount: userATA,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          program: program.programId,
        },
        signers: [user],
      }
    );

    let userBalance = await provider.connection.getTokenAccountBalance(userATA);
    console.log("Asserting user balance is greater than deposited amount");
    console.log("new user balance is", parseInt(userBalance.value.amount));
    assert(parseInt(userBalance.value.amount) > 100000);
  });
});
