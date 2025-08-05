const { invoke } = window.__TAURI__.core;
const { opener } = window.__TAURI__;

export async function addIcon() {
  try {
    const result = await invoke("add_launcher_desktop_icon");
    if (result != "") alert(result);
  } catch (err) {
    alert("Erreur lors de lajout de l'icon : " + err);
  }
}

export function openUrl(url) {
  opener.openUrl(url);
}
