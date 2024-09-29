import { unified } from "unified"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkGfm from "remark-gfm"
import remarkGemoji from "remark-gemoji"
import rehypeSanitize from "rehype-sanitize"
import rehypeStringify from "rehype-stringify"
import rehypeRaw from "rehype-raw"
import rehypeShiki from "@shikijs/rehype"
import { createCssVariablesTheme } from "shiki"
import remarkFrontmatter from "remark-frontmatter"
import { parent } from "hast-util-assert"
import { toString } from "hast-util-to-string"
import rehypeSlug from "rehype-slug"
import { h, s } from "hastscript"
import { map } from "unist-util-map"

export const markdown = unified()
	.use(remarkParse)
	.use(remarkFrontmatter)
	.use(remarkGfm)
	.use(remarkGemoji)
	.use(remarkRehype, { allowDangerousHtml: true })
	.use(rehypeRaw)
	.use(rehypeSanitize)
	.use(rehypeShiki, {
		theme: createCssVariablesTheme({
			name: "css-variables",
			variablePrefix: "--shiki-",
			variableDefaults: {},
			fontStyle: true,
		}),
		defaultLanguage: "text",
	})
	.use(() => (node, file) => {
		parent(node)

		const first = node.children[0]
		if (first.type === "element" && first.tagName === "h1") {
			const titleNode = node.children.shift()!
			file.data.title = toString(titleNode)
		}
	})
	.use(rehypeSlug)
	.use(() => (node, file) => {
		return map(node, (node) => {
			parent(node)

			if (node.type === "element") {
				if (
					node.tagName === "h1" ||
					node.tagName === "h2" ||
					node.tagName === "h3" ||
					node.tagName === "h4" ||
					node.tagName === "h5" ||
					node.tagName === "h6"
				) {
					return h()
				}
			}

			return node
		})
	})
	.use(rehypeStringify)
