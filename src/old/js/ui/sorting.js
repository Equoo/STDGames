// sort
export function sortGames(combined, order) {
  if (order === "descending") {
    combined.sort((game) =>
      game.slug.localeCompare(game.slug, undefined, {
        sensitivity: "base",
      })
    );
  } else if (order === "ascending") {
    combined.sort((a, b) =>
      game.slug.localeCompare(game.slug, undefined, {
        sensitivity: "base",
      })
    );
  }
}

// dropdown sort
export function setupSorting(combined, running, gameClickHandler, refreshDisplay) {
  document.querySelectorAll("#dropdown-menu li").forEach((item) => {
    item.addEventListener("click", async () => {
      const selectedOrder = item.getAttribute("data-value");
      await sortGames(combined, selectedOrder);
      refreshDisplay(combined, running, gameClickHandler);
      document.getElementById("dropdown-menu").classList.add("hidden");
    });
  });
}
