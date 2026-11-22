import {
  CATALOGUE_DESCRIPTION,
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
  const filterCount = document.querySelector<HTMLElement>("[data-filter-count]");
  const qInput = document.querySelector<HTMLInputElement>("input[data-q]");
  const sortSelect = document.querySelector<HTMLSelectElement>("select[data-sort]");
  const authorSelect = document.querySelector<HTMLSelectElement>("select[data-author]");
  const typeSelect = document.querySelector<HTMLSelectElement>("select[data-type]");
  if (
    !controls ||
    !grid ||
    !template ||
    !count ||
    !empty ||
    !chipsRow ||
    !chipList ||
    !clear ||
    !reset ||
    !toggle ||
    !panel ||
    !qInput ||
    !sortSelect ||
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

    let state: CatalogueState = readLocation();
    let searchTimer: ReturnType<typeof setTimeout> | undefined;
    let searchCommitted = false;
    let pageCount = 1;

    const canonicalLink = document.querySelector<HTMLLinkElement>('link[rel="canonical"]');
    let configuredOrigin = location.origin;
    if (canonicalLink?.href) {
      try {
        configuredOrigin = new URL(canonicalLink.href).origin;
      } catch {
        configuredOrigin = location.origin;
      }
    }

    function readLocation(): CatalogueState {
      const params = new URLSearchParams(location.search);
      const next = parseState(params, valid);
      if (!params.has("page")) {
        const match = location.pathname.match(/\/pets\/page\/(\d+)\/?$/);
        if (match) next.page = Math.max(1, Number.parseInt(match[1], 10));
      }
      return next;
    }

    function optionText(select: HTMLSelectElement, value: string): string {
      return Array.from(select.options).find((option) => option.value === value)?.text ?? value;
    }

    function categoryLabel(id: string): string {
      for (const chip of document.querySelectorAll<HTMLElement>(
        "[data-category-nav] a[data-cat]",
      )) {
        if ((chip.dataset.cat ?? "") !== id) continue;
        const clone = chip.cloneNode(true) as HTMLElement;
        clone.querySelector(".cat-count")?.remove();
        return clone.textContent?.trim() || id;
      }
      return id;
    }

    function syncFilterCount() {
      if (!filterCount) return;
      const active = (state.author ? 1 : 0) + (state.type ? 1 : 0);
      filterCount.textContent = String(active);
      filterCount.hidden = active === 0;
    }

    function syncControls() {
      qInput!.value = state.q;
      sortSelect!.value = state.sort;
      authorSelect!.value = state.author;
      typeSelect!.value = state.type;
      syncFilterCount();
    }

    function syncCategory() {
      for (const chip of document.querySelectorAll<HTMLElement>(
        "[data-category-nav] a[data-cat]",
      )) {
        if ((chip.dataset.cat ?? "") === state.category) chip.setAttribute("aria-current", "page");
        else chip.removeAttribute("aria-current");
      }
    }

    function canonicalPath(): string {
      return stateParams({ ...state, page: 1 }).toString() === ""
        ? pagePath("/pets/", state.page)
        : `/pets/?${stateParams(state)}`;
    }

    function breadcrumbItems(): { label: string; path: string }[] {
      const items = [{ label: "All pets", path: "/pets/" }];
      if (state.page > 1) items.push({ label: `Page ${state.page}`, path: canonicalPath() });
      return items;
    }

    function syncBreadcrumb() {
      const items = breadcrumbItems();
      let nav = document.querySelector<HTMLElement>('nav[aria-label="Breadcrumb"]');
      if (items.length <= 1) {
        if (nav) {
          nav.hidden = true;
          nav.style.display = "none";
        }
        return;
      }
      if (!nav) {
        nav = document.createElement("nav");
        nav.className = "crumbs";
        nav.setAttribute("aria-label", "Breadcrumb");
        const slot = document.querySelector<HTMLElement>("[data-crumbs]");
        const head = document.querySelector<HTMLElement>(".listing-head");
        if (slot) slot.append(nav);
        else if (head?.parentElement) head.parentElement.insertBefore(nav, head);
        else document.body.prepend(nav);
      }
      nav.hidden = false;
      nav.style.removeProperty("display");
      let list = nav.querySelector<HTMLOListElement>("ol.crumb-list");
      if (!list) {
        list = document.createElement("ol");
        list.className = "crumb-list";
        nav.replaceChildren(list);
      }
      const nodes = items.map((item, index) => {
        const li = document.createElement("li");
        if (index > 0) {
          const separator = document.createElement("span");
          separator.setAttribute("aria-hidden", "true");
          separator.textContent = "/";
          li.append(separator);
        }
        if (index === items.length - 1) {
          const label = document.createElement("span");
          label.setAttribute("aria-current", "page");
          label.textContent = item.label;
          li.append(label);
        } else {
          const link = document.createElement("a");
          link.href = item.path;
          link.textContent = item.label;
          li.append(link);
        }
        return li;
      });
      list.replaceChildren(...nodes);
    }

    function syncDocument() {
      document.title = state.page > 1 ? `Pets · Page ${state.page} · DeskPet` : "Pets · DeskPet";
      const url = configuredOrigin + canonicalPath();
      if (canonicalLink) canonicalLink.href = url;
      const description =
        CATALOGUE_DESCRIPTION + (state.page > 1 ? ` Page ${state.page} of ${pageCount}.` : "");
      for (const meta of document.querySelectorAll<HTMLMetaElement>(
        'meta[name="description"], meta[property="og:description"], meta[name="twitter:description"]',
      )) {
        meta.content = description;
      }
      const ogUrl = document.querySelector<HTMLMetaElement>('meta[property="og:url"]');
      if (ogUrl) ogUrl.content = url;
      const script = document.querySelector<HTMLScriptElement>(
        'script[type="application/ld+json"]',
      );
      if (script) {
        script.textContent = JSON.stringify({
          "@context": "https://schema.org",
          "@type": "BreadcrumbList",
          itemListElement: breadcrumbItems().map((item, index) => ({
            "@type": "ListItem",
            position: index + 1,
            name: item.label,
            item: configuredOrigin + item.path,
          })),
        });
      }
    }

    function commit(push: boolean) {
      const params = stateParams(state).toString();
      const url = `/pets/${params ? `?${params}` : ""}`;
      history[push ? "pushState" : "replaceState"]({}, "", url);
      syncDocument();
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
          label: categoryLabel(state.category),
          clear: () => {
            state.category = "";
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
      if (!pagination) return;
      if (pageCount <= 1) {
        pagination.hidden = true;
        pagination.replaceChildren();
        return;
      }
      pagination.hidden = false;
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
      const status = document.createElement("span");
      status.className = "pagination-status";
      status.textContent = `Page ${state.page} of ${pageCount}`;
      pagination.replaceChildren(status, ...nodes);
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
      syncCategory();
      syncFilterCount();
      syncBreadcrumb();
      syncDocument();
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

    for (const link of document.querySelectorAll<HTMLAnchorElement>(
      "[data-category-nav] a[data-cat]",
    )) {
      link.addEventListener("click", (event) => {
        event.preventDefault();
        state.category = link.dataset.cat ?? "";
        state.page = 1;
        searchCommitted = false;
        clearTimeout(searchTimer);
        render();
        commit(true);
      });
    }

    clear.addEventListener("click", resetAll);
    reset.addEventListener("click", resetAll);

    pagination?.addEventListener("click", (event) => {
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
      state = readLocation();
      searchCommitted = false;
      clearTimeout(searchTimer);
      syncControls();
      render();
    });

    syncControls();
    if (location.search) {
      render();
      if (stateParams(state).toString() !== location.search.slice(1)) commit(false);
    } else {
      pageCount = Math.max(
        1,
        Math.ceil(filterPets(index, state, (pet) => pet.authorSlug).length / PAGE_SIZE),
      );
      syncCategory();
      syncBreadcrumb();
      syncDocument();
    }
    controls.dataset.ready = "true";
  })();
}
