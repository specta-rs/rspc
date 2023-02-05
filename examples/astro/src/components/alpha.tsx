// This file contains a demo of the new alpha API's these are stable to use but there may be rough edges.
/** @jsxImportSource solid-js */

import { fetchLink, initRspc } from "@rspc/client";
import { DEMO } from "@rspc/client/full";
import { Procedures } from "../../../bindings";

console.log(DEMO);

const rspc = initRspc<Procedures>().use(
  fetchLink({
    url: "http://localhost:4000/rspc",
  })
);

export default function AlphaPage() {
  // TODO: Don't copy this and use the vanilla client in SolidJS. It's not gonna end well in a more complicated app.
  rspc.query(["basic.echo", "Hello World"]).then(console.log);

  return <h1>Alpha</h1>;
}
