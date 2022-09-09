import { ContractPromise } from "@polkadot/api-contract";
import { KeyringPair } from "@polkadot/keyring/types";
import { generate_commitment } from "slushie";
import { eventDataToBytes, wait } from "./utils";

/** Generate commitment and call deposit contract method */
export async function deposit(
  contract: ContractPromise,
  signer: KeyringPair,
  depositSize: number
): Promise<[number, number, Uint8Array, Uint8Array]> {
  // Generate commitment
  const [k, r, c, h] = generate_commitment();

  const gasLimit = -1;
  const storageDepositLimit = null;
  // Call contract deposit method and then get commitment from event to test it
  const unsub = await contract.tx
    .deposit(
      {
        storageDepositLimit: storageDepositLimit,
        gasLimit: gasLimit,
        value: BigInt(depositSize) * 1000000000000n,
      },
      c
    )
    .signAndSend(signer, (result) => {
      if (result.status.isInBlock || result.status.isFinalized) {
        result.events.forEach(({ event: { data, method } }) => {
          if (method == "ContractEmitted") {
            // Event format is:
            // 1 byte for event type
            // 32 bytes for commitment
            // 8 bytes for timestamp
            const depositEventData = eventDataToBytes(data);
            // To get commitment, we should use bytes from 1 to 33
            const commitment = depositEventData.slice(1, 33);

            expect(c).toEqual(commitment);

            unsub();
          }
        });
      }
    });

  // Wait until deposit will be finished
  await wait(3000);

  return [k as number, r as number, c as Uint8Array, h as Uint8Array];
}
