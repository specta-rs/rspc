import { getSidebarData } from "../utils";

const rspcIcon = await (
  await import.meta.glob("../assets/*", { as: "raw" })["../assets/logo.svg"]
)();

const sidebar = await getSidebarData();

function classNames(...classes) {
  return classes.filter(Boolean).join(" ");
}

export default function Sidebar(props: { activePath: string }) {
  return (
    <nav class="h-screen shrink-0 lg:pl-96 bg-[#1A1A1A] px-8 overflow-y-auto overflow-x-hidden scrollbar-thin">
      <div class="w-52 flex flex-col items-center sticky top-0 pb-4 bg-[#1A1A1A]">
        <a
          innerHTML={rspcIcon}
          class="route p-4 [&>*]:w-24 [&>*]:h-24"
          href="/"
        />
        <h1 class="text-5xl font-extrabold">rspc</h1>
      </div>

      <div class="py-1 ">
        {[...sidebar.values()].map((category) => {
          return (
            <div class="mb-5" key={category.name}>
              <h2 class="font-semibold no-underline">{category.name}</h2>
              <ul class="mt-3">
                {category.children.map((page) => {
                  const active =
                    props.activePath === `/${page.url}/` ||
                    props.activePath === `/${page.url}`;
                  return (
                    <li
                      class={classNames(
                        "flex border-l border-gray-600",
                        active && "border-l-2 border-blue-500"
                      )}
                      key={page.title}
                    >
                      <a
                        href={`/${page.url}`}
                        class={classNames(
                          "route font-normal w-full rounded px-3 py-1 hover:text-gray-50 no-underline text-[14px] text-gray-350",
                          active && "!text-white !font-medium "
                        )}
                      >
                        {page.title}
                      </a>
                    </li>
                  );
                })}
              </ul>
            </div>
          );
        })}
      </div>
    </nav>
  );
}
