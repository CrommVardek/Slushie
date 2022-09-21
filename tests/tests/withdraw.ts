import { ContractPromise } from "@polkadot/api-contract";
import Keyring from "@polkadot/keyring";
import { KeyringPair } from "@polkadot/keyring/types";
import { generate_tree_opening, generate_proof_with_pd } from "slushie";
import { getRootHash } from "./get-root-hash";
import { getBalance, wait } from "./utils";

/** Generate proof and call withdraw contract method */
export async function withdraw(
  contract: ContractPromise,
  signer: KeyringPair,
  depositSize: number,
  nullifierHash: Uint8Array,
  k: number,
  r: number,
  leafIndex: number,
  recipient: Uint8Array,
  commitments: Uint8Array[],
  keyring: Keyring,
  useProof?: Uint8Array
): Promise<Uint8Array> {
  const gasLimit = -1;
  const storageDepositLimit = null;

  // Get root hash from contract
  const root = await getRootHash(contract, signer.address);

  // Generate tree opening using flattened array of commitments and leaf index
  const [treeOpening] = generate_tree_opening(
    commitments.reduce(
      (acc, commitment) => new Uint8Array([...acc, ...commitment])
    ),
    leafIndex
  );

  let proof;
  if (!useProof) {
    // Generate proof
    proof = generate_proof_with_pd(
      leafIndex,
      root,
      treeOpening,
      k,
      r,
      recipient,
      keyring.decodeAddress(signer.address),
      0n
    );
  } else {
    // Or use it from parameters
    proof = useProof;
  }

  // Get initial balance before withdraw
  const initialBalance = await getBalance(contract.api, recipient);

  // Call contract withdraw method
  let error = false;
  const unsub = await contract.tx
    .withdraw(
      {
        storageDepositLimit: storageDepositLimit,
        gasLimit: gasLimit,
      },
      {
        nullifierHash: nullifierHash,
        root: root,
        proof: proof,
        fee: 0,
        recipient: recipient,
      }
    )
    .signAndSend(signer, (result) => {
      if (result.status.isInBlock || result.status.isFinalized) {
        result.events.forEach(({ event: { method } }) => {
          if (method === "ExtrinsicFailed") {
            error = true;

            unsub();
          }
        });
      }
    });

  // Wait until withdraw will be finished
  await wait(50000);

  expect(error).toBe(false);

  // Get balance after withdraw
  const balance = await getBalance(contract.api, recipient);

  // Balance must change on the deposit value
  expect(balance).toBe(initialBalance + BigInt(depositSize) * 1000000000000n);

  return proof;
}
