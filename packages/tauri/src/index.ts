import { OperationType, Transport, RSPCError } from "@rspc/client";
import { ProcedureKind } from "@rspc/client/next"; 
import { tauriExecute } from "./next";

export class TauriTransport implements Transport {
  async doRequest(
    operation: OperationType,
    key: string,
    input: any
  ): Promise<any> {
    return await new Promise((resolve, reject) => {
      const obs = tauriExecute({
        type: operation as ProcedureKind,
        path: key,
        input,
      });

      obs.subscribe((v) => {
        if (v?.code === 200) {
          resolve(v.value);
        } else {
          reject(new RSPCError(v?.code ?? 0, v?.value));
        }
      });
    });
  }
}
