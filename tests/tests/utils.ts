import { ApiBase } from "@polkadot/api/base";
import { Codec, IEventData } from "@polkadot/types/types";

/** Wait for transaction finishing to get produced events data, because test-runner does not wait*/
export async function wait(delay: number) {
  return await new Promise((resolve) => {
    setTimeout(() => {
      resolve(1);
    }, delay);
  });
}

/** Convert event data to bytes array */
export function eventDataToBytes(data: Codec[] & IEventData): Uint8Array {
  const eventDataHex = hexStringToBytes(
    (data.toHuman() as Record<string, string>).data
  );

  return eventDataHex;
}

/** Convert hex string to bytes array */
export function hexStringToBytes(str: string): Uint8Array {
  const eventDataHex = str
    .slice(2) // to remove first "0x"
    .match(/../g) // matches every two characters
    ?.map((h: string) => parseInt(h, 16)); // parse every two characters from hex string to number

  if (eventDataHex === undefined) {
    throw Error("String is not in a HEX format");
  }

  return new Uint8Array(eventDataHex);
}

/** Get balance of address */
export async function getBalance(
  api: ApiBase<"promise">,
  address: string | Uint8Array
): Promise<bigint> {
  const hash = (await api.rpc.chain.getHeader()).hash;
  const apiAt = await api.at(hash);
  const balance = await apiAt.query.system.account(address);

  return BigInt(
    (balance.toPrimitive() as Record<string, Record<string, string>>).data.free
  );
}
