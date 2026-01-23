import { a as attr, b as attr_class, s as store_get, u as unsubscribe_stores, e as ensure_array_like, c as stringify, d as attr_style } from "../../chunks/index2.js";
import { Y as ssr_context, X as escape_html } from "../../chunks/context.js";
import "clsx";
import { d as derived, w as writable } from "../../chunks/index.js";
function html(value) {
  var html2 = String(value ?? "");
  var open = "<!---->";
  return open + html2 + "<!---->";
}
function onDestroy(fn) {
  /** @type {SSRContext} */
  ssr_context.r.on_destroy(fn);
}
const gameLibrary = writable([]);
const runningGame = writable("");
const selectedGame = writable(null);
const currentView = writable("library");
const searchQuery = writable("");
const sortOrder = writable("descending");
const activeTag = writable(null);
const filteredGames = derived(
  [gameLibrary, searchQuery, sortOrder, activeTag],
  ([$gameLibrary, $searchQuery, $sortOrder, $activeTag]) => {
    let games = [...$gameLibrary];
    if ($searchQuery) {
      const query = $searchQuery.toLowerCase();
      games = games.filter((game) => game.name?.toLowerCase().includes(query));
    }
    if ($activeTag) {
      games = games.filter((game) => game.tags?.includes($activeTag));
    }
    games.sort((a, b) => {
      const nameA = a.name?.toLowerCase() || "";
      const nameB = b.name?.toLowerCase() || "";
      if ($sortOrder === "descending") {
        return nameA.localeCompare(nameB);
      } else {
        return nameB.localeCompare(nameA);
      }
    });
    return games;
  }
);
const logo = "/_app/immutable/assets/stdgames.EC2X8a3h.png";
const discordLogo = "data:image/png;base64,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";
const settingsIcon = "data:image/png;base64,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";
function Topbar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    $$renderer2.push(`<div class="topbar svelte-h6bux4"><div class="topbar-content svelte-h6bux4"><div class="left-group svelte-h6bux4"><img${attr("src", logo)} alt="Logo" class="logo svelte-h6bux4"/> <button class="topbar-btn svelte-h6bux4">Add to desktop</button> <button${attr_class("topbar-btn svelte-h6bux4", void 0, {
      "active": store_get($$store_subs ??= {}, "$currentView", currentView) === "library"
    })}>Library</button></div> <div class="right-group svelte-h6bux4"><button class="topbar-btn icon-btn svelte-h6bux4"><img${attr("src", discordLogo)} alt="Discord" class="svelte-h6bux4"/></button> <button class="topbar-btn icon-btn svelte-h6bux4"><img${attr("src", settingsIcon)} alt="Settings" class="svelte-h6bux4"/></button></div></div></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function Sidebar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    function highlightMatch(name, query) {
      if (!query) return name;
      const regex = new RegExp(`(${query})`, "gi");
      return name.replace(regex, '<span class="highlight">$1</span>');
    }
    $$renderer2.push(`<div class="sidebar svelte-129hoe0"><div class="search-container svelte-129hoe0"><div class="search-bar svelte-129hoe0"><span class="search-icon svelte-129hoe0">⌕</span> <input type="text" placeholder="Search ..."${attr("value", store_get($$store_subs ??= {}, "$searchQuery", searchQuery))} class="svelte-129hoe0"/></div></div> <div class="sidebar-content svelte-129hoe0"><div class="game-list svelte-129hoe0" role="list"><!--[-->`);
    const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$filteredGames", filteredGames));
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let game = each_array[$$index];
      $$renderer2.push(`<button type="button"${attr_class("game-list-item svelte-129hoe0", void 0, {
        "running": store_get($$store_subs ??= {}, "$runningGame", runningGame) === game.slug
      })}><div class="icon-container svelte-129hoe0"><img${attr("src", game.icon)}${attr("alt", `${stringify(game.slug)} icon`)} class="game-list-icon svelte-129hoe0"/></div> <span>${html(highlightMatch(game.name || game.slug, store_get($$store_subs ??= {}, "$searchQuery", searchQuery)))}</span></button>`);
    }
    $$renderer2.push(`<!--]--></div></div></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function GameCard($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let { game } = $$props;
    $$renderer2.push(`<button type="button"${attr_class("game-card svelte-n3ft5o", void 0, {
      "running": store_get($$store_subs ??= {}, "$runningGame", runningGame) === game.slug
    })}${attr("aria-label", `Open ${stringify(game.name || game.slug)}`)}><div class="game-cover svelte-n3ft5o"${attr_style(`background-image: url('${stringify(game.cover)}');`)}></div></button>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function Library($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let dropdownOpen = false;
    $$renderer2.push(`<div class="library page svelte-1pli1h7"><div class="library-header svelte-1pli1h7"><h1 class="title svelte-1pli1h7">Library</h1> <div class="right svelte-1pli1h7"><button type="button"${attr_class("tag-button svelte-1pli1h7", void 0, {
      "active": store_get($$store_subs ??= {}, "$activeTag", activeTag) === "multiplayer"
    })}>Multiplayer</button> <button type="button"${attr_class("tag-button svelte-1pli1h7", void 0, {
      "active": store_get($$store_subs ??= {}, "$activeTag", activeTag) === "solo"
    })}>Solo</button> <div class="custom-dropdown svelte-1pli1h7"><button type="button"${attr_class("dropdown-button svelte-1pli1h7", void 0, { "active": dropdownOpen })}>Sort by ▼</button> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div></div></div> <div${attr_class("games-container svelte-1pli1h7", void 0, {
      "has-running": store_get($$store_subs ??= {}, "$runningGame", runningGame) !== ""
    })}><!--[-->`);
    const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$filteredGames", filteredGames));
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let game = each_array[$$index];
      GameCard($$renderer2, { game });
    }
    $$renderer2.push(`<!--]--></div></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function Carousel($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { screenshots = [], videos = [], thumbnails = [] } = $$props;
    let mediaItems = (() => {
      const result = [];
      const totalScreenshots = screenshots?.length || 0;
      const totalVideos = videos?.length || 0;
      const totalMedia = totalScreenshots + totalVideos;
      if (totalMedia === 0) return result;
      const videosInterval = totalVideos > 0 ? Math.floor(totalScreenshots / totalVideos) : Infinity;
      let videoIndex = 0;
      let screenshotIndex = 0;
      for (let i = 0; i < totalMedia; i++) {
        const isVideo = totalVideos > 0 && videosInterval > 0 && i % (videosInterval + 1) === 0 && videoIndex < totalVideos;
        if (isVideo && videos) {
          result.push({
            type: "video",
            src: videos[videoIndex],
            thumbnail: thumbnails?.[videoIndex]
          });
          videoIndex++;
        } else if (screenshotIndex < totalScreenshots && screenshots) {
          result.push({ type: "image", src: screenshots[screenshotIndex] });
          screenshotIndex++;
        }
      }
      return result;
    })();
    if (mediaItems.length > 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="carousel-container svelte-8ojyxu"><div class="carousel-track svelte-8ojyxu"><!--[-->`);
      const each_array = ensure_array_like(mediaItems);
      for (let i = 0, $$length = each_array.length; i < $$length; i++) {
        let item = each_array[i];
        $$renderer2.push(`<div${attr_class("carousel-item svelte-8ojyxu", void 0, { "video": item.type === "video" })}>`);
        if (item.type === "video") {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<video class="media-content svelte-8ojyxu" muted loop${attr("autoplay", i === 0, true)}${attr("poster", item.thumbnail)}><source${attr("src", item.src)} type="video/mp4"/> <track kind="captions"/></video> <div class="video-overlay svelte-8ojyxu"><button type="button" class="play-btn svelte-8ojyxu" aria-label="Play or pause video"><svg class="play-icon svelte-8ojyxu" viewBox="0 0 24 24" aria-hidden="true"><polygon points="5,3 19,12 5,21"></polygon></svg></button></div>`);
        } else {
          $$renderer2.push("<!--[!-->");
          $$renderer2.push(`<img class="media-content svelte-8ojyxu"${attr("src", item.src)}${attr("alt", `Screenshot ${stringify(i + 1)}`)}/>`);
        }
        $$renderer2.push(`<!--]--></div>`);
      }
      $$renderer2.push(`<!--]--></div></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]-->`);
  });
}
function GamePreview($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let isLaunching = false;
    let artworkUrl = store_get($$store_subs ??= {}, "$selectedGame", selectedGame)?.hero || store_get($$store_subs ??= {}, "$selectedGame", selectedGame)?.cover || store_get($$store_subs ??= {}, "$selectedGame", selectedGame)?.screenshots?.[0] || "";
    let isRunning = store_get($$store_subs ??= {}, "$runningGame", runningGame) === store_get($$store_subs ??= {}, "$selectedGame", selectedGame)?.slug;
    if (store_get($$store_subs ??= {}, "$selectedGame", selectedGame)) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="game-preview-container svelte-mmsfr6"><button class="back-button svelte-mmsfr6">← Back to library</button> <div class="game-preview svelte-mmsfr6"><div class="image-crop-container svelte-mmsfr6"><div class="game-preview-artwork svelte-mmsfr6"${attr_style(`background-image: url('${stringify(artworkUrl)}');`)}></div> <h1 class="title-overlay svelte-mmsfr6">${escape_html(store_get($$store_subs ??= {}, "$selectedGame", selectedGame).name || store_get($$store_subs ??= {}, "$selectedGame", selectedGame).slug)}</h1> `);
      if (store_get($$store_subs ??= {}, "$selectedGame", selectedGame).tags && store_get($$store_subs ??= {}, "$selectedGame", selectedGame).tags.length > 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="game-genres svelte-mmsfr6"><!--[-->`);
        const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$selectedGame", selectedGame).tags);
        for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
          let tag = each_array[$$index];
          $$renderer2.push(`<div class="game-genres-item svelte-mmsfr6">${escape_html(tag)}</div>`);
        }
        $$renderer2.push(`<!--]--></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <div class="game-meta svelte-mmsfr6">`);
      if (store_get($$store_subs ??= {}, "$selectedGame", selectedGame).short_description) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="game-description svelte-mmsfr6">${escape_html(store_get($$store_subs ??= {}, "$selectedGame", selectedGame).short_description)}</div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <div class="button-overlay svelte-mmsfr6"><button${attr_class("play-button svelte-mmsfr6", void 0, { "kill-button": isRunning })}${attr("disabled", isLaunching, true)}>${escape_html(isRunning ? "Kill the process" : "Play")}</button> <button class="game-settings-button svelte-mmsfr6">⚙️ Settings</button></div></div></div> <div class="game-details svelte-mmsfr6">`);
      Carousel($$renderer2, {
        screenshots: store_get($$store_subs ??= {}, "$selectedGame", selectedGame).screenshots,
        videos: store_get($$store_subs ??= {}, "$selectedGame", selectedGame).movies,
        thumbnails: store_get($$store_subs ??= {}, "$selectedGame", selectedGame).movies_thumbnails
      });
      $$renderer2.push(`<!----></div> `);
      if (store_get($$store_subs ??= {}, "$selectedGame", selectedGame).description) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="description-section svelte-mmsfr6">${html(store_get($$store_subs ??= {}, "$selectedGame", selectedGame).description)}</div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]-->`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    onDestroy(() => {
    });
    Topbar($$renderer2);
    $$renderer2.push(`<!----> <div class="frosted-glass svelte-1uha8ag"><div class="big-container svelte-1uha8ag">`);
    Sidebar($$renderer2);
    $$renderer2.push(`<!----> `);
    if (store_get($$store_subs ??= {}, "$currentView", currentView) === "library") {
      $$renderer2.push("<!--[-->");
      Library($$renderer2);
    } else {
      $$renderer2.push("<!--[!-->");
      GamePreview($$renderer2);
    }
    $$renderer2.push(`<!--]--></div></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
