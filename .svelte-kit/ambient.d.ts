
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * Environment variables [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env`. Like [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), this module cannot be imported into client-side code. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * _Unlike_ [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), the values exported from this module are statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * ```ts
 * import { API_KEY } from '$env/static/private';
 * ```
 * 
 * Note that all environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * 
 * ```
 * MY_FEATURE_FLAG=""
 * ```
 * 
 * You can override `.env` values from the command line like so:
 * 
 * ```sh
 * MY_FEATURE_FLAG="enabled" npm run dev
 * ```
 */
declare module '$env/static/private' {
	export const TAURI_ENV_PLATFORM: string;
	export const USER: string;
	export const npm_config_user_agent: string;
	export const TAURI_CLI_VERBOSITY: string;
	export const HOSTNAME: string;
	export const npm_node_execpath: string;
	export const npm_config_noproxy: string;
	export const HOME: string;
	export const npm_package_json: string;
	export const GTK_MODULES: string;
	export const GDK_DISABLE_MITSHM: string;
	export const npm_config_local_prefix: string;
	export const npm_config_userconfig: string;
	export const GDK_SCALE: string;
	export const TAURI_ENV_TARGET_TRIPLE: string;
	export const npm_config_engine_strict: string;
	export const COLOR: string;
	export const GDK_USE_X11: string;
	export const npm_config_npm_version: string;
	export const npm_config_prefix: string;
	export const TERM: string;
	export const npm_config_cache: string;
	export const RUSTUP_HOME: string;
	export const TAURI_ENV_DEBUG: string;
	export const QT_X11_NO_MITSHM: string;
	export const npm_config_node_gyp: string;
	export const PATH: string;
	export const NODE: string;
	export const TAURI_ENV_PLATFORM_VERSION: string;
	export const npm_package_name: string;
	export const GDK_BACKEND: string;
	export const DISPLAY: string;
	export const TAURI_ENV_ARCH: string;
	export const npm_lifecycle_script: string;
	export const npm_lifecycle_event: string;
	export const npm_package_version: string;
	export const NO_AT_BRIDGE: string;
	export const RUST_VERSION: string;
	export const TAURI_ENV_FAMILY: string;
	export const npm_config_globalconfig: string;
	export const npm_config_init_module: string;
	export const PWD: string;
	export const npm_execpath: string;
	export const CARGO_HOME: string;
	export const npm_config_global_prefix: string;
	export const npm_command: string;
	export const EDITOR: string;
	export const INIT_CWD: string;
	export const NODE_ENV: string;
}

/**
 * Similar to [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private), except that it only includes environment variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Values are replaced statically at build time.
 * 
 * ```ts
 * import { PUBLIC_BASE_URL } from '$env/static/public';
 * ```
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to runtime environment variables, as defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * This module cannot be imported into client-side code.
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * console.log(env.DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` always includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 */
declare module '$env/dynamic/private' {
	export const env: {
		TAURI_ENV_PLATFORM: string;
		USER: string;
		npm_config_user_agent: string;
		TAURI_CLI_VERBOSITY: string;
		HOSTNAME: string;
		npm_node_execpath: string;
		npm_config_noproxy: string;
		HOME: string;
		npm_package_json: string;
		GTK_MODULES: string;
		GDK_DISABLE_MITSHM: string;
		npm_config_local_prefix: string;
		npm_config_userconfig: string;
		GDK_SCALE: string;
		TAURI_ENV_TARGET_TRIPLE: string;
		npm_config_engine_strict: string;
		COLOR: string;
		GDK_USE_X11: string;
		npm_config_npm_version: string;
		npm_config_prefix: string;
		TERM: string;
		npm_config_cache: string;
		RUSTUP_HOME: string;
		TAURI_ENV_DEBUG: string;
		QT_X11_NO_MITSHM: string;
		npm_config_node_gyp: string;
		PATH: string;
		NODE: string;
		TAURI_ENV_PLATFORM_VERSION: string;
		npm_package_name: string;
		GDK_BACKEND: string;
		DISPLAY: string;
		TAURI_ENV_ARCH: string;
		npm_lifecycle_script: string;
		npm_lifecycle_event: string;
		npm_package_version: string;
		NO_AT_BRIDGE: string;
		RUST_VERSION: string;
		TAURI_ENV_FAMILY: string;
		npm_config_globalconfig: string;
		npm_config_init_module: string;
		PWD: string;
		npm_execpath: string;
		CARGO_HOME: string;
		npm_config_global_prefix: string;
		npm_command: string;
		EDITOR: string;
		INIT_CWD: string;
		NODE_ENV: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * Similar to [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), but only includes variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Note that public dynamic environment variables must all be sent from the server to the client, causing larger network requests — when possible, use `$env/static/public` instead.
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.PUBLIC_DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
