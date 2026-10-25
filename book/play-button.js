// Adds a "play the game" button to the menu bar, next to print and GitHub. The game is
// deployed into play/ at the book's root (see .github/workflows/pages.yml).
(() => {
  const buttons = document.querySelector(".right-buttons");
  if (!buttons) return;
  const link = document.createElement("a");
  link.href = `${path_to_root}play/`;
  link.title = "Play the game";
  link.setAttribute("aria-label", "Play the game");
  // Font Awesome Free 6 "play" (CC BY 4.0), like mdBook's own icons.
  link.innerHTML =
    '<span class="fa-svg"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 384 512">' +
    '<path d="M73 39c-14.8-9.1-33.4-9.4-48.5-.9S0 62.6 0 80V432c0 17.4 9.4 33.4 24.5 41.9s33.7 ' +
    '8.1 48.5-.9L361 297c14.3-8.7 23-24.2 23-41s-8.7-32.2-23-41L73 39z"/></svg></span>';
  buttons.prepend(link);
})();
