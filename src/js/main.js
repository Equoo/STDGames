// Import all modules
const { invoke } = window.__TAURI__.core;
import { addIcon, openUrl } from './api/system.js';
import { launchGame, fetchGameLibrary, monitorGameProcess } from './api/games.js';
import { 
  hideGameCards, 
  showGameCards, 
  hideGameInfo, 
  showGameInfo,
  displayLibrary,
  displayGameList,
  refreshDisplay
} from './ui/display.js';
import { changeGamePreview, displayGamePreview } from './ui/preview.js';
import { sortGames, setupSorting } from './ui/sorting.js';
import { setupSearch } from './ui/search.js';
import { debounce } from './utils/debounce.js';
import { setupSmoothScroll } from './utils/helpers.js';

// Define gameClickHandler at module level so it's accessible everywhere
function createGameClickHandler(library) {
  return function(card) {
    card.addEventListener("click", function() {
      const game = this.getAttribute("game");
      
      hideGameCards();
      showGameInfo();
      document.getElementById("library-button").classList.remove("active");

      let data = null;
      let gamedata = null;
      for (let i = 0; i < library.gamesdata.length; i++) {
        if (library.games[i].name === game) {
          data = library.gamesdata[i];
          gamedata = library.games[i];
          break;
        }
      }
      
      if (data) {
        changeGamePreview(gamedata, data);
      } else {
        console.error("Game data not found for:", game);
      }
    });
  };
}

// Initialize the app
window.addEventListener("DOMContentLoaded", async () => {
  // Setup utilities
  setupSmoothScroll();
  setupSearch();
  
  // Load game library
  const library = await fetchGameLibrary();
  if (!library || !library.games || !library.gamesdata) {
    console.error("Failed to load game library or library structure is invalid");
    return;
  }

  let running = "";
  displayGamePreview(null, null);

  // Combine games and gamesdata
  const combined = library.games.map((game, i) => ({
    game: game,
    data: library.gamesdata[i],
  }));

  // Create the game click handler with access to library
  const gameClickHandler = createGameClickHandler(library);

  // Initial sort
  sortGames(combined, "descending");
  
  // Setup sorting
  setupSorting(combined, running, gameClickHandler, refreshDisplay);

  // Setup tag filtering
  setupTagFiltering(combined);

  // Initial display - wait for next tick to ensure DOM is ready
  setTimeout(() => {
    combined.forEach(({ game, data }) => {
      data.name = game.name;
      displayLibrary(game, data, running);
      displayGameList(game, data, running);
    });

    // Attach click handlers after elements are created
    document.querySelectorAll(".game-card").forEach(gameClickHandler);
    document.querySelectorAll(".game-list-item").forEach(gameClickHandler);

    // Invoke the backend to set client loaded
    invoke("set_client_loaded", {});
  }, 0);

  // Setup UI event listeners
  setupUIEventListeners(combined, library, gameClickHandler);

  // Start process monitoring
  monitorGameProcess();
});

function setupTagFiltering(combined) {
  const buttons = document.querySelectorAll(".tag-button");
  if (!buttons.length) {
    console.warn("No tag buttons found");
    return;
  }

  buttons.forEach((button) => {
    button.addEventListener("click", function() {
      const tag = this.getAttribute("tag");
      const allButtons = document.querySelectorAll(".tag-button");
      const gameCards = document.querySelectorAll(".game-card");
      const gameListItems = document.querySelectorAll(".game-list-item");

      const wasActive = this.classList.contains("active");
      allButtons.forEach(btn => btn.classList.remove("active"));

      if (wasActive) {
        gameCards.forEach(card => card.classList.remove("hidden"));
        gameListItems.forEach(item => item.classList.remove("hidden"));
        return;
      }

      this.classList.add("active");

      gameCards.forEach(card => {
        const gameName = card.getAttribute("game");
        const gameData = combined.find(item => item.game.name === gameName);

        if (!gameData) {
          card.classList.add("hidden");
          return;
        }

		let hasTag = false;
		if (gameData.game.tags)
        	hasTag = gameData.game.tags.includes(tag);
		console.log(gameName, hasTag)
        // const isSoloTag = tag === "solo" && (!gameData.game.tags || gameData.game.tags.length === 0);

        card.classList.toggle("hidden", !hasTag);
      });

    //   gameListItems.forEach(item => {
    //     const gameName = item.getAttribute("game");
    //     const gameData = combined.find(item => item.game.name === gameName);

    //     if (!gameData) {
    //       item.classList.add("hidden");
    //       return;
    //     }

    //     const hasTag = gameData.game.tags && gameData.game.tags.includes(tag);
    //     const isSoloTag = tag === "solo" && (!gameData.game.tags || gameData.game.tags.length === 0);

    //     item.classList.toggle("hidden", !(hasTag || isSoloTag));
    //   });
    });
  });
}

function setupUIEventListeners(combined, library, gameClickHandler) {
  // Library button
  const libButton = document.getElementById("library-button");
  if (libButton) {
    libButton.addEventListener("click", function() {
      hideGameInfo();
      showGameCards();
      this.classList.add("active");
    });
  }

  // Add icon button
  const addIconButton = document.getElementById("addicon-button");
  if (addIconButton) {
    addIconButton.addEventListener("click", addIcon);
  }

  // About us button
  const aboutButton = document.getElementById("aboutus-button");
  if (aboutButton) {
    aboutButton.addEventListener("click", function() {
      openUrl("https://discord.gg/YR7fwGy5D7");
    });
  }

  // Settings button
  const settingsButton = document.getElementById("settings-button");
  if (settingsButton) {
    settingsButton.addEventListener("click", openSettings);
  }
}

function openSettings() {
  console.log("Settings functionality to be implemented");
  // Implement settings functionality
}