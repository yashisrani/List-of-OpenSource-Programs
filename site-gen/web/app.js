// Progressive enhancement for the program list.
//
// The full list is server-rendered, so the page works with JS disabled and is
// fully crawlable. This file only layers filtering on top: it reads the model
// out of data-attributes already present in the DOM rather than fetching a
// second copy of the data.

(function () {
  "use strict";

  // ---- Theme -------------------------------------------------------------
  // Applied inline in <head> to avoid a flash; this only wires the toggle.
  var root = document.documentElement;
  var darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

  function currentTheme() {
    return root.getAttribute("data-theme") || (darkQuery.matches ? "dark" : "light");
  }

  function labelFor(theme) {
    return theme === "dark" ? "Switch to light theme" : "Switch to dark theme";
  }

  var themeBtn = document.querySelector(".theme-btn");
  if (themeBtn) {
    themeBtn.setAttribute("aria-label", labelFor(currentTheme()));

    themeBtn.addEventListener("click", function () {
      var next = currentTheme() === "dark" ? "light" : "dark";

      // Suppress every transition for the duration of the swap so the whole
      // page changes in one repaint. Without this, elements with their own
      // hover transitions (nav links, chips, rows) lag behind the rest and the
      // switch looks like a ripple.
      root.setAttribute("data-switching", "");
      root.setAttribute("data-theme", next);

      // Re-enable on the next frame. rAF is throttled in background tabs and
      // would otherwise leave transitions permanently off, so a timer backs it
      // up; whichever fires first wins and the other is a no-op.
      var restore = function () {
        root.removeAttribute("data-switching");
      };
      requestAnimationFrame(function () {
        requestAnimationFrame(restore);
      });
      setTimeout(restore, 120);

      try {
        localStorage.setItem("theme", next);
      } catch (e) {
        /* private mode: the toggle still works for this page view */
      }
      themeBtn.setAttribute("aria-label", labelFor(next));
    });
  }

  // Follow the OS while the user has not made an explicit choice.
  darkQuery.addEventListener("change", function () {
    var stored = null;
    try {
      stored = localStorage.getItem("theme");
    } catch (e) {
      /* ignore */
    }
    if (!stored && themeBtn) {
      themeBtn.setAttribute("aria-label", labelFor(currentTheme()));
    }
  });

  // ---- Filtering ---------------------------------------------------------
  var list = document.querySelector("[data-list]");
  if (!list) return;

  var rows = Array.prototype.slice.call(list.querySelectorAll("[data-row]"));
  var search = document.querySelector("[data-search]");
  var chips = Array.prototype.slice.call(document.querySelectorAll("[data-tag]"));
  var count = document.querySelector("[data-count]");
  var empty = document.querySelector("[data-empty]");
  var reset = document.querySelector("[data-reset]");

  // Pre-extract the haystack once. Doing this per keystroke over ~20 rows
  // would be fine, but it costs nothing to hoist and keeps input handling flat.
  var model = rows.map(function (row) {
    return {
      el: row,
      text: (row.getAttribute("data-search-text") || "").toLowerCase(),
      tags: (row.getAttribute("data-tags") || "").split(" ").filter(Boolean),
    };
  });

  var active = new Set();
  var query = "";

  function apply() {
    var shown = 0;
    model.forEach(function (item) {
      var matchesQuery = !query || item.text.indexOf(query) !== -1;
      var matchesTags = true;
      active.forEach(function (tag) {
        if (item.tags.indexOf(tag) === -1) matchesTags = false;
      });
      var visible = matchesQuery && matchesTags;
      item.el.hidden = !visible;
      if (visible) shown++;
    });

    if (count) {
      count.textContent = shown === model.length
        ? model.length + " programs"
        : shown + " of " + model.length;
    }
    if (empty) empty.hidden = shown !== 0;

    // Announce the result count to screen readers without stealing focus.
    var live = document.querySelector("[data-live]");
    if (live) live.textContent = shown + " programs match";
  }

  if (search) {
    search.addEventListener("input", function () {
      query = search.value.trim().toLowerCase();
      apply();
    });
    // Escape clears the field, matching the muscle memory of every other
    // search box.
    search.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && search.value) {
        search.value = "";
        query = "";
        apply();
      }
    });
  }

  chips.forEach(function (chip) {
    chip.addEventListener("click", function () {
      var tag = chip.getAttribute("data-tag");
      if (active.has(tag)) {
        active.delete(tag);
        chip.setAttribute("aria-pressed", "false");
      } else {
        active.add(tag);
        chip.setAttribute("aria-pressed", "true");
      }
      apply();
    });
  });

  if (reset) {
    reset.addEventListener("click", function () {
      active.clear();
      query = "";
      if (search) search.value = "";
      chips.forEach(function (c) {
        c.setAttribute("aria-pressed", "false");
      });
      apply();
      if (search) search.focus();
    });
  }
})();
