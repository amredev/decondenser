import { defineConfig, HeadConfig } from "vitepress";
import vite from "./vite.config";

const head: HeadConfig[] = [
    ["link", { rel: "icon", href: `/decondenser-logo-thumb.png` }],
    ["meta", { property: "og:image", content: `/decondenser-logo-thumb.png` }],
];

const srcDir = "src";

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "Decondenser",
    description: "Prettify condensed text based on bracket placement",

    cleanUrls: true,
    lastUpdated: true,

    markdown: {
        theme: {
            dark: "dark-plus",
            light: "light-plus",
        },
    },

    srcExclude: ["README.md"],

    head,
    vite,
    srcDir,

    // https://vitepress.dev/reference/default-theme-config
    themeConfig: {
        logo: "/decondenser-logo-thumb.png",

        lastUpdated: {
            formatOptions: {
                dateStyle: "long",
                timeStyle: undefined,
                forceLocale: false,
            },
        },

        editLink: {
            pattern: `https://github.com/amredev/decondenser/edit/master/website/${srcDir}/:path`,
            text: "Edit this page on GitHub",
        },

        // Enable the search only in the final build on CI. Locally, it takes additional
        // time during the dev HMR server startup and config reloads.
        search: !process.env.CI
            ? undefined
            : {
                  provider: "local",
              },

        nav: [
            { text: "Playground", link: "/playground" },
            { text: "Changelog", link: "/changelog" },
            { text: "Blog", link: "/blog" },
        ],

        socialLinks: [
            { icon: "github", link: "https://github.com/amredev/decondenser" },
            { icon: "discord", link: "https://decondenser.dev/discord" },
        ],

        sidebar: {},
    },
});
