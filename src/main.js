const { invoke } = window.__TAURI__.core;

async function addIcon() {
	try {
		const result = await invoke("add_launcher_desktop_icon");
		if (result != "")
			alert(result);
	} catch (err) {
		alert("Erreur lors de lajout de l'icon : " + err);
	}
}

async function launchGame(game) {
	try {
		const result = await invoke("launch_game", { game: game });
		alert(result);
	} catch (err) {
		alert("Erreur lors du lancement : " + err);
	}
}

async function fetchGameLibrary() {
    try {
        const library = await invoke("get_game_library", {});
        console.log("Game Library:", library);
        return library;
    } catch (error) {
        console.error("Failed to fetch game library:", error);
    }
}

window.addEventListener("DOMContentLoaded", () => {
	fetchGameLibrary().then(library => {
		library.forEach(game => {
			document.querySelector("#games").insertAdjacentHTML("afterbegin", `<button class="game-card" game="${game.name}">${game.display_name}</button>`);
		});

		document.querySelectorAll(".game-card").forEach((card) => {
			card.addEventListener("click", function () {
				const game = this.getAttribute("game");
				launchGame(game);
			});
		});
	});
	document.getElementById("add_icon").addEventListener("click", addIcon);
});



