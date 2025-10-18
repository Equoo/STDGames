const { invoke } = window.__TAURI__.core;

export async function launchGame(game) {
  console.log(`Attempting to launch game: ${game}`);
  try {
    const result = await invoke("launch_game", { game: game });
    console.log(`Game launched successfully: ${result}`);
    return true;
  } catch (err) {
    alert("Erreur lors du lancement : " + err);
    return false;
  }
}

export async function fetchGameLibrary() {
  try {
    const library = await invoke("get_game_library", {});
    return library;
  } catch (error) {
    console.error("Failed to fetch game library:", error);
  }
}
