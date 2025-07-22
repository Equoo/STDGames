export function setupSearch() {
	const searchInput = document.getElementById('search-input');
	const gameList = document.getElementById('game-list');
	const games = gameList.getElementsByTagName('li');
	const noResults = document.createElement('div');
	noResults.className = 'no-results';
	noResults.textContent = 'No games found';
	gameList.parentNode.insertBefore(noResults, gameList.nextSibling);

	searchInput.addEventListener('input', function () {
		const searchTerm = this.value.toLowerCase();
		let hasResults = false;

		Array.from(games).forEach(game => {
			const game_title = game.getElementsByTagName("h5")[0];

			const gameName = game_title.textContent.toLowerCase();
			const isVisible = gameName.includes(searchTerm);

			if (isVisible) hasResults = true;

			game.style.display = isVisible ? 'flex' : 'none';


			const spans = game_title.getElementsByTagName('span');
			while (spans[0]) {
				game_title.replaceChild(document.createTextNode(spans[0].textContent), spans[0]);
			}

			if (searchTerm && isVisible) {
				const regex = new RegExp(searchTerm, 'gi');
				game_title.innerHTML = game_title.textContent.replace(regex,
					match => `<span class="highlight">${match}</span>`);
			}
		});

		noResults.style.display = hasResults || !searchTerm ? 'none' : 'block' ;
		noResults.style.position = hasResults || !searchTerm ? 'none' : 'absolute' ;
	});
}