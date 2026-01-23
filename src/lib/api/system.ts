import { browser } from '$app/environment';

function getInvoke() {
	if (!browser) return null;
	return window.__TAURI__?.core?.invoke;
}

export async function addDesktopIcon(): Promise<void> {
	const invoke = getInvoke();
	if (!invoke) return;

	try {
		await invoke('add_launcher_to_desktop', {});
	} catch (error) {
		console.error('Error adding desktop icon:', error);
	}
}

export function openUrl(url: string): void {
	if (!browser) return;
	// Simple approach - window.open works in Tauri
	window.open(url, '_blank');
}
