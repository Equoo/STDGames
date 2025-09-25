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

  const artworkUrl = game.hero || game.cover || game.screenshots?.[0] || './resources/default-game.jpg';
  document.querySelector(".game-preview-artwork").style.backgroundImage = `url('${artworkUrl}')`;

  document.querySelector(".title-overlay").textContent = game.name;

  document.getElementById('back-to-library').addEventListener('click', () => {
    showGameCards();
    hideGameInfo();
    document.getElementById("library-button").classList.add("active");
  });

    document.querySelector(".button-overlay .play-button").className = "play-button kill-button";

  const descElement = document.querySelector(".game-description");
  descElement.textContent = game.short_description || "No description available";
  descElement.style.display = game.short_description ? "block" : "none";

  if (game.tags) {
    updateGenres(game.tags);
  }
  updatePlayButton(game);
  updateMedias(game.screenshots, game.movies, game.movies_thumbnails);

  document.getElementById("description-section").innerHTML = game.description || "<p>No description available.</p>";
  const videos = document.querySelectorAll('#description-section video');
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
  playButton.onclick = () => {
    if (launchGame(game.slug)) {
      playButton.textContent = "Running...";
      playButton.disabled = true;
    }
  };
}

import { init_carousel } from './carousel.js';

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function updateMedias(screenshots, videos, thumbnails) {
	const container = document.querySelector(".carousel-track");
	container.innerHTML = "";

	let nbr_medias = screenshots.length + videos.length;
	let videos_each = screenshots.length / videos.length;
	let videos_i = 0;

	for (let i = 0; i < nbr_medias; i++) {
		let is_video = videos.length != 0 && i % videos_each == 0

      		const e = document.createElement("div");
		e.className = "carousel-item";
		if (is_video) {
			let source = videos[videos_i];
			let thumb = thumbnails[videos_i];
			e.className = e.className + " video";
			e.innerHTML = `
			<video class="media-content" muted loop ${i === 0 ? "autoplay" : ""}
			       poster="${thumb}">
			    <source src="${source}" type="video/mp4">
			</video>
			<div class="video-overlay">
			    <button class="play-button" onclick="toggleVideo(this)">
				<svg class="play-icon" viewBox="0 0 24 24">
				    <polygon points="5,3 19,12 5,21"></polygon>
				</svg>
			    </button>
			</div>
			`;
			videos_i++;
		} else {
			let source = screenshots[i - videos_i];
			e.innerHTML = `
			<img class="media-content" 
			     src="${source}">
			`;
		}

		container.appendChild(e);
	}
	init_carousel(container.parentElement);
}
function openFullscreen(imageUrl) {
  console.log("Opening fullscreen:", imageUrl);
}
