
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
	export const LANGUAGE: string;
	export const USER: string;
	export const npm_config_user_agent: string;
	export const TAURI_CLI_VERBOSITY: string;
	export const HOSTNAME: string;
	export const XDG_SESSION_TYPE: string;
	export const npm_node_execpath: string;
	export const LD_LIBRARY_PATH: string;
	export const SHLVL: string;
	export const XDG_CACHE_HOME: string;
	export const npm_config_noproxy: string;
	export const HOME: string;
	export const LESS: string;
	export const OLDPWD: string;
	export const DESKTOP_SESSION: string;
	export const npm_package_json: string;
	export const LSCOLORS: string;
	export const ZSH: string;
	export const GTK_MODULES: string;
	export const PAGER: string;
	export const XDG_SEAT_PATH: string;
	export const GDK_DISABLE_MITSHM: string;
	export const npm_config_local_prefix: string;
	export const npm_config_userconfig: string;
	export const DBUS_SESSION_BUS_ADDRESS: string;
	export const GDK_SCALE: string;
	export const SYSTEMD_EXEC_PID: string;
	export const TAURI_ENV_TARGET_TRIPLE: string;
	export const COLOR: string;
	export const COLORTERM: string;
	export const LIBVIRT_DEFAULT_URI: string;
	export const GDK_USE_X11: string;
	export const GTK_IM_MODULE: string;
	export const LOGNAME: string;
	export const _: string;
	export const npm_config_npm_version: string;
	export const npm_config_prefix: string;
	export const XDG_SESSION_CLASS: string;
	export const TERM: string;
	export const npm_config_cache: string;
	export const FT_HOOK_PATHNAME: string;
	export const GNOME_DESKTOP_SESSION_ID: string;
	export const RUSTUP_HOME: string;
	export const TAURI_ENV_DEBUG: string;
	export const QT_X11_NO_MITSHM: string;
	export const npm_config_node_gyp: string;
	export const PATH: string;
	export const GDM_LANG: string;
	export const NODE: string;
	export const SESSION_MANAGER: string;
	export const TAURI_ENV_PLATFORM_VERSION: string;
	export const npm_package_name: string;
	export const GDK_BACKEND: string;
	export const GNOME_TERMINAL_SCREEN: string;
	export const XDG_MENU_PREFIX: string;
	export const XDG_RUNTIME_DIR: string;
	export const XDG_SESSION_PATH: string;
	export const DISPLAY: string;
	export const DOTNET_BUNDLE_EXTRACT_BASE_DIR: string;
	export const LANG: string;
	export const TAURI_ENV_ARCH: string;
	export const XDG_CURRENT_DESKTOP: string;
	export const GNOME_TERMINAL_SERVICE: string;
	export const LS_COLORS: string;
	export const XAUTHORITY: string;
	export const XDG_SESSION_DESKTOP: string;
	export const XMODIFIERS: string;
	export const npm_lifecycle_script: string;
	export const SSH_AGENT_LAUNCHER: string;
	export const SSH_AUTH_SOCK: string;
	export const XDG_GREETER_DATA_DIR: string;
	export const KRB5CCNAME: string;
	export const SHELL: string;
	export const npm_lifecycle_event: string;
	export const npm_package_version: string;
	export const GDMSESSION: string;
	export const NO_AT_BRIDGE: string;
	export const QT_ACCESSIBILITY: string;
	export const RUSTUP_INIT_SKIP_PATH_CHECK: string;
	export const RUST_VERSION: string;
	export const DOCKER_HOST: string;
	export const FT_HOOK_NAME: string;
	export const GPG_AGENT_INFO: string;
	export const QT_IM_MODULE: string;
	export const RUSTUP_METADATA_DIR: string;
	export const TAURI_ENV_FAMILY: string;
	export const npm_config_globalconfig: string;
	export const npm_config_init_module: string;
	export const PWD: string;
	export const npm_execpath: string;
	export const CARGO_HOME: string;
	export const XDG_CONFIG_DIRS: string;
	export const XDG_DATA_DIRS: string;
	export const npm_config_global_prefix: string;
	export const npm_command: string;
	export const VTE_VERSION: string;
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
		LANGUAGE: string;
		USER: string;
		npm_config_user_agent: string;
		TAURI_CLI_VERBOSITY: string;
		HOSTNAME: string;
		XDG_SESSION_TYPE: string;
		npm_node_execpath: string;
		LD_LIBRARY_PATH: string;
		SHLVL: string;
		XDG_CACHE_HOME: string;
		npm_config_noproxy: string;
		HOME: string;
		LESS: string;
		OLDPWD: string;
		DESKTOP_SESSION: string;
		npm_package_json: string;
		LSCOLORS: string;
		ZSH: string;
		GTK_MODULES: string;
		PAGER: string;
		XDG_SEAT_PATH: string;
		GDK_DISABLE_MITSHM: string;
		npm_config_local_prefix: string;
		npm_config_userconfig: string;
		DBUS_SESSION_BUS_ADDRESS: string;
		GDK_SCALE: string;
		SYSTEMD_EXEC_PID: string;
		TAURI_ENV_TARGET_TRIPLE: string;
		COLOR: string;
		COLORTERM: string;
		LIBVIRT_DEFAULT_URI: string;
		GDK_USE_X11: string;
		GTK_IM_MODULE: string;
		LOGNAME: string;
		_: string;
		npm_config_npm_version: string;
		npm_config_prefix: string;
		XDG_SESSION_CLASS: string;
		TERM: string;
		npm_config_cache: string;
		FT_HOOK_PATHNAME: string;
		GNOME_DESKTOP_SESSION_ID: string;
		RUSTUP_HOME: string;
		TAURI_ENV_DEBUG: string;
		QT_X11_NO_MITSHM: string;
		npm_config_node_gyp: string;
		PATH: string;
		GDM_LANG: string;
		NODE: string;
		SESSION_MANAGER: string;
		TAURI_ENV_PLATFORM_VERSION: string;
		npm_package_name: string;
		GDK_BACKEND: string;
		GNOME_TERMINAL_SCREEN: string;
		XDG_MENU_PREFIX: string;
		XDG_RUNTIME_DIR: string;
		XDG_SESSION_PATH: string;
		DISPLAY: string;
		DOTNET_BUNDLE_EXTRACT_BASE_DIR: string;
		LANG: string;
		TAURI_ENV_ARCH: string;
		XDG_CURRENT_DESKTOP: string;
		GNOME_TERMINAL_SERVICE: string;
		LS_COLORS: string;
		XAUTHORITY: string;
		XDG_SESSION_DESKTOP: string;
		XMODIFIERS: string;
		npm_lifecycle_script: string;
		SSH_AGENT_LAUNCHER: string;
		SSH_AUTH_SOCK: string;
		XDG_GREETER_DATA_DIR: string;
		KRB5CCNAME: string;
		SHELL: string;
		npm_lifecycle_event: string;
		npm_package_version: string;
		GDMSESSION: string;
		NO_AT_BRIDGE: string;
		QT_ACCESSIBILITY: string;
		RUSTUP_INIT_SKIP_PATH_CHECK: string;
		RUST_VERSION: string;
		DOCKER_HOST: string;
		FT_HOOK_NAME: string;
		GPG_AGENT_INFO: string;
		QT_IM_MODULE: string;
		RUSTUP_METADATA_DIR: string;
		TAURI_ENV_FAMILY: string;
		npm_config_globalconfig: string;
		npm_config_init_module: string;
		PWD: string;
		npm_execpath: string;
		CARGO_HOME: string;
		XDG_CONFIG_DIRS: string;
		XDG_DATA_DIRS: string;
		npm_config_global_prefix: string;
		npm_command: string;
		VTE_VERSION: string;
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
