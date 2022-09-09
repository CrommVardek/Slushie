import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";

import { deployContract } from "./contract-deploy";
import { getRootHash } from "./get-root-hash";
import { deposit } from "./deposit";

import { KeyringPair } from "@polkadot/keyring/types";
import { withdraw } from "./withdraw";

// Set test timeout due to long proof generation
jest.setTimeout(350000);

// Global contract related variables
let wsProvider: WsProvider;
let api: ApiPromise;
let contract: ContractPromise;
let keyring: Keyring;
let alice: KeyringPair;

// Global logic related variables
let depositSize: number;
const commitments: Uint8Array[] = [];
const nullifiers_hash: Uint8Array[] = [];
const nullifiers: number[] = [];
const randomnesses: number[] = [];

// Setup connection with local node and deploy contract
beforeAll(async () => {
  wsProvider = new WsProvider();
  api = await ApiPromise.create({ provider: wsProvider });

  depositSize = Math.ceil(Math.random() * (100 - 1) + 1);

  keyring = new Keyring({ type: "sr25519" });
  alice = keyring.addFromUri("//Alice");

  contract = await deployContract(api, alice, depositSize);
});

describe("Deposits", () => {
  test("First deposit", async () => {
    await depositTest();
  });

  test("Second deposit", async () => {
    await depositTest();
  });
});

describe("Withdraws with wrong inputs", () => {
  test.failing("Withdraw with wrong nullifiers hash", async () => {
    // Change nullifier hash
    const wrongNullifierHash = new Uint8Array([...Array(32).keys()]);

    await withdraw(
      contract,
      alice,
      depositSize,
      wrongNullifierHash,
      nullifiers[0],
      randomnesses[0],
      0,
      keyring.decodeAddress("5D9x7yc3EcKoGPSETYRMWYVPmjpukbqvUHF6sRLvs5yktfeU"),
      commitments,
      keyring
    );
  });
  test.failing("Withdraw with wrong randomness", async () => {
    await withdraw(
      contract,
      alice,
      depositSize,
      nullifiers_hash[0],
      nullifiers[0],
      randomnesses[0] + 1,
      0,
      keyring.decodeAddress("5D9x7yc3EcKoGPSETYRMWYVPmjpukbqvUHF6sRLvs5yktfeU"),
      commitments,
      keyring
    );
  });

  test.failing("Withdraw with wrong nullifier", async () => {
    await withdraw(
      contract,
      alice,
      depositSize,
      nullifiers_hash[0],
      nullifiers[0] + 1,
      randomnesses[0],
      0,
      keyring.decodeAddress("5D9x7yc3EcKoGPSETYRMWYVPmjpukbqvUHF6sRLvs5yktfeU"),
      commitments,
      keyring
    );
  });

  test.failing("Withdraw with wrong commitments", async () => {
    // Change commitments
    const wrongCommitments = [...commitments];
    wrongCommitments[1] = new Uint8Array([...Array(32).keys()]);

    await withdraw(
      contract,
      alice,
      depositSize,
      nullifiers_hash[0],
      nullifiers[0],
      randomnesses[0],
      0,
      keyring.decodeAddress("5D9x7yc3EcKoGPSETYRMWYVPmjpukbqvUHF6sRLvs5yktfeU"),
      wrongCommitments,
      keyring
    );
  });
});

describe("Withdraws", () => {
  let usedProof: Uint8Array;

  test("Withdraw second", async () => {
    usedProof = await withdrawTest(1);
  });

  test("Withdraw first", async () => {
    await withdrawTest(0);
  });

  test.failing("Withdraw first, which has bes withdrawn", async () => {
    // Used same proof twice
    await withdrawTest(1, usedProof);
  });
});

// Disconnecting from node
afterAll(async () => {
  await wsProvider.disconnect();
  await api.disconnect();
});

/** Make deposit and check root hash changing */
async function depositTest(): Promise<number> {
  const initialRootHash = await getRootHash(contract, alice.address);

  const [k, r, c, h] = await deposit(contract, alice, depositSize);

  const changedRootHash = await getRootHash(contract, alice.address);

  expect(changedRootHash).not.toBe(initialRootHash);

  const index = commitments.length;
  commitments.push(c);
  nullifiers_hash.push(h);
  nullifiers.push(k);
  randomnesses.push(r);

  return index;
}

/** Make withdraw */
async function withdrawTest(
  index: number,
  useProof?: Uint8Array
): Promise<Uint8Array> {
  return await withdraw(
    contract,
    alice,
    depositSize,
    nullifiers_hash[index],
    nullifiers[index],
    randomnesses[index],
    index,
    keyring.decodeAddress("5D9x7yc3EcKoGPSETYRMWYVPmjpukbqvUHF6sRLvs5yktfeU"),
    commitments,
    keyring,
    useProof
  );
}
