import type { Game, GameDisplay } from '$lib/types/game';
import { browser } from '$app/environment';
import { mockGames } from './mockData';

function getInvoke() {
	if (!browser) return null;
	return window.__TAURI__?.core?.invoke;
}

function isTauriAvailable(): boolean {
	return browser && typeof window.__TAURI__ !== 'undefined';
}

export async function fetchGameLibrary(): Promise<GameDisplay[]> {
	const invoke = getInvoke();

	// Use mock data when Tauri is not available (frontend-only dev)
	if (!invoke) {
		console.log('[DEV] Using mock game data');
		return mockGames;
	}

	let retries = 0;
	const maxRetries = 50;

	while (retries < maxRetries) {
		try {
			const library: Game[] = await invoke('get_game_library', {});
			if (library && library.length > 0) {
				// Transform Game to GameDisplay
				return library.map((game) => ({
					slug: game.slug,
					name: game.metadata.name,
					icon: game.metadata.icon,
					logo: game.metadata.logo,
					hero: game.metadata.hero,
					cover: game.metadata.cover,
					description: game.metadata.description,
					short_description: game.metadata.short_description,
					screenshots: game.metadata.screenshots,
					movies: game.metadata.movies,
					movies_thumbnails: game.metadata.movies_thumbnails,
					tags: game.metadata.tags
				}));
			}
		} catch (error) {
			console.error('Error fetching game library:', error);
		}

		retries++;
		await new Promise((resolve) => setTimeout(resolve, 100));
	}

	console.error('Failed to fetch game library after max retries');
	return [];
}

export async function launchGame(slug: string): Promise<boolean> {
	const invoke = getInvoke();

	if (!invoke) {
		console.log(`[DEV] Mock launch game: ${slug}`);
		return true;
	}

	try {
		await invoke('launch_game', { game: slug });
		return true;
	} catch (error) {
		console.error('Error launching game:', error);
		return false;
	}
}

export async function getRunningGame(): Promise<string> {
	const invoke = getInvoke();

	if (!invoke) {
		// No running game in mock mode
		return '';
	}

	try {
		return await invoke('get_running_game', {});
	} catch (error) {
		console.error('Error getting running game:', error);
		return '';
	}
}

export async function killRunningGame(): Promise<void> {
	const invoke = getInvoke();

	if (!invoke) {
		console.log('[DEV] Mock kill running game');
		return;
	}

	try {
		await invoke('kill_running_game', {});
	} catch (error) {
		console.error('Error killing running game:', error);
	}
}
