import {
  createContext,
  Fragment,
  PropsWithChildren,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { Listbox, Portal, Transition } from "@headlessui/react";
import { CheckIcon, ChevronUpDownIcon } from "@heroicons/react/20/solid";
import Image from "next/image";
import reactLogo from "../images/react-logo.svg";
import solidLogo from "../images/solid-logo.svg";
import vueLogo from "../images/vue-logo.svg";
import svelteLogo from "../images/svelte-logo.svg";
import pnpmLogo from "../images/pnpm-logo.svg";
import npmLogo from "../images/npm-logo.svg";
import yarnLogo from "../images/yarn-logo.svg";
import { usePopper } from "react-popper";

const LS_FW_KEY = "rspc-fw";
const frameworks = [
  { id: "react", logo: reactLogo, name: "React" },
  { id: "solid", logo: solidLogo, name: "Solid" },
  { id: "vue", logo: vueLogo, name: "Vue", disabled: true },
  { id: "svelte", logo: svelteLogo, name: "Svelte", disabled: true },
];

const LS_PM_KEY = "rspc-pm";
const packageManagers = [
  { id: "pnpm", logo: pnpmLogo, name: "pnpm" },
  { id: "npm", logo: npmLogo, name: "npm" },
  { id: "yarn", logo: yarnLogo, name: "Yarn" },
];

type CtxType = {
  activeFramework: (typeof frameworks)[0];
  activePackageManager: (typeof packageManagers)[0];
  setActiveFramework: (fw: (typeof frameworks)[0]) => void;
  setActivePackageManager: (pm: (typeof packageManagers)[0]) => void;
};
const ctx = createContext<CtxType>(undefined!);

export const Provider = ({ children }: PropsWithChildren) => {
  const [activeFramework, setActiveFramework] = useState(frameworks[0]);
  const [activePackageManager, setActivePackageManager] = useState(
    packageManagers[0]
  );

  useEffect(() => {
    const frameworkId = localStorage.getItem(LS_FW_KEY);
    const framework = frameworks.find((f) => f.id === frameworkId);
    if (framework) {
      setActiveFramework(framework);
    } else {
      localStorage.removeItem(LS_FW_KEY);
    }

    const packageManagerId = localStorage.getItem(LS_PM_KEY);
    const packageManager = packageManagers.find(
      (f) => f.id === packageManagerId
    );
    if (packageManager) {
      setActivePackageManager(packageManager);
    } else {
      localStorage.removeItem(LS_PM_KEY);
    }
  }, []);

  return (
    <ctx.Provider
      value={{
        activeFramework: activeFramework!,
        activePackageManager: activePackageManager!,
        setActiveFramework: (fw) => {
          setActiveFramework(fw);
          let framework = frameworks.find((f) => f.id === fw?.id);
          if (framework) {
            localStorage.setItem(LS_FW_KEY, framework.id);
          } else {
            localStorage.removeItem(LS_FW_KEY);
          }
        },
        setActivePackageManager: (pm) => {
          setActivePackageManager(pm);
          let packageManager = packageManagers.find((f) => f.id === pm?.id);
          if (packageManager) {
            localStorage.setItem(LS_PM_KEY, packageManager.id);
          } else {
            localStorage.removeItem(LS_PM_KEY);
          }
        },
      }}
    >
      {children}
    </ctx.Provider>
  );
};

export const useCtx = () => {
  const framework = useContext(ctx);

  if (!framework) {
    throw new Error("`FrameworkSwitcherProvider` not found");
  }

  return framework;
};

export function IfFramework({
  framework,
  children,
}: PropsWithChildren<{
  framework: string;
}>) {
  const { activeFramework } = useCtx();
  if (activeFramework.id === framework) {
    return <>{children}</>;
  }

  return null;
}

// This is horrible but Markdown code blocks don't have a system for interpolation and DOM elements don't have syntax highlighting.
export function Interpolate({ children }: PropsWithChildren) {
  const ref = useRef<HTMLDivElement>(null);
  const { activeFramework, activePackageManager } = useCtx();

  useEffect(() => {
    const modifiedNodes = new Set<any>();
    if (ref.current) {
      console.log(1);
      (function iterate_node(node: any) {
        if (node.nodeType === 3) {
          // Node.TEXT_NODE
          console.log(activePackageManager.name.toLowerCase());
          var text = node.data
            .replace("pnpm", activePackageManager.name.toLowerCase())
            .replace(
              "@rspc/react",
              `@rspc/${activeFramework.name.toLowerCase()}`
            );
          if (text != node.data) {
            modifiedNodes.add(node);
            console.log("NODE", typeof node);
            // there's a Safari bug
            node.data = text;
          }
        } else if (node.nodeType === 1) {
          // Node.ELEMENT_NODE
          for (var i = 0; i < node.childNodes.length; i++) {
            iterate_node(node.childNodes[i]); // run recursive on DOM
          }
        }
      })(ref.current);
    }

    return () => {
      for (const node of modifiedNodes) {
        node.data = node.data
          .replace(activePackageManager.name.toLowerCase(), "pnpm")
          .replace(
            `@rspc/${activeFramework.name.toLowerCase()}`,
            "@rspc/react"
          );
      }
    };
  }, [activeFramework, activePackageManager]);

  return <div ref={ref}>{children}</div>;
}

export function Switchers() {
  const {
    activeFramework,
    activePackageManager,
    setActiveFramework,
    setActivePackageManager,
  } = useCtx();

  const referenceElement = useRef<HTMLButtonElement>(null);
  const popperElement = useRef<HTMLDivElement>(null);
  let { styles, attributes } = usePopper(
    referenceElement.current,
    popperElement.current,
    {
      placement: "bottom-start",
    }
  );

  return (
    <div className="flex">
      <FrameworkSwitch />
      <PackageManagerSwitcher />
    </div>
  );
}

function FrameworkSwitch() {
  const { activeFramework, setActiveFramework } = useCtx();

  const referenceElement = useRef<HTMLButtonElement>(null);
  const popperElement = useRef<HTMLDivElement>(null);
  let { styles, attributes } = usePopper(
    referenceElement.current,
    popperElement.current,
    {
      placement: "bottom-start",
    }
  );

  return (
    <Listbox value={activeFramework} onChange={setActiveFramework}>
      <div className="flex-grow relative mt-1 pr-2">
        <Listbox.Button
          ref={referenceElement}
          className="relative w-full cursor-default rounded-lg bg-white hover:bg-gray-100 dark:bg-zinc-900 dark:hover:bg-zinc-800 transition-colors py-2 pl-3 pr-10 text-left shadow-md focus:outline-none focus-visible:border-blue-500 focus-visible:ring-2 focus-visible:ring-white dark:focus-visible:ring-black focus-visible:ring-opacity-75 focus-visible:ring-offset-2 focus-visible:ring-offset-blue-300 sm:text-sm"
        >
          <span className="flex truncate">
            {!!activeFramework && <Row framework={activeFramework} />}
          </span>
          <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
            <ChevronUpDownIcon
              className="h-5 w-5 text-gray-400"
              aria-hidden="true"
            />
          </span>
        </Listbox.Button>
        <Portal>
          <div ref={popperElement} style={styles.popper} {...attributes.popper}>
            <Transition
              as={Fragment}
              leave="transition ease-in duration-100"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <Listbox.Options className="absolute mt-1 max-h-60 w-36 overflow-auto rounded-md bg-white dark:bg-zinc-900 py-1 text-base shadow-lg ring-1 ring-black dark:ring-zinc-700 ring-opacity-5 focus:outline-none sm:text-sm z-10">
                {frameworks.map((framework) => (
                  <Listbox.Option
                    key={framework.id}
                    value={framework}
                    className={({ active }) =>
                      `flex relative cursor-default select-none py-2 pl-4 pr-4 hover:bg-gray-100  dark:hover:bg-zinc-800 ${
                        active ? "text-blue-500" : ""
                      } ${
                        framework.disabled
                          ? "cursor-not-allowed"
                          : "cursor-pointer"
                      }`
                    }
                    disabled={framework.disabled}
                  >
                    {({ selected }) => (
                      <Row framework={framework} selected={selected} />
                    )}
                  </Listbox.Option>
                ))}
              </Listbox.Options>
            </Transition>
          </div>
        </Portal>
      </div>
    </Listbox>
  );
}

function PackageManagerSwitcher() {
  const { activePackageManager, setActivePackageManager } = useCtx();

  const referenceElement = useRef<HTMLButtonElement>(null);
  const popperElement = useRef<HTMLDivElement>(null);
  let { styles, attributes } = usePopper(
    referenceElement.current,
    popperElement.current,
    {
      placement: "bottom-start",
    }
  );

  return (
    <Listbox value={activePackageManager} onChange={setActivePackageManager}>
      <div className="w-[60px] relative mt-1">
        <Listbox.Button
          ref={referenceElement}
          className="relative w-full cursor-default rounded-lg bg-white hover:bg-gray-100 dark:bg-zinc-900 dark:hover:bg-zinc-800 transition-colors py-2 pl-3 text-left shadow-md focus:outline-none focus-visible:border-blue-500 focus-visible:ring-2 focus-visible:ring-white dark:focus-visible:ring-black focus-visible:ring-opacity-75 focus-visible:ring-offset-2 focus-visible:ring-offset-blue-300 sm:text-sm"
        >
          <span className="flex truncate">
            {!!activePackageManager && (
              <Logo
                logo={activePackageManager.logo}
                alt={`${activePackageManager.name} logo`}
              />
            )}
          </span>
          <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
            <ChevronUpDownIcon
              className="h-5 w-5 text-gray-400"
              aria-hidden="true"
            />
          </span>
        </Listbox.Button>
        <Portal>
          <div ref={popperElement} style={styles.popper} {...attributes.popper}>
            <Transition
              as={Fragment}
              leave="transition ease-in duration-100"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <Listbox.Options className="absolute mt-1 max-h-60 w-32 overflow-auto rounded-md bg-white dark:bg-zinc-900 py-1 text-base shadow-lg ring-1 ring-black dark:ring-zinc-700 ring-opacity-5 focus:outline-none sm:text-sm z-10">
                {packageManagers.map((pkg) => (
                  <Listbox.Option
                    key={pkg.id}
                    value={pkg}
                    className={({ active }) =>
                      `flex relative cursor-default select-none py-2 pl-4 pr-4 hover:bg-gray-100  dark:hover:bg-zinc-800 ${
                        active ? "text-blue-500" : ""
                      }`
                    }
                  >
                    {({ selected }) => (
                      <Row framework={pkg} selected={selected} />
                    )}
                  </Listbox.Option>
                ))}
              </Listbox.Options>
            </Transition>
          </div>
        </Portal>
      </div>
    </Listbox>
  );
}

const Logo = ({ logo, alt }: { logo: string; alt: string }) => (
  <figure className="flex mr-2">
    <Image
      width={18}
      height={18}
      className="h-[20px] w-[20px]"
      src={logo}
      alt={alt}
    />
  </figure>
);

const Row = ({
  framework,
  selected,
}: {
  framework: (typeof frameworks)[0];
  selected?: boolean;
}) => (
  <>
    <Logo logo={framework.logo} alt={`${framework.name} logo`} />
    <span
      className={`block truncate ${selected ? "font-medium" : "font-normal"}`}
    >
      {framework.name}
    </span>
    {selected ? (
      <span className="absolute inset-y-0 right-0 flex items-center pr-3 text-blue-600">
        <CheckIcon className="h-5 w-5" aria-hidden="true" />
      </span>
    ) : null}
  </>
);
