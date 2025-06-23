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

export function displayLibrary(game, data, running, containerId = "games") {
  document.querySelector(`#${containerId}`).insertAdjacentHTML(
    "afterbegin",
    `<button class="game-card ${running}" id="${game.name}" game="${game.name}">
      <div style="background-image: url('${data.cover}');"></div>
    </button>`
  );
}

export function displayGameList(game, data, running, containerId = "game-list") {
  document.querySelector(`#${containerId}`).insertAdjacentHTML(
    "afterbegin",
    `<li class="game-list-item ${running}" id="item_${game.name}" game="${game.name}">
      <img src="${data.icon}" alt="${game.name} icon" class="game-list-icon">
      ${data.displayname}
    </li>`
  );
}

export function refreshDisplay(combined, running, gameClickHandler) {
  document.getElementById("games").innerHTML = "";
  document.getElementById("game-list").innerHTML = "";

  combined.forEach(({ game, data }) => {
    displayLibrary(game, data, running);
    displayGameList(game, data, running);
  });

  document.querySelectorAll(".game-card").forEach(gameClickHandler);
  document.querySelectorAll(".game-list-item").forEach(gameClickHandler);
}