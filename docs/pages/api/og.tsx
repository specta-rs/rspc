import { NextRequest } from "next/server";
import { ImageResponse } from "@vercel/og";

const font = fetch(new URL("./Inter-SemiBold.otf", import.meta.url)).then(
  (res) => res.arrayBuffer()
);

export const config = {
  runtime: "edge",
};

export default async function handler(req: NextRequest) {
  const url = new URL(req.url);
  const title = url.searchParams.get("title") || "rspc";
  const description =
    url.searchParams.get("description") ||
    "The best way to build typesafe APIs between Rust and Typescript.";

  return new ImageResponse(
    (
      <div tw="bg-zinc-900 h-full w-full text-white bg-cover flex flex-col p-14">
        <div tw="flex flex-col justify-center items-center w-full h-full">
          <div tw="pt-3 flex justify-center items-center">
            <img src="https://rspc.dev/logo.png" width="240px" height="240px" />
          </div>
          <h1 tw="text-7xl py-3 tracking-tight">{title}</h1>
          <p tw="text-center text-3xl text-zinc-300 max-w-[600px]">
            {description}
          </p>
          <p tw="color-blue-500 text-2xl">rspc.dev</p>
        </div>
      </div>
    ),
    {
      width: 1200,
      height: 600,
      fonts: [{ name: "Inter", data: await font, weight: 900 }],
    }
  );
}
