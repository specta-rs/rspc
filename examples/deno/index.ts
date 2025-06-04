import { createClient, fetchExecute, sseExecute } from "@rspc/client/next";
import { Procedures } from "../bindings.ts";

const url = "http://[::]:4000/rspc";

const client = createClient<Procedures>((args) => {
  if (args.type === "subscription") return sseExecute({ url }, args);
  else return fetchExecute({ url, batch: true, stream: true }, args);
});

client.version.query().then((v) => console.log("version", v));
client.flush.query().then((v) => console.log("flush", v));
client.flush2.query().then((v) => console.log("flush2", v));

client.sendMsg.mutate("message").then(v => console.log("message", v))

client.basicSubscription.subscribe(void 0, {
  onData(value) {
    console.log(value)
  },
  onError(err) {
    console.error(err)
  },
  onComplete() {
    console.log("completed")
  },
})
