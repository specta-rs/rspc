import Image from "next/image";
import { useRouter } from "next/router";
import { DocsThemeConfig } from "nextra-theme-docs";
import { Switchers } from "./components/Switchers";
import logoIcon from "./public/logo.png";

const config: DocsThemeConfig = {
  docsRepositoryBase: "https://github.com/oscartbeaumont/rspc/tree/main/docs",
  useNextSeoProps() {
    const { asPath } = useRouter();

    const description =
      "The best way to build typesafe APIs between Rust and Typescript.";

    return {
      titleTemplate: "%s - rspc",
      description,
      keywords: [],
      additionalMetaTags: [
        {
          name: "author",
          content: "Oscar Beaumont",
        },
      ],
      canonical: "https://rspc.dev" + asPath,
      openGraph: {
        images: [
          {
            url: "https://rspc.dev/api/og",
            width: 1200,
            height: 600,
            alt: "Og Image Alt",
            type: "image/jpeg",
          },
        ],
      },
      twitter: {
        handle: "@oscartbeaumont",
        site: "@site",
        cardType: "summary_large_image",
      },
    };
  },
  head: <></>, // Remove's the default OG tags
  logo: () => {
    const router = useRouter();
    return (
      <div
        onContextMenu={(e) => {
          e.preventDefault();
          router.push("/brand");
        }}
        className="flex place-items-center"
      >
        <Image src={logoIcon} alt="rspc logo" width={45} height={45} />
        <h1 className="pl-3 tracking-tighter font-sans text-2xl">rspc</h1>
      </div>
    );
  },
  navbar: {
    extraContent: (
      <>
        <NavbarIcon
          title="Crates.io"
          url="https://crates.io/crates/rspc"
          iconViewBox="0 0 512 512"
          icon={
            <path d="M239.1 6.3l-208 78c-18.7 7-31.1 25-31.1 45v225.1c0 18.2 10.3 34.8 26.5 42.9l208 104c13.5 6.8 29.4 6.8 42.9 0l208-104c16.3-8.1 26.5-24.8 26.5-42.9V129.3c0-20-12.4-37.9-31.1-44.9l-208-78C262 2.2 250 2.2 239.1 6.3zM256 68.4l192 72v1.1l-192 78-192-78v-1.1l192-72zm32 356V275.5l160-65v133.9l-160 80z"></path>
          }
        />
        <NavbarIcon
          title="NPM"
          url="https://www.npmjs.com/org/rspc"
          iconViewBox="0 0 256 256"
          icon={
            <>
              <path d="M0 256V0h256v256z" fill="#C12127"></path>
              <path d="M48 48h160v160h-32V80h-48v128H48z" fill="#FFF"></path>
            </>
          }
        />
        <NavbarIcon
          title="GitHub"
          url="https://github.com/oscartbeaumont/rspc"
          iconViewBox="3 3 18 18"
          icon={
            <path d="M12 3C7.0275 3 3 7.12937 3 12.2276C3 16.3109 5.57625 19.7597 9.15374 20.9824C9.60374 21.0631 9.77249 20.7863 9.77249 20.5441C9.77249 20.3249 9.76125 19.5982 9.76125 18.8254C7.5 19.2522 6.915 18.2602 6.735 17.7412C6.63375 17.4759 6.19499 16.6569 5.8125 16.4378C5.4975 16.2647 5.0475 15.838 5.80124 15.8264C6.51 15.8149 7.01625 16.4954 7.18499 16.7723C7.99499 18.1679 9.28875 17.7758 9.80625 17.5335C9.885 16.9337 10.1212 16.53 10.38 16.2993C8.3775 16.0687 6.285 15.2728 6.285 11.7432C6.285 10.7397 6.63375 9.9092 7.20749 9.26326C7.1175 9.03257 6.8025 8.08674 7.2975 6.81794C7.2975 6.81794 8.05125 6.57571 9.77249 7.76377C10.4925 7.55615 11.2575 7.45234 12.0225 7.45234C12.7875 7.45234 13.5525 7.55615 14.2725 7.76377C15.9937 6.56418 16.7475 6.81794 16.7475 6.81794C17.2424 8.08674 16.9275 9.03257 16.8375 9.26326C17.4113 9.9092 17.76 10.7281 17.76 11.7432C17.76 15.2843 15.6563 16.0687 13.6537 16.2993C13.98 16.5877 14.2613 17.1414 14.2613 18.0065C14.2613 19.2407 14.25 20.2326 14.25 20.5441C14.25 20.7863 14.4188 21.0746 14.8688 20.9824C16.6554 20.364 18.2079 19.1866 19.3078 17.6162C20.4077 16.0457 20.9995 14.1611 21 12.2276C21 7.12937 16.9725 3 12 3Z"></path>
          }
        />
        <NavbarIcon
          title="Discord"
          url="https://discord.gg/JgqH8b4ycw"
          iconViewBox="0 5 30.67 23.25"
          icon={
            <path d="M26.0015 6.9529C24.0021 6.03845 21.8787 5.37198 19.6623 5C19.3833 5.48048 19.0733 6.13144 18.8563 6.64292C16.4989 6.30193 14.1585 6.30193 11.8336 6.64292C11.6166 6.13144 11.2911 5.48048 11.0276 5C8.79575 5.37198 6.67235 6.03845 4.6869 6.9529C0.672601 12.8736 -0.41235 18.6548 0.130124 24.3585C2.79599 26.2959 5.36889 27.4739 7.89682 28.2489C8.51679 27.4119 9.07477 26.5129 9.55525 25.5675C8.64079 25.2265 7.77283 24.808 6.93587 24.312C7.15286 24.1571 7.36986 23.9866 7.57135 23.8161C12.6241 26.1255 18.0969 26.1255 23.0876 23.8161C23.3046 23.9866 23.5061 24.1571 23.7231 24.312C22.8861 24.808 22.0182 25.2265 21.1037 25.5675C21.5842 26.5129 22.1422 27.4119 22.7621 28.2489C25.2885 27.4739 27.8769 26.2959 30.5288 24.3585C31.1952 17.7559 29.4733 12.0212 26.0015 6.9529ZM10.2527 20.8402C8.73376 20.8402 7.49382 19.4608 7.49382 17.7714C7.49382 16.082 8.70276 14.7025 10.2527 14.7025C11.7871 14.7025 13.0425 16.082 13.0115 17.7714C13.0115 19.4608 11.7871 20.8402 10.2527 20.8402ZM20.4373 20.8402C18.9183 20.8402 17.6768 19.4608 17.6768 17.7714C17.6768 16.082 18.8873 14.7025 20.4373 14.7025C21.9717 14.7025 23.2271 16.082 23.1961 17.7714C23.1961 19.4608 21.9872 20.8402 20.4373 20.8402Z"></path>
          }
        />
      </>
    ),
  },
  // DO NOT REMOVE or search will be broken. This is a workaround for https://github.com/shuding/nextra/issues/1213
  search: {
    loading: "Loading...",
  },
  footer: {
    component: <></>,
  },
  sidebar: {
    titleComponent({ title, type }) {
      if (type === "separator" && title === "switchers") {
        return <Switchers />;
      }

      return <>{title}</>;
    },
  },
};

function NavbarIcon(props: {
  title: string;
  url: string;
  iconViewBox: string;
  icon: React.ReactNode;
}) {
  return (
    <a
      href={props.url}
      target="_blank"
      rel="noreferrer"
      className="nx-p-2 nx-text-current"
    >
      <svg
        width="24"
        height="24"
        fill="currentColor"
        viewBox={props.iconViewBox}
      >
        <title>{props.title}</title>
        {props.icon}
      </svg>
      <span className="nx-sr-only">{props.title}</span>
      <span className="nx-sr-only"> (opens in a new tab)</span>
    </a>
  );
}

export default config;
