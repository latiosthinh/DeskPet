import {
  DEFAULT_STATE,
  PAGE_SIZE,
  filterPets,
  pagePath,
  parseState,
  sortPets,
  stateParams,
  type CatalogueState,
  type IndexPet,
} from "./catalogue-core";

export function initCatalogue(): void {
  const controls = document.querySelector<HTMLElement>("[data-controls]");
  const grid = document.querySelector<HTMLUListElement>("#catalogue-grid");
  const template = document.querySelector<HTMLTemplateElement>("#pet-card-template");
  const count = document.querySelector<HTMLElement>("[data-count]");
  const pagination = document.querySelector<HTMLElement>("[data-pagination]");
  const empty = document.querySelector<HTMLElement>("[data-empty]");
  const chipsRow = document.querySelector<HTMLElement>("[data-chips-row]");
  const chipList = document.querySelector<HTMLElement>("[data-chip-list]");
  const clear = document.querySelector<HTMLButtonElement>("[data-clear]");
  const reset = document.querySelector<HTMLButtonElement>("[data-reset]");
  const toggle = document.querySelector<HTMLButtonElement>("[data-filters-toggle]");
  const panel = document.querySelector<HTMLElement>("[data-filters-panel]");
  const qInput = document.querySelector<HTMLInputElement>("input[data-q]");
  const sortSelect = document.querySelector<HTMLSelectElement>("select[data-sort]");
  const categorySelect = document.querySelector<HTMLSelectElement>("select[data-category]");
  const authorSelect = document.querySelector<HTMLSelectElement>("select[data-author]");
  const typeSelect = document.querySelector<HTMLSelectElement>("select[data-type]");
  if (
    !controls ||
    !grid ||
    !template ||
    !count ||
    !pagination ||
    !empty ||
    !chipsRow ||
    !chipList ||
    !clear ||
    !reset ||
    !toggle ||
    !panel ||
    !qInput ||
    !sortSelect ||
    !categorySelect ||
    !authorSelect ||
    !typeSelect
  ) {
    return;
  }

  const cardTemplate = template.content.firstElementChild as HTMLElement | null;
  if (!cardTemplate) return;

  void (async () => {
    let index: IndexPet[];
    try {
      const response = await fetch("/pets/catalogue.json");
      if (!response.ok) return;
      const data: unknown = await response.json();
      if (!data || typeof data !== "object" || !Array.isArray((data as { pets?: unknown }).pets)) {
        return;
      }
      index = (data as { pets: IndexPet[] }).pets;
    } catch {
      return;
    }

    const valid = {
      categories: [...new Set(index.map((pet) => pet.category))],
      authors: [...new Set(index.map((pet) => pet.authorSlug))],
    };

    let state: CatalogueState = parseState(new URLSearchParams(location.search), valid);
    let searchTimer: ReturnType<typeof setTimeout> | undefined;
    let searchCommitted = false;
    let pageCount = 1;

    function optionText(select: HTMLSelectElement, value: string): string {
      return Array.from(select.options).find((option) => option.value === value)?.text ?? value;
    }

    function syncControls() {
      qInput!.value = state.q;
      sortSelect!.value = state.sort;
      categorySelect!.value = state.category;
      authorSelect!.value = state.author;
      typeSelect!.value = state.type;
    }

    function commit(push: boolean) {
      const params = stateParams(state).toString();
      const url = `/pets/${params ? `?${params}` : ""}`;
      history[push ? "pushState" : "replaceState"]({}, "", url);
    }

    function renderCard(pet: IndexPet): HTMLLIElement {
      const card = cardTemplate!.cloneNode(true) as HTMLAnchorElement;
      card.href = `/pets/${pet.id}/`;
      const img = card.querySelector<HTMLImageElement>("[data-img]");
      if (img) {
        img.src = pet.gallery.src;
        img.width = pet.gallery.width;
        img.height = pet.gallery.height;
      }
      const title = card.querySelector<HTMLElement>("[data-title]");
      if (title) {
        title.textContent = pet.shortName;
        title.title = pet.name;
      }
      const tricks = card.querySelector<HTMLElement>("[data-tricks]");
      if (tricks) {
        tricks.textContent = `${pet.trickCount} ${pet.trickCount === 1 ? "trick" : "tricks"}`;
      }
      const badge = card.querySelector<HTMLElement>("[data-badge]");
      const addon = card.querySelector<HTMLElement>("[data-addon]");
      if (badge) badge.hidden = !pet.builtin;
      if (addon) addon.hidden = pet.builtin;
      const item = document.createElement("li");
      item.append(card);
      return item;
    }

    function renderChips() {
      chipList!.replaceChildren();
      const chips: { label: string; clear: () => void }[] = [];
      if (state.q) {
        chips.push({
          label: `"${state.q}"`,
          clear: () => {
            state.q = "";
            qInput!.value = "";
          },
        });
      }
      if (state.category) {
        chips.push({
          label: optionText(categorySelect!, state.category),
          clear: () => {
            state.category = "";
            categorySelect!.value = "";
          },
        });
      }
      if (state.author) {
        chips.push({
          label: optionText(authorSelect!, state.author),
          clear: () => {
            state.author = "";
            authorSelect!.value = "";
          },
        });
      }
      if (state.type) {
        chips.push({
          label: state.type === "included" ? "Included with DeskPet" : "Add-on",
          clear: () => {
            state.type = "";
            typeSelect!.value = "";
          },
        });
      }

      chips.forEach((chip, chipIndex) => {
        const item = document.createElement("li");
        const label = document.createElement("span");
        label.textContent = chip.label;
        const remove = document.createElement("button");
        remove.type = "button";
        remove.textContent = "×";
        remove.setAttribute("aria-label", `Remove filter: ${chip.label}`);
        remove.addEventListener("click", () => {
          chip.clear();
          state.page = 1;
          searchCommitted = false;
          clearTimeout(searchTimer);
          render();
          commit(true);
          const buttons = Array.from(chipList!.querySelectorAll<HTMLButtonElement>("button"));
          const next = buttons[chipIndex];
          if (next) next.focus();
          else if (!chipsRow!.hidden) clear!.focus();
          else qInput!.focus();
        });
        item.append(label, remove);
        chipList!.append(item);
      });

      chipsRow!.hidden = chips.length === 0;
    }

    function renderPagination() {
      if (pageCount <= 1) {
        pagination!.hidden = true;
        pagination!.replaceChildren();
        return;
      }
      pagination!.hidden = false;
      const fullyDefault = stateParams({ ...state, page: 1 }).toString() === "";
      const hrefFor = (page: number) =>
        fullyDefault ? pagePath("/pets/", page) : `/pets/?${stateParams({ ...state, page })}`;
      const makeLink = (page: number, text: string) => {
        const link = document.createElement("a");
        link.href = hrefFor(page);
        link.dataset.page = String(page);
        link.textContent = text;
        return link;
      };
      const nodes: HTMLAnchorElement[] = [];
      if (state.page > 1) {
        const previous = makeLink(state.page - 1, "Previous");
        previous.rel = "prev";
        previous.setAttribute("aria-label", "Previous page");
        nodes.push(previous);
      }
      for (let page = 1; page <= pageCount; page++) {
        const link = makeLink(page, String(page));
        if (page === state.page) link.setAttribute("aria-current", "page");
        nodes.push(link);
      }
      if (state.page < pageCount) {
        const next = makeLink(state.page + 1, "Next");
        next.rel = "next";
        next.setAttribute("aria-label", "Next page");
        nodes.push(next);
      }
      pagination!.replaceChildren(...nodes);
    }

    function render() {
      const filtered = sortPets(
        filterPets(index, state, (pet) => pet.authorSlug),
        state.sort,
      );
      pageCount = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
      state.page = Math.min(Math.max(state.page, 1), pageCount);

      const start = (state.page - 1) * PAGE_SIZE;
      grid!.replaceChildren(...filtered.slice(start, start + PAGE_SIZE).map(renderCard));

      count!.textContent = `${filtered.length} ${filtered.length === 1 ? "pet" : "pets"}`;
      renderChips();
      empty!.hidden = filtered.length > 0;
      renderPagination();
    }

    function resetAll() {
      state = { ...DEFAULT_STATE };
      searchCommitted = false;
      clearTimeout(searchTimer);
      syncControls();
      render();
      commit(true);
      qInput!.focus();
    }

    toggle.addEventListener("click", () => {
      const open = panel.dataset.open !== "true";
      panel.dataset.open = String(open);
      toggle.setAttribute("aria-expanded", String(open));
    });

    qInput.addEventListener("input", () => {
      state.q = qInput.value;
      state.page = 1;
      render();
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => {
        commit(!searchCommitted);
        searchCommitted = true;
      }, 500);
    });

    const selects: [HTMLSelectElement, (value: string) => void][] = [
      [
        sortSelect,
        (value) => {
          state.sort = value as CatalogueState["sort"];
        },
      ],
      [
        categorySelect,
        (value) => {
          state.category = value;
        },
      ],
      [
        authorSelect,
        (value) => {
          state.author = value;
        },
      ],
      [
        typeSelect,
        (value) => {
          state.type = value as CatalogueState["type"];
        },
      ],
    ];
    for (const [select, apply] of selects) {
      select.addEventListener("change", () => {
        apply(select.value);
        state.page = 1;
        searchCommitted = false;
        clearTimeout(searchTimer);
        render();
        commit(true);
      });
    }

    clear.addEventListener("click", resetAll);
    reset.addEventListener("click", resetAll);

    pagination.addEventListener("click", (event) => {
      const link = (event.target as HTMLElement).closest<HTMLAnchorElement>("a[data-page]");
      if (!link) return;
      if (stateParams({ ...state, page: 1 }).toString() === "") return;
      const page = Number(link.dataset.page);
      if (!Number.isInteger(page)) return;
      event.preventDefault();
      state.page = page;
      searchCommitted = false;
      clearTimeout(searchTimer);
      render();
      commit(true);
      pagination.querySelector<HTMLElement>('[aria-current="page"]')?.focus();
    });

    window.addEventListener("popstate", () => {
      state = parseState(new URLSearchParams(location.search), valid);
      searchCommitted = false;
      clearTimeout(searchTimer);
      syncControls();
      render();
    });

    syncControls();
    if (location.search) {
      render();
      if (stateParams(state).toString() !== location.search.slice(1)) commit(false);
    }
    controls.dataset.ready = "true";
  })();
}
