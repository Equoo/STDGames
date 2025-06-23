const { invoke } = window.__TAURI__.core;

export async function launchGame(game) {
  let state = await invoke("get_gameprocess_state", {});
  try {
    const result = await invoke("launch_game", { game: game });
  } catch (err) {
    alert("Erreur lors du lancement : " + err);
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

export async function getGameProcessState() {
  return await invoke("get_gameprocess_state", {});
}

export async function monitorGameProcess() {
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  while (1) {
    await sleep(250);
    let state = await getGameProcessState();
    return state;
  }
}