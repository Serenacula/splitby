// @ts-check
import { defineConfig } from "astro/config"
import starlight from "@astrojs/starlight"

// https://astro.build/config
export default defineConfig({
    site: "https://serenacula.github.io/splitby/",
    integrations: [
        starlight({
            // plugins: [starlightAutoSidebar()],
            title: "splitby",
            customCss: ["./src/styles/custom.css"],
            sidebar: [
                {
                    label: "Start Here",
                    items: [
                        // { label: "Home", link: "/" },
                        { label: "Install", link: "/install" },
                        { label: "Basics", link: "/basics" },
                        { label: "Modes", link: "/modes" },
                    ],
                },
                {
                    label: "Flags",
                    items: [
                        { label: "Overview", link: "/flags/" },
                        {
                            label: "File Flags",
                            items: [
                                {
                                    label: "Input",
                                    link: "/flags/input",
                                },
                                {
                                    label: "Output",
                                    link: "/flags/output",
                                },
                            ],
                        },
                        {
                            label: "Input Modes",
                            items: [
                                {
                                    label: "Per line",
                                    link: "/flags/per-line",
                                },
                                {
                                    label: "Whole string",
                                    link: "/flags/whole-string",
                                },
                                {
                                    label: "Zero terminated",
                                    link: "/flags/zero-terminated",
                                },
                            ],
                        },
                        {
                            label: "Selection Modes",
                            items: [
                                {
                                    label: "Fields",
                                    link: "/flags/fields",
                                },
                                {
                                    label: "Bytes",
                                    link: "/flags/bytes",
                                },
                                {
                                    label: "Characters",
                                    link: "/flags/characters",
                                },
                            ],
                        },
                        {
                            label: "Selection Flags",
                            items: [
                                {
                                    label: "Delimiter",
                                    link: "/flags/delimiter",
                                },
                                {
                                    label: "Invert",
                                    link: "/flags/invert",
                                },
                                {
                                    label: "Skip empty",
                                    link: "/flags/skip-empty",
                                },
                            ],
                        },
                        {
                            label: "Transform Flags",
                            items: [
                                {
                                    label: "Align",
                                    link: "/flags/align",
                                },
                                {
                                    label: "Count",
                                    link: "/flags/count",
                                },
                                {
                                    label: "Join",
                                    link: "/flags/join",
                                },
                                {
                                    label: "Placeholder",
                                    link: "/flags/placeholder",
                                },
                            ],
                        },
                        {
                            label: "Strict Flags",
                            items: [
                                {
                                    label: "Strict",
                                    link: "/flags/strict",
                                },
                                {
                                    label: "Strict bounds",
                                    link: "/flags/strict-bounds",
                                },
                                {
                                    label: "Strict return",
                                    link: "/flags/strict-return",
                                },
                                {
                                    label: "Strict range order",
                                    link: "/flags/strict-range-order",
                                },
                                {
                                    label: "Strict UTF-8",
                                    link: "/flags/strict-utf8",
                                },
                            ],
                        },
                    ],
                },
                {
                    label: "FAQ",
                    items: [{ label: "FAQ", link: "/faq" }],
                },
            ],
        }),
    ],
})
