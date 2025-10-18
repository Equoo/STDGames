export function hideGameCards() {
  const gamesSection = document.querySelector("#library");
  gamesSection.classList.add("hidden");
}

export function showGameCards() {
  const gamesSection = document.querySelector("#library");
  gamesSection.classList.remove("hidden");
}

export function hideGameInfo() {
  const gamesSection = document.querySelector("#game-preview-container");
  gamesSection.classList.add("hidden");
}

export function showGameInfo() {
  const gamesSection = document.querySelector("#game-preview-container");
  gamesSection.classList.remove("hidden");
}

export function extractImageUrls(imageString) {
  if (!imageString) return [];
  const urls = imageString.split(",");
  return urls[0];
}

export function displayLibrary(game, running, containerId = "games") {
  document.querySelector(`#${containerId}`).insertAdjacentHTML(
    "afterbegin",
    `<button class="game-card ${running}" id="${game.slug}" game="${game.slug}">
      <div style="background-image: url('${game.cover}');"></div>
    </button>`
  );
}

export function displayGameList(game, running, containerId = "game-list") {
  document.querySelector(`#${containerId}`).insertAdjacentHTML(
    "afterbegin",
    `<li class="game-list-item ${running}" id="item_${game.slug}" game="${game.slug}">
      <div class="icon-container"><img src="${game.icon}" alt="${game.slug} icon" class="game-list-icon"></img></div>
      <span>${game.name}</span>
    </li>`
  );
}

export function refreshDisplay(combined, running, gameClickHandler) {
  document.getElementById("games").innerHTML = "";
  document.getElementById("game-list").innerHTML = "";

  combined.forEach((game) => {
    displayLibrary(game, running);
    displayGameList(game, running);
  });

  document.querySelectorAll(".game-card").forEach(gameClickHandler);
  document.querySelectorAll(".game-list-item").forEach(gameClickHandler);
}
