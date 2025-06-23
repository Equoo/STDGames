const { invoke } = window.__TAURI__.core;
const { shell } = window.__TAURI__;

export async function addIcon() {
  try {
    const result = await invoke("add_launcher_desktop_icon");
    if (result != "") alert(result);
  } catch (err) {
    alert("Erreur lors de lajout de l'icon : " + err);
  }
}

export function openUrl(url) {
  shell.open(url);
}