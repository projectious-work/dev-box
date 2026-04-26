// Auto-initialize asciinema-player elements on page load.
// Usage in markdown (via raw HTML):
//
//   <div class="asciinema" data-cast="/aibox/screencasts/layout-dev.cast"
//        data-poster="npt:4" data-cols="160" data-rows="45"></div>
//
// Use absolute paths (starting with /) so this works regardless of page depth.

function initAsciinemaPlayers() {
  if (typeof AsciinemaPlayer === 'undefined') return;

  document.querySelectorAll("div.asciinema:not([data-initialized])").forEach(function (el) {
    var src = el.getAttribute("data-cast");
    if (!src) return;

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

    var theme = el.getAttribute("data-theme");
    if (theme) {
      var player = el.querySelector(".ap-player");
      if (player) player.setAttribute("data-theme", theme);
    }
  });
}

document.addEventListener("DOMContentLoaded", initAsciinemaPlayers);

// Docusaurus client-side navigation: re-run after each page transition
if (typeof window !== "undefined") {
  var _origPushState = history.pushState;
  history.pushState = function () {
    _origPushState.apply(this, arguments);
    setTimeout(initAsciinemaPlayers, 300);
  };
}
