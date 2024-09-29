import { defineMiddleware } from "astro:middleware"
import { fetchRegistryJson, type PackageVersionResponse } from "./lib/registry-api"

export const onRequest = defineMiddleware(async (context, next) => {
	if (context.url.pathname.startsWith("/packages/")) {
		const { scope, name, version, target } = context.params
		const pkg = await fetchRegistryJson<PackageVersionResponse>(
			`packages/${encodeURIComponent(`${scope}/${name}`)}/${version}/${target}`,
		)

		context.locals.pkg = pkg
	}

	return next()
})
