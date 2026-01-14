// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'splitby',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/Serenacula/splitby' },
			],
			customCss: ['./src/styles/custom.css'],
			sidebar: [
				{
					label: 'Start Here',
					items: [
						{ label: 'Home', link: '/' },
						{ label: 'Install', link: '/install/' },
						{ label: 'Basics', link: '/basics/' },
					],
				},
				{
					label: 'Modes',
					items: [{ label: 'Modes', link: '/modes/' }],
				},
				{
					label: 'Flags',
					items: [
						{ label: 'Overview', link: '/flags/' },
						{ label: 'Delimiter (-d, --delimiter)', link: '/flags/delimiter/' },
						{ label: 'Join (-j, --join)', link: '/flags/join/' },
						{ label: 'Skip empty (-e, -E)', link: '/flags/skip-empty/' },
						{ label: 'Invert (--invert)', link: '/flags/invert/' },
						{ label: 'Count (--count)', link: '/flags/count/' },
						{ label: 'Placeholder (--placeholder)', link: '/flags/placeholder/' },
						{ label: 'Trim newline (--trim-newline)', link: '/flags/trim-newline/' },
						{ label: 'Strict (--strict, --no-strict)', link: '/flags/strict/' },
						{ label: 'Strict bounds (--strict-bounds)', link: '/flags/strict-bounds/' },
						{ label: 'Strict return (--strict-return)', link: '/flags/strict-return/' },
						{
							label: 'Strict range order (--strict-range-order)',
							link: '/flags/strict-range-order/',
						},
						{ label: 'Strict UTF-8 (--strict-utf8)', link: '/flags/strict-utf8/' },
						{ label: 'Per-line (--per-line)', link: '/flags/per-line/' },
						{ label: 'Whole string (-w, --whole-string)', link: '/flags/whole-string/' },
						{ label: 'Zero terminated (-z, --zero-terminated)', link: '/flags/zero-terminated/' },
						{ label: 'Input file (-i, --input)', link: '/flags/input/' },
						{ label: 'Output file (-o, --output)', link: '/flags/output/' },
						{ label: 'Fields (-f, --fields)', link: '/flags/fields/' },
						{ label: 'Bytes (-b, --bytes)', link: '/flags/bytes/' },
						{ label: 'Characters (-c, --characters)', link: '/flags/characters/' },
					],
				},
				{
					label: 'FAQ',
					items: [{ label: 'FAQ', link: '/faq/' }],
				},
			],
		}),
	],
});
