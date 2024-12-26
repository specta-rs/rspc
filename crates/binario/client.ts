// TODO: This is not stable, just a demonstration of how it could work.

(async () => {
  const resp = await fetch(
    "http://localhost:4000/rspc/binario?procedure=binario",
    {
      method: "POST",
      headers: {
        "Content-Type": "text/x-binario",
      },
      // { name: "Oscar" }
      body: new Uint8Array([5, 0, 0, 0, 0, 0, 0, 0, 79, 115, 99, 97, 114]),
    },
  );

  console.log(resp.status, await resp.clone().arrayBuffer());
})();
