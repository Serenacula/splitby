// @ts-check
import { defineConfig } from "astro/config"
import starlight from "@astrojs/starlight"
import starlightAutoSidebar from "starlight-auto-sidebar"

// https://astro.build/config
export default defineConfig({
    integrations: [
        starlight({
            // plugins: [starlightAutoSidebar()],
            title: "splitby",
            social: [
                {
                    icon: "github",
                    label: "GitHub",
                    href: "https://github.com/Serenacula/splitby",
                },
            ],
            customCss: ["./src/styles/custom.css"],
            sidebar: [
                {
                    label: "Start Here",
                    items: [
                        { label: "Home", link: "/" },
                        { label: "Install", link: "/install/" },
                        { label: "Basics", link: "/basics/" },
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
                                    link: "/flags/file_flags/input",
                                },
                                {
                                    label: "Output",
                                    link: "/flags/file_flags/output",
                                },
                            ],
                        },
                        {
                            label: "Input Modes",
                            items: [
                                {
                                    label: "Per line",
                                    link: "/flags/input_modes/per-line",
                                },
                                {
                                    label: "Whole string",
                                    link: "/flags/input_modes/whole-string",
                                },
                                {
                                    label: "Zero terminated",
                                    link: "/flags/input_modes/zero-terminated",
                                },
                            ],
                        },
                        {
                            label: "Selection Modes",
                            items: [
                                {
                                    label: "Fields",
                                    link: "/flags/selection_modes/fields",
                                },
                                {
                                    label: "Bytes",
                                    link: "/flags/selection_modes/bytes",
                                },
                                {
                                    label: "Characters",
                                    link: "/flags/selection_modes/characters",
                                },
                            ],
                        },
                        {
                            label: "Selection Flags",
                            items: [
                                {
                                    label: "Delimiter",
                                    link: "/flags/selection_flags/delimiter",
                                },
                                {
                                    label: "Invert",
                                    link: "/flags/selection_flags/invert",
                                },
                                {
                                    label: "Skip empty",
                                    link: "/flags/selection_flags/skip-empty",
                                },
                            ],
                        },
                        {
                            label: "Transform Flags",
                            items: [
                                {
                                    label: "Align",
                                    link: "/flags/transform_flags/align",
                                },
                                {
                                    label: "Count",
                                    link: "/flags/transform_flags/count",
                                },
                                {
                                    label: "Join",
                                    link: "/flags/transform_flags/join",
                                },
                                {
                                    label: "Placeholder",
                                    link: "/flags/transform_flags/placeholder",
                                },
                            ],
                        },
                        {
                            label: "Strict Flags",
                            items: [
                                {
                                    label: "Strict",
                                    link: "/flags/strict_flags/strict",
                                },
                                {
                                    label: "Strict bounds",
                                    link: "/flags/strict_flags/strict-bounds",
                                },
                                {
                                    label: "Strict return",
                                    link: "/flags/strict_flags/strict-return",
                                },
                                {
                                    label: "Strict range order",
                                    link: "/flags/strict_flags/strict-range-order",
                                },
                                {
                                    label: "Strict UTF-8",
                                    link: "/flags/strict_flags/strict-utf8",
                                },
                            ],
                        },
                    ],
                },
                {
                    label: "FAQ",
                    items: [{ label: "FAQ", link: "/faq/" }],
                },
            ],
        }),
    ],
})
