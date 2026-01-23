export interface ApiClient {
	id: number;
	client: string;
}

export interface GameMetadata {
	api?: ApiClient;
	store_pages?: string[];
	name?: string;
	icon?: string;
	logo?: string;
	hero?: string;
	cover?: string;
	description?: string;
	short_description?: string;
	screenshots?: string[];
	movies?: string[];
	movies_thumbnails?: string[];
	tags?: string[];
}

export interface GameLaunchData {
	proton?: string;
	winetricks?: string[];
	noruntime?: boolean;
	epicgame?: boolean;
	environs?: Record<string, string>;
	overlays: string[];
	start: string[];
	prestart?: string[];
}

export interface Game {
	slug: string;
	status: string;
	metadata: GameMetadata;
	launch: GameLaunchData;
}

export interface GameDisplay {
	slug: string;
	name?: string;
	icon?: string;
	logo?: string;
	hero?: string;
	cover?: string;
	description?: string;
	short_description?: string;
	screenshots?: string[];
	movies?: string[];
	movies_thumbnails?: string[];
	tags?: string[];
}

export type SortOrder = 'ascending' | 'descending';
