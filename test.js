
	  async function test() {
		//cards
		let elem = document.createElement("button");
		elem.outerHTML(`<button class="game-card ${running}" id="${game.name}" game="${game.name}">
				<div style="background-image: url('${data.cover}');"></div>
			  </button>`);
	  
		elem.addEventListener("click", click_on_game);
		document.querySelector("#games").appendChild(elem);
	  
		//list
		elem = document.createElement("li");
		elem.outerHTML(`<li class="game-list-item ${running}" id="${game.name}">
				  <img src="${data.icon}" alt="${game.name} icon" class="game-list-icon">
				  ${game.name}
				</li>`);
		elem.addEventListener("click", click_on_game);
		document.querySelector("#game-list").appendChild(elem);
	  }