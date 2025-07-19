import { showGameCards, hideGameInfo, showGameInfo } from './display.js';
import { launchGame } from '../api/games.js';

export function displayGamePreview(game, data) {
  const gamesSection = document.querySelector("#game-preview-container");
  if (gamesSection == null) {
    document.querySelector("#game-preview-container").classList.add("hidden");
  }
}

export function changeGamePreview(game, data) {
  const gameSection = document.querySelector("#game-preview-container");
  if (!gameSection) return;

  gameSection.querySelector(".game-preview").setAttribute("game", game.name);

  const artworkUrl = data.artworks?.[0] || data.cover || './resources/default-game.jpg';
  document.querySelector(".game-preview-artwork").style.backgroundImage = `url('${artworkUrl}')`;

  document.querySelector(".title-overlay").textContent = data.name || game.name;

  document.getElementById('back-to-library').addEventListener('click', () => {
    showGameCards();
    hideGameInfo();
    document.getElementById("library-button").classList.add("active");
  });

  const descElement = document.querySelector(".game-description");
  descElement.textContent = data.summary || "No description available";
  descElement.style.display = data.summary ? "block" : "none";

  updateGenres(data.genres);
  updatePlayButton(game);
  updateScreenshots(data.screenshots);

  showGameInfo();
}

function updateGenres(genres) {
  const genresContainer = document.querySelector(".game-genres");
  genresContainer.innerHTML = "";

  if (genres && genres.length > 0) {
    genres.forEach(genre => {
      const genreElement = document.createElement("div");
      genreElement.className = "game-genres-item";
      genreElement.textContent = genre;
      genresContainer.appendChild(genreElement);
    });
  }
}

function updatePlayButton(game) {
  const playButton = document.querySelector(".play-button");
  playButton.setAttribute("data-game", game.name);
  playButton.onclick = () => launchGame(game.name);
}

function updateScreenshots(screenshots) {
  const screenshotsContainer = document.querySelector(".screenshots-container");
  screenshotsContainer.innerHTML = "";

  if (screenshots && screenshots.length > 0) {
    const screenshotsTitle = document.createElement("h3");
    screenshotsContainer.appendChild(screenshotsTitle);

    const grid = document.createElement("div");
    grid.className = "screenshots-grid";

    screenshots.forEach(url => {
      const img = document.createElement("img");
      img.src = url;
      img.className = "screenshot-thumbnail";
      img.onclick = () => openFullscreen(url);
      grid.appendChild(img);
    });

    screenshotsContainer.appendChild(grid);
  }
}

function openFullscreen(imageUrl) {
  console.log("Opening fullscreen:", imageUrl);
}
