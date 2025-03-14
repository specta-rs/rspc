import { createClient, fetchExecute, sseExecute } from "@rspc/client/next";

import { Procedures } from "../bindings.ts";

const url = "http://[::]:4000/rspc";

const client = createClient<Procedures>((args) => {
	if (args.type === "subscription") return sseExecute({ url }, args);
	else return fetchExecute({ url }, args);
});

client.basicSubscription.subscribe(undefined, {
	onData: (v) => console.log("onData", v),
});

client.version.query().then((v) => console.log("version", v));
