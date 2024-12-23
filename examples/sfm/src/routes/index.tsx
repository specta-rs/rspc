import { Title } from "@solidjs/meta";
import { action, redirect, useAction, useHref } from "@solidjs/router";
import Counter from "~/components/Counter";
import { doMutation } from "~/lib";
import { FileRoutes } from "@solidjs/start/router";

const doThingAction = action(async (input: string) => {
  console.log("GOT:", input);

  // Basically:
  //  - `onSuccess` depends on knowing the return type, hence executing after `doMutation` has returned and causing a waterfall.
  //  - `redirect`ing on the server depends on know what data is dependant on the new router which requires running `preload` on the server. It can't be in a manifest as it's dynamic on params.

  await doMutation("DO THING");
  // , {
  //   onSuccess: (v) => {
  //     // redirect("/about");
  //   },
  // }

  console.log(redirect("/about")); // TODO

  // TODO: How can we see the `redirect` through this?
  // TODO: Knowing this redirect we need to run it's preload function in the same batch as a the mutation.
  return redirect("/about");
});

export default function Home() {
  const doThing = useAction(doThingAction);

  return (
    <main>
      <Title>Hello World</Title>
      <h1>Hello world!</h1>
      <Counter />
      <p>
        Visit{" "}
        <a href="https://start.solidjs.com" target="_blank">
          start.solidjs.com
        </a>{" "}
        to learn how to build SolidStart apps.
      </p>

      <button onClick={() => doThing("Hello World")}>Do Thing</button>

      <button onClick={() => getManifest()}>Manifest</button>
    </main>
  );
}

function getManifest() {
  const routes = FileRoutes();

  // routes.preload();

  console.log(routes);
}
