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
  if (!resp.ok) throw new Error(`Failed to fetch ${resp.status}`);
  if (resp.headers.get("content-type") !== "text/x-binario")
    throw new Error("Invalid content type");

  const result = new Uint8Array(await resp.clone().arrayBuffer());
  const expected = new Uint8Array([
    5, 0, 0, 0, 0, 0, 0, 0, 79, 115, 99, 97, 114,
  ]);
  if (!isEqualBytes(result, expected))
    throw new Error(`Result doesn't match expected value. Got ${result}`);

  console.log("Success!", result);

  const resp2 = await fetch(
    "http://localhost:4000/rspc/binario?procedure=streaming",
    {
      method: "POST",
      headers: {
        "Content-Type": "text/x-binario",
      },
      // { name: "Oscar" }
      body: new Uint8Array([5, 0, 0, 0, 0, 0, 0, 0, 79, 115, 99, 97, 114]),
    },
  );
  if (!resp2.ok) throw new Error(`Failed to fetch ${resp2.status}`);
  if (resp2.headers.get("content-type") !== "text/x-binario")
    throw new Error("Invalid content type");

  console.log(await resp2.arrayBuffer());
})();

function isEqualBytes(bytes1: Uint8Array, bytes2: Uint8Array): boolean {
  if (bytes1.length !== bytes2.length) {
    return false;
  }

  for (let i = 0; i < bytes1.length; i++) {
    if (bytes1[i] !== bytes2[i]) {
      return false;
    }
  }

  return true;
}
