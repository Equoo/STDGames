import { showGameCards, hideGameInfo, showGameInfo } from './display.js';
import { launchGame } from '../api/games.js';

export function displayGamePreview(game) {
  const gamesSection = document.querySelector("#game-preview-container");
  if (gamesSection == null) {
    document.querySelector("#game-preview-container").classList.add("hidden");
  }
}

export function changeGamePreview(game) {
  const gameSection = document.querySelector("#game-preview-container");
  if (!gameSection) return;

  gameSection.querySelector(".game-preview").setAttribute("game", game.slug);

  const artworkUrl = game.hero || game.screenshots?.[0] || game.cover || './resources/default-game.jpg';
  document.querySelector(".game-preview-artwork").style.backgroundImage = `url('${artworkUrl}')`;

  document.querySelector(".title-overlay").textContent = game.name;

  document.getElementById('back-to-library').addEventListener('click', () => {
    showGameCards();
    hideGameInfo();
    document.getElementById("library-button").classList.add("active");
  });

  const descElement = document.querySelector(".game-description");
  descElement.textContent = game.short_description || "No description available";
  descElement.style.display = game.short_description ? "block" : "none";

  if (game.tags) {
    updateGenres(game.tags);
  }
  updatePlayButton(game);
  updateScreenshots(game.screenshots);

  document.getElementById("description-section").innerHTML = game.description || "<p>No description available.</p>";
  const videos = document.querySelectorAll('video');
  console.log(videos);
  videos.forEach(video => {
      // Ensure autoplay and loop
      video.muted = true;
      video.loop = false;
      video.playsInline = true;
      
      // Force loop manually
      video.addEventListener('ended', function() {
          this.currentTime = 0;
          this.play();
      });
      
      // Handle loading and start playing
      video.addEventListener('loadeddata', function() {
          video.play().catch(err => {
              console.error('Video play failed:', err);
          });
      });
      
      // Additional loop safety
      video.addEventListener('timeupdate', function() {
          // If video is near the end, restart it
          if (video.duration - video.currentTime < 0.1) {
              video.currentTime = 0;
          }
      });
  });

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
  playButton.setAttribute("data-game", game.slug);
  playButton.onclick = () => launchGame(game.slug);
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
