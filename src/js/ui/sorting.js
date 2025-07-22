export function sortGames(combined, order) {
  if (order === "descending") {
    combined.sort((a, b) =>
      b.data.name.localeCompare(a.data.name, undefined, {
        sensitivity: "base",
      })
    );
  } else if (order === "ascending") {
    combined.sort((a, b) =>
      a.data.name.localeCompare(b.data.name, undefined, {
        sensitivity: "base",
      })
    );
  }
}

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
