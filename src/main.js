const { invoke } = window.__TAURI__.core;
const { shell } = window.__TAURI__;

async function addIcon() {
  try {
    const result = await invoke("add_launcher_desktop_icon");
    if (result != "") alert(result);
  } catch (err) {
    alert("Erreur lors de lajout de l'icon : " + err);
  }
}

async function launchGame(game) {
  let state = await invoke("get_gameprocess_state", {});
  try {
    const result = await invoke("launch_game", { game: game });
  } catch (err) {
    alert("Erreur lors du lancement : " + err);
  }
}

async function fetchGameLibrary() {
  try {
    const library = await invoke("get_game_library", {});
    //console.log("Game Library:", library);
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

async function processinfo_think() {
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  while (1) {
    await sleep(250);
    let state = await invoke("get_gameprocess_state", {});

    if (state) {
      // Game is running, highlight it
      document
        .querySelectorAll(".game-card")
        .forEach((el) => el.classList.remove("running"));
      document
        .querySelectorAll(".game-list-item")
        .forEach((el) => el.classList.remove("running"));

      let card = document.querySelector(`.game-card[game="${state}"]`);
      let listItem = document.querySelector(`.game-list-item[game="${state}"]`);
      let playButton = document.querySelector(`.play-button`);

      if (card) card.classList.add("running");
      if (listItem) listItem.classList.add("running");
      if (playButton) {
        playButton.textContent = "Kill";
        playButton.classList.add("kill-button");
      }
    } else {
      // No game is running
      document
        .querySelectorAll(".game-card")
        .forEach((el) => el.classList.remove("running"));
      document
        .querySelectorAll(".game-list-item")
        .forEach((el) => el.classList.remove("running"));
      let playButton = document.querySelector(`.play-button`);
      if (playButton) {
        playButton.textContent = "Play";
        playButton.classList.remove("kill-button");
      }
    }
  }
}

//Library and Game_preview section
async function hideGameCards() {
  const gamesSection = document.querySelector("#library");
  gamesSection.classList.add("hidden");
}

async function showGameCards() {
  const gamesSection = document.querySelector("#library");
  gamesSection.classList.remove("hidden");
}

async function hideGameInfo() {
  const gamesSection = document.querySelector("#game-preview-container");
  gamesSection.classList.add("hidden");
}

async function showGameInfo() {
  const gamesSection = document.querySelector("#game-preview-container");
  gamesSection.classList.remove("hidden");
}

function extractImageUrls(imageString) {
  if (!imageString) return [];
  const urls = imageString.split(",");
  return urls[0];
}

//DISPLAY
async function displayGamePreview(game, data) {
  const gamesSection = document.querySelector("#game-preview-container");

  if (gamesSection == null) {
    document.querySelector("#main-page").insertAdjacentHTML(
      "beforeend",
      `<div id="game-preview-container" class="page">
          <div class="game-preview" game="asd">
            <div class="image-crop-container">
              <div class="game-preview-artwork"></div>
              <h1 class="title-overlay">Game Title</h1>
                <div class="button-overlay">
                  <button class="play-button">Play</button>
                  <button class="game-settings-button">Game Settings</button>
                </div>
              </div>
          </div>
          <h2>Summary</h2>
          <div class="game-description"></div>
          <h3>Genres</h3>
          <div class="game-genres"></div>
        </div>`
    );
    document.querySelector("#game-preview-container").classList.add("hidden");
  }
}

async function changeGamePreview(game, data) {
  const gameSection = document.querySelector("#game-preview-container");
  if (!gameSection) return;

  // Change title
  const titleElement = gameSection.querySelector(".title-overlay");
  titleElement.textContent = data.displayname;

  // Change image source
  const game_preview = document.querySelector(".game-preview");
  game_preview.setAttribute("game", data.name);
  const img = document.querySelector(".game-preview-artwork");

  if (data.artworks[0] == null) {
    img.style.backgroundImage = `url(${data.cover})`;
  } else {
    img.style.backgroundImage = `url(${data.artworks[0]})`;
  }

  //data.genres.forEach((genre) => {
  //  document.querySelector(".game-genres").innerHTML("afterbegin" ,`<div class="game-genres-item"></div>`);
  //  document.querySelector(".game-genres-item").textContent = genre;
  //});
  // Change description
  const descriptionElement = gameSection.querySelector(".game-description");
  if (data.summary) {
    descriptionElement.textContent = data.summary;
  }
  // Reveal the section (if needed)
  gameSection.classList.remove("hidden");
}

async function displayLibrary(game, data, running) {
  document.querySelector("#games").insertAdjacentHTML(
    "afterbegin",
    `<button class="game-card ${running}" id="${game.name}" game="${game.name}">
      <div style="background-image: url('${data.cover}');"></div>
    </button>`
  );
}

async function displayGameList(game, data, running) {
  document.querySelector("#game-list").insertAdjacentHTML(
    "afterbegin",
    `<li class="game-list-item ${running}" id="item_${game.name}" game="${game.name}">
      <img src="${data.icon}" alt="${game.name} icon" class="game-list-icon">
      ${data.displayname}
    </li>`
  );
}

function refreshDisplay(combined, running, game_click_handler) {
  // Clear the current display
  document.getElementById("games").innerHTML = "";
  document.getElementById("game-list").innerHTML = "";

  // Re-render the sorted list
  combined.forEach(({ game, data }) => {
    displayLibrary(game, data, running);
    displayGameList(game, data, running);
  });

  // Re-attach the click handler
  document.querySelectorAll(".game-card").forEach(game_click_handler);
  document.querySelectorAll(".game-list-item").forEach(game_click_handler);
}

//SORTING
async function sortGames(combined, order) {
  if (order === "descending") {
    combined.sort((a, b) =>
      b.data.displayname.localeCompare(a.data.displayname, undefined, {
        sensitivity: "base",
      })
    );
  } else if (order === "ascending") {
    combined.sort((a, b) =>
      a.data.displayname.localeCompare(b.data.displayname, undefined, {
        sensitivity: "base",
      })
    );
  }
}

//async function handleSortBy(combined, running, game_click_handler) {
//  document
//    .getElementById("sort-select")
//    .addEventListener("change", async function () {
//      const selectedOrder = this.value;

//      await sortGames(combined, selectedOrder);

//      refreshDisplay(combined, running, game_click_handler);
//    });
//}

//CUSTOM DROPDOWN
document.getElementById("dropdown-button").addEventListener("click", () => {
  document.getElementById("dropdown-menu").classList.toggle("hidden");
});

window.addEventListener("DOMContentLoaded", () => {
  fetchGameLibrary().then((library) => {
    let running = "";
    displayGamePreview(null, null);

    // Zip games and gamesdata together
    const combined = library.games.map((game, i) => ({
      game: game,
      data: library.gamesdata[i],
    }));

    sortGames(combined, "descending");
    // SORT SYSTEM
    document.querySelectorAll("#dropdown-menu li").forEach((item) => {
      item.addEventListener("click", async () => {
        const selectedOrder = item.getAttribute("data-value");

        await sortGames(combined, selectedOrder);

        refreshDisplay(combined, running, game_click_handler);

        document.getElementById("dropdown-menu").classList.add("hidden");
      });
    });

    combined.forEach(({ game, data }) => {
      data.name = game.name;
      displayLibrary(game, data, running);
      displayGameList(game, data, running);
    });

    //GAME PREVIEW
    function game_click_handler(card) {
      card.addEventListener("click", function () {
        const game = this.getAttribute("game");

        hideGameCards();
        showGameInfo();

        let data = null;

        // Find the correct game data
        for (let i = 0; i < library.gamesdata.length; i++) {
          if (library.gamesdata[i].name === game) {
            data = library.gamesdata[i];
            break;
          }
        }
        if (data) {
          changeGamePreview(game, data);
        } else {
          console.error("Game data not found for:", game);
        }
      });
    }

    //handleSortBy(combined, running, game_click_handler);

    //////////////////ONCLICK////////////////////////

    ////////////TOPBAR/////////////
    //addicon
    let addicon_button = document.getElementById("addicon-button");
    addicon_button.addEventListener("click", function () {
      addIcon();
    });
    //library
    let lib_button = document.getElementById("library-button");
    lib_button.addEventListener("click", function () {
      hideGameInfo();
      showGameCards();
    });
    //aboutus

    let aboutus_button = document.getElementById("aboutus-button");
    aboutus_button.addEventListener("click", function () {
      console.log(window.__TAURI__);
      window.__TAURI__.opener.openUrl("https://discord.gg/YR7fwGy5D7");
    });
    //settings
    let settings_button = document.getElementById("settings-button");
    settings_button.addEventListener("click", function () {
      openSettings();
    });

    ////////////GAMECARD//////////////

    document.querySelectorAll(".game-card").forEach(game_click_handler);
    document.querySelectorAll(".game-list-item").forEach(game_click_handler);

    let playbutton = document.querySelector(".play-button");
    playbutton.addEventListener("click", function () {
      const game = document.querySelector(".game-preview").getAttribute("game");
      launchGame(game);
    });
  });
  setup_progressbar();
  processinfo_think();
});
