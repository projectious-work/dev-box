// Auto-initialize asciinema-player elements on page load.
// Usage in markdown (via raw HTML):
//
//   <div class="asciinema" data-cast="assets/screencasts/layout-dev.cast"
//        data-poster="npt:4" data-cols="160" data-rows="45"></div>
//
// For a still frame (no controls):
//
//   <div class="asciinema" data-cast="assets/screencasts/init-demo.cast"
//        data-poster="npt:3" data-controls="false"></div>

function getBaseUrl() {
  // Derive site root from the player CSS link that MkDocs already relativized
  var link = document.querySelector('link[href*="asciinema-player.css"]');
  if (link) {
    // href is e.g. "../../assets/player/asciinema-player.css" — strip the filename portion
    return link.getAttribute("href").replace("assets/player/asciinema-player.css", "");
  }
  return "";
}

function initAsciinemaPlayers() {
  var base = getBaseUrl();

  document.querySelectorAll("div.asciinema:not([data-initialized])").forEach(function (el) {
    var src = el.getAttribute("data-cast");
    if (!src) return;

    // Prepend base so paths work from any page depth
    src = base + src;

    var opts = {
      poster: el.getAttribute("data-poster") || "npt:0",
      autoPlay: el.getAttribute("data-autoplay") !== "false",
      loop: el.getAttribute("data-loop") === "true",
      fit: el.getAttribute("data-fit") || "width",
      terminalFontSize: el.getAttribute("data-font-size") || "small",
      speed: parseFloat(el.getAttribute("data-speed")) || 1,
    };

    if (el.getAttribute("data-cols"))
      opts.cols = parseInt(el.getAttribute("data-cols"));
    if (el.getAttribute("data-rows"))
      opts.rows = parseInt(el.getAttribute("data-rows"));
    if (el.getAttribute("data-controls") === "false")
      opts.controls = false;

    AsciinemaPlayer.create(src, el, opts);
    el.setAttribute("data-initialized", "true");

    // Propagate data-theme to the .ap-player element for CSS palette overrides
    var theme = el.getAttribute("data-theme");
    if (theme) {
      var player = el.querySelector(".ap-player");
      if (player) player.setAttribute("data-theme", theme);
    }
  });
}

// Initialize on DOMContentLoaded and on MkDocs instant navigation
document.addEventListener("DOMContentLoaded", initAsciinemaPlayers);

// MkDocs Material instant loading triggers this custom event
if (typeof document$ !== "undefined") {
  document$.subscribe(function () {
    initAsciinemaPlayers();
  });
}
