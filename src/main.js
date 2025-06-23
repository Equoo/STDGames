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
    document.querySelector("#game-preview-container").classList.add("hidden");
  }
}

// Update the changeGamePreview function
async function changeGamePreview(game, data) {
  const gameSection = document.querySelector("#game-preview-container");
  if (!gameSection) return;

  // Update game attribute on preview container
  gameSection.querySelector(".game-preview").setAttribute("game", game.name);

  // Update artwork - fallback to cover if no artwork
  const artworkUrl = data.artworks?.[0] || data.cover || './resources/default-game.jpg';
  document.querySelector(".game-preview-artwork").style.backgroundImage = `url('${artworkUrl}')`;

  // Update title
  document.querySelector(".title-overlay").textContent = data.displayname || game.name;

  // Back button
  // Update the back button handler in changeGamePreview
  document.getElementById('back-to-library').addEventListener('click', () => {
    showGameCards();
    hideGameInfo();
    // Set library button as active when returning
    document.getElementById("library-button").classList.add("active");
  });

  // Update description
  const descElement = document.querySelector(".game-description");
  descElement.textContent = data.summary || "No description available";
  descElement.style.display = data.summary ? "block" : "none";

  // Update genres
  const genresContainer = document.querySelector(".game-genres");
  genresContainer.innerHTML = "";

  if (data.genres && data.genres.length > 0) {
    data.genres.forEach(genre => {
      const genreElement = document.createElement("div");
      genreElement.className = "game-genres-item";
      genreElement.textContent = genre;
      genresContainer.appendChild(genreElement);
    });
  }

  // Update play button
  const playButton = document.querySelector(".play-button");
  playButton.setAttribute("data-game", game.name);
  playButton.onclick = () => launchGame(game.name);

  // Update screenshots
  const screenshotsContainer = document.querySelector(".screenshots-container");
  screenshotsContainer.innerHTML = "";

  if (data.screenshots && data.screenshots.length > 0) {
    const screenshotsTitle = document.createElement("h3");
    screenshotsContainer.appendChild(screenshotsTitle);

    const grid = document.createElement("div");
    grid.className = "screenshots-grid";

    data.screenshots.forEach(url => {
      const img = document.createElement("img");
      img.src = url;
      img.className = "screenshot-thumbnail";
      img.onclick = () => openFullscreen(url);
      grid.appendChild(img);
    });

    screenshotsContainer.appendChild(grid);
  }

  // Show the preview
  showGameInfo();
}

function openFullscreen(imageUrl) {
  // Implement a lightbox/fullscreen viewer
  console.log("Opening fullscreen:", imageUrl);
  // You can use a library like basicLightbox or implement your own solution
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
    // Replace your existing tag button event listener with this:

    document.querySelectorAll(".tag-button").forEach((button) => {
      button.addEventListener("click", function () {
        const tag = this.getAttribute("tag");
        const allButtons = document.querySelectorAll(".tag-button");
        const gameCards = document.querySelectorAll(".game-card");
        const gameListItems = document.querySelectorAll(".game-list-item");

        // Check if this button was already active
        const wasActive = this.classList.contains("active");

        // First remove active class from all buttons
        allButtons.forEach(btn => btn.classList.remove("active"));

        if (wasActive) {
          // If clicking an active button, show all games
          gameCards.forEach(card => card.classList.remove("hidden"));
          gameListItems.forEach(item => item.classList.remove("hidden"));
          return;
        }

        // Activate the clicked button
        this.classList.add("active");

        // Filter games based on tag
        gameCards.forEach(card => {
          const gameName = card.getAttribute("game");
          const gameData = combined.find(item => item.game.name === gameName);

          if (!gameData) {
            card.classList.add("hidden");
            return;
          }

          const hasTag = gameData.game.tags && gameData.game.tags.includes(tag);
          const isSoloTag = tag === "solo" && (!gameData.game.tags || gameData.game.tags.length === 0);

          card.classList.toggle("hidden", !(hasTag || isSoloTag));
        });

        // Also filter the sidebar list items
        gameListItems.forEach(item => {
          const gameName = item.getAttribute("game");
          const gameData = combined.find(item => item.game.name === gameName);

          if (!gameData) {
            item.classList.add("hidden");
            return;
          }

          const hasTag = gameData.game.tags && gameData.game.tags.includes(tag);
          const isSoloTag = tag === "solo" && (!gameData.game.tags || gameData.game.tags.length === 0);

          item.classList.toggle("hidden", !(hasTag || isSoloTag));
        });
      });
    });
    combined.forEach(({ game, data }) => {
      data.name = game.name;
      displayLibrary(game, data, running);
      displayGameList(game, data, running);
    });


    //library
    let lib_button = document.getElementById("library-button");
    lib_button.addEventListener("click", function () {
      hideGameInfo();
      showGameCards();
      // Set library button as active
      this.classList.add("active");
    });
    //GAME PREVIEW
    function game_click_handler(card) {
      card.addEventListener("click", function () {
        const game = this.getAttribute("game");

        hideGameCards();
        showGameInfo();

        // Remove active class from library button
        document.getElementById("library-button").classList.remove("active");

        // Rest of your existing code...
        let data = null;
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
  processinfo_think();
});

// Debounce scroll events for better performance
function debounce(func, wait = 20, immediate = true) {
  let timeout;
  return function () {
    const context = this, args = arguments;
    const later = function () {
      timeout = null;
      if (!immediate) func.apply(context, args);
    };
    const callNow = immediate && !timeout;
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
    if (callNow) func.apply(context, args);
  };
}

// Optimize scroll performance
window.addEventListener('scroll', debounce(function () {
  // Any scroll-related logic can go here
}), { passive: true });

// Smooth scroll for all anchor links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
  anchor.addEventListener('click', function (e) {
    e.preventDefault();
    document.querySelector(this.getAttribute('href')).scrollIntoView({
      behavior: 'smooth',
      block: 'start'
    });
  });
});

// Search functionality for game list
document.addEventListener('DOMContentLoaded', function () {
  const searchInput = document.getElementById('search-input');
  const gameList = document.getElementById('game-list');
  const games = gameList.getElementsByTagName('li');
  const noResults = document.createElement('div');
  noResults.className = 'no-results';
  noResults.textContent = 'No games found';
  gameList.parentNode.insertBefore(noResults, gameList.nextSibling);

  searchInput.addEventListener('input', function () {
    const searchTerm = this.value.toLowerCase();
    let hasResults = false;

    Array.from(games).forEach(game => {
      const gameName = game.textContent.toLowerCase();
      const isVisible = gameName.includes(searchTerm);

      if (isVisible) hasResults = true;

      game.style.display = isVisible ? 'flex' : 'none';

      // Remove previous highlights
      const spans = game.getElementsByTagName('span');
      while (spans[0]) {
        game.replaceChild(document.createTextNode(spans[0].textContent), spans[0]);
      }

      // Add new highlights
      if (searchTerm && isVisible) {
        const regex = new RegExp(searchTerm, 'gi');
        game.innerHTML = game.textContent.replace(regex,
          match => `<span class="highlight">${match}</span>`);
      }
    });

    noResults.style.display = hasResults || !searchTerm ? 'none' : 'block';
  });


});

