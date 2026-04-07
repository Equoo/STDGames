import { writable, derived } from 'svelte/store';
import type { GameDisplay, SortOrder } from '$lib/types/game';

// Game library store
export const gameLibrary = writable<GameDisplay[]>([]);

// Currently running game slug
export const runningGame = writable<string>('');

// Currently selected game for preview
export const selectedGame = writable<GameDisplay | null>(null);

// Current view
export const currentView = writable<'home' | 'library' | 'preview' | 'settings'>('home');

// Theme store with localStorage persistence
export type Theme = 'dark' | 'light';
function createThemeStore() {
	const stored = (typeof localStorage !== 'undefined' ? localStorage.getItem('theme') : null) as Theme | null;
	const { subscribe, set } = writable<Theme>(stored ?? 'dark');
	return {
		subscribe,
		set: (value: Theme) => {
			if (typeof localStorage !== 'undefined') localStorage.setItem('theme', value);
			if (typeof document !== 'undefined') document.documentElement.setAttribute('data-theme', value);
			set(value);
		}
	};
}
export const theme = createThemeStore();

// Favorites store with localStorage persistence
function createFavoritesStore() {
	const stored = (typeof localStorage !== 'undefined' ? localStorage.getItem('favorites') : null);
	const initial: string[] = stored ? JSON.parse(stored) : [];
	const { subscribe, update } = writable<string[]>(initial);
	return {
		subscribe,
		toggle: (slug: string) => update(slugs => {
			const next = slugs.includes(slug) ? slugs.filter(s => s !== slug) : [...slugs, slug];
			if (typeof localStorage !== 'undefined') localStorage.setItem('favorites', JSON.stringify(next));
			return next;
		}),
		isFavorite: (slugs: string[], slug: string) => slugs.includes(slug),
	};
}
export const favorites = createFavoritesStore();

// Search query
export const searchQuery = writable<string>('');

// Sort order
export const sortOrder = writable<SortOrder>('descending');

// Active tag filter
export const activeTag = writable<string | null>(null);

// Filtered and sorted games
export const filteredGames = derived(
	[gameLibrary, searchQuery, sortOrder, activeTag],
	([$gameLibrary, $searchQuery, $sortOrder, $activeTag]) => {
		let games = [...$gameLibrary];

		// Filter by search query
		if ($searchQuery) {
			const query = $searchQuery.toLowerCase();
			games = games.filter((game) => game.name?.toLowerCase().includes(query));
		}

		// Filter by tag
		if ($activeTag) {
			games = games.filter((game) => game.tags?.includes($activeTag));
		}

		// Sort by name
		games.sort((a, b) => {
			const nameA = a.name?.toLowerCase() || '';
			const nameB = b.name?.toLowerCase() || '';
			if ($sortOrder === 'descending') {
				return nameA.localeCompare(nameB);
			} else {
				return nameB.localeCompare(nameA);
			}
		});

		return games;
	}
);
