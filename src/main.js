const { invoke } = window.__TAURI__.core;

async function addIcon() {
  try {
    const result = await invoke("add_launcher_desktop_icon");
    if (result != "") alert(result);
  } catch (err) {
    alert("Erreur lors de lajout de l'icon : " + err);
  }
}

async function launchGame(game) {
  try {
    const result = await invoke("launch_game", { game: game });
  } catch (err) {
    alert("Erreur lors du lancement : " + err);
  }
}

async function fetchGameLibrary() {
  try {
    const library = await invoke("get_game_library", {});
    console.log("Game Library:", library);
    return library;
  } catch (error) {
    console.error("Failed to fetch game library:", error);
  }
}

async function setup_progressbar() {
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const progressBar = document.getElementById("progress-bar");
  progressBar.style.width = "0%";
  progressBar.style.transition = "none";
  progressBar.style.display = "block";
  progressBar.style.transition = "width 0.5s ease-in-out";

  console.log("Setting up progress bar...");
  let percent = 0;
  let fake_percent = 0;
  while (percent < 100) {
    await sleep(50);
    let state = await invoke("get_setup_state", {});
    fake_percent =
      Math.min(fake_percent + Math.random() * 0.3, 100 - 5) /
      (state.progress || 50);
    percent = state.progress + fake_percent;
    progressBar.style.width = `${percent}%`;
  }
  percent = 100;
  progressBar.style.width = `${percent}%`;
  //for (let i = 0; i <= 100; i++) {
  //	await sleep(50);
  //	progressBar.style.width = `${i}%`;
  //}
}

window.addEventListener("DOMContentLoaded", () => {
  fetchGameLibrary().then((library) => {
    let i = 0;

    library.games.forEach((game) => {
      let data = library.gamesdata[i];

      let running = "";
      if (game.name == "cs2") running = "running";
      document.querySelector("#games").insertAdjacentHTML(
        "afterbegin",
        `<button class="game-card ${running}" id="${game.name}" game="${game.name}">
					<div style="background-image: url('${data.cover}');"></div>
				</button>`
      );
      document.querySelector("#game-list").insertAdjacentHTML(
        "afterbegin",
        `<li class="game-list-item ${running}" id="${game.name}">
					${game.name}
				</li>`
      );
      i++;
    });
    document.querySelectorAll(".game-card").forEach((card) => {
      card.addEventListener("click", function () {
        const game = this.getAttribute("game");
        launchGame(game);
      });
    });
  });
  document.getElementById("add_icon").addEventListener("click", addIcon);
  setup_progressbar();
});
