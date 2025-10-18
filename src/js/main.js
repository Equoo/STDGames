// Import all modules
const { invoke } = window.__TAURI__.core;
import { addIcon, openUrl } from './api/system.js';
import { launchGame, fetchGameLibrary } from './api/games.js';
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
      for (let i = 0; i < library.length; i++) {
        if (library[i].slug === game) {
          data = library[i];
          break;
        }
      }
      
      if (data) {
        changeGamePreview(data);
      } else {
        console.error("Game data not found for:", game);
      }
    });
  };
}

// Initialize the app
window.addEventListener("DOMContentLoaded", async () => {
  // Setup utilities
  debounce(setupSmoothScroll());
  setupSearch();
  
  // Load game library
  const library = await fetchGameLibrary();
  console.log(library);
	if (!library) {
    console.error("Failed to load game library or library structure is invalid");
    return;
  }

  let running = "";
  displayGamePreview(null, null);

  // Combine games and gamesdata
  const combined = library.map((game, i) => {
    let data = game.metadata;
    data.slug = game.slug;
    return (data) 
  });

  // Create the game click handler with access to library
  const gameClickHandler = createGameClickHandler(combined);

  // Initial sort
  sortGames(combined, "descending");
  
  // Setup sorting
  setupSorting(combined, running, gameClickHandler, refreshDisplay);

  // Setup tag filtering
  setupTagFiltering(combined);

  // Initial display - wait for next tick to ensure DOM is ready
  setTimeout(() => {
    combined.forEach((game) => {
      displayLibrary(game, running);
      displayGameList(game, running);
    });

    // Attach click handlers after elements are created
    document.querySelectorAll(".game-card").forEach(gameClickHandler);
    document.querySelectorAll(".game-list-item").forEach(gameClickHandler);

    // Invoke the backend to set client loaded
	//invoke("set_client_loaded", {});
  }, 0);

  // Setup UI event listeners
  setupUIEventListeners(combined, gameClickHandler);


  // Check game running
  let lastGame = "";
  
  const playButton = document.querySelector(".play-button");
  playButton.onclick = async () => {
    const gameSlug = document.querySelector(".game-preview").getAttribute("game");
    console.log(`Play button clicked for game: ${gameSlug}`);

    if (lastGame === gameSlug) {
      console.log(`Killing running game: ${gameSlug}`);
      await invoke("kill_running_game", {});
        playButton.className = "play-button";
      playButton.textContent = "Play";
    } else if (lastGame === "") {
      const success = await launchGame(gameSlug);
      if (success) {
        playButton.className = "play-button kill-button";
        playButton.textContent = "Kill the process";
      }
    }
  };

  setInterval(async () => {
      let game = await invoke("get_running_game", {});
      const button = document.querySelector(`#play`);
      
      if (game != "" && button.textContent != "Kill the process") {
          document.querySelector(`#${game}`).className = "game-card running";
          document.querySelector(`#item_${game}`).className = "game-list-item running";
          if (document.querySelector(".game-preview").getAttribute("game") == game) {
              button.className = "play-button kill-button";
              button.textContent = "Kill the process";
          }
      } else if (game == "" && lastGame != "" && button.textContent != "Play") {
          document.querySelector(`#${lastGame}`).className = "game-card";
          document.querySelector(`#item_${lastGame}`).className = "game-list-item";
          button.className = "play-button";
          button.textContent = "Play";
      }
      if ((game == "" || document.querySelector(".game-preview").getAttribute("game") != game) && button.textContent != "Play") {
          button.className = "play-button";
          button.textContent = "Play";
      }

      lastGame = game;
  }, 100)
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
        const gameData = combined.find(item => item.slug === gameName);

        if (!gameData) {
          card.classList.add("hidden");
          return;
        }

		let hasTag = false;
		if (gameData.tags)
        	hasTag = gameData.tags.includes(tag);
		console.log(gameName, hasTag)
        card.classList.toggle("hidden", !hasTag);
      });

    });
  });
}

function setupUIEventListeners(combined, gameClickHandler) {
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
