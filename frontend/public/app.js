// frontend/src/modules/theme.ts
(function() {
  const stored = localStorage.getItem("theme");
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const theme = stored || (prefersDark ? "dark" : "light");
  document.documentElement.setAttribute("data-theme", theme);
})();
function initTheme() {
  const themeToggle = document.getElementById("theme-toggle");
  if (themeToggle) {
    const updateIcon = () => {
      const current = document.documentElement.getAttribute("data-theme");
      themeToggle.textContent = current === "dark" ? "☀️" : "\uD83C\uDF19";
    };
    updateIcon();
    themeToggle.addEventListener("click", () => {
      const current = document.documentElement.getAttribute("data-theme");
      const next = current === "dark" ? "light" : "dark";
      document.documentElement.setAttribute("data-theme", next);
      localStorage.setItem("theme", next);
      updateIcon();
    });
  }
}

// frontend/src/modules/search.ts
function initSearch() {
  const filterInput = document.getElementById("filter-input");
  if (filterInput) {
    filterInput.addEventListener("input", (e) => {
      const query = e.target.value.toLowerCase();
      const filterableItems = document.querySelectorAll("[data-filter-text]");
      filterableItems.forEach((item) => {
        const text = item.getAttribute("data-filter-text");
        const matches = text && text.includes(query);
        if (item instanceof HTMLElement) {
          if (matches) {
            item.style.display = "";
          } else {
            item.style.display = "none";
          }
        }
      });
    });
  }
}

// frontend/src/modules/list.ts
function initListFeatures() {
  document.addEventListener("click", (e) => {
    const target = e.target;
    const button = target.closest(".toggle-children");
    if (!button)
      return;
    const epicItem = button.closest(".epic-item");
    if (!epicItem)
      return;
    const children = epicItem.querySelector(".epic-children");
    if (!children)
      return;
    const isCollapsed = children.classList.contains("collapsed");
    children.classList.toggle("collapsed");
    button.classList.toggle("expanded");
    if (isCollapsed) {
      children.style.maxHeight = children.scrollHeight + "px";
      children.style.opacity = "1";
    } else {
      children.style.maxHeight = "0";
      children.style.opacity = "0";
    }
  });
}

// frontend/src/modules/toast.ts
class ToastManager {
  container = null;
  activeToasts = new Set;
  initialized = false;
  constructor() {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", () => this.init());
    } else {
      this.init();
    }
  }
  init() {
    if (this.initialized)
      return;
    this.container = document.createElement("div");
    this.container.id = "toast-container";
    this.container.className = "toast-container";
    document.body.appendChild(this.container);
    this.initialized = true;
  }
  createToast(config) {
    const toast = document.createElement("div");
    toast.className = `toast toast-${config.type || "info"}`;
    const message = document.createElement("div");
    message.className = "toast-message";
    message.textContent = config.message;
    const actions = document.createElement("div");
    actions.className = "toast-actions";
    if (config.retryAction) {
      const retryBtn = document.createElement("button");
      retryBtn.className = "toast-retry";
      retryBtn.textContent = "Retry";
      retryBtn.addEventListener("click", async () => {
        retryBtn.disabled = true;
        retryBtn.textContent = "Retrying...";
        try {
          await config.retryAction();
          this.remove(toast);
          this.show({ message: "Success!", type: "success", duration: 2000 });
        } catch {
          retryBtn.disabled = false;
          retryBtn.textContent = "Retry";
        }
      });
      actions.appendChild(retryBtn);
    }
    const closeBtn = document.createElement("button");
    closeBtn.className = "toast-close";
    closeBtn.textContent = "×";
    closeBtn.addEventListener("click", () => this.remove(toast));
    actions.appendChild(closeBtn);
    toast.appendChild(message);
    toast.appendChild(actions);
    return toast;
  }
  show(config) {
    if (!this.initialized)
      return;
    if (!this.container)
      return;
    const toast = this.createToast(config);
    this.container.appendChild(toast);
    this.activeToasts.add(toast);
    setTimeout(() => {
      toast.classList.add("toast-show");
    }, 10);
    if (config.duration && config.duration > 0) {
      setTimeout(() => {
        this.remove(toast);
      }, config.duration);
    }
  }
  remove(toast) {
    toast.classList.remove("toast-show");
    setTimeout(() => {
      if (toast.parentNode) {
        toast.parentNode.removeChild(toast);
      }
      this.activeToasts.delete(toast);
    }, 300);
  }
  clear() {
    this.activeToasts.forEach((toast) => this.remove(toast));
  }
}
var toast = new ToastManager;
function handleError(error, context, retryAction) {
  const message = error instanceof Error ? error.message : "Unknown error occurred";
  console.error(`[${context}] ${message}`, error);
  toast.show({
    message: `${context}: ${message}`,
    type: "error",
    duration: retryAction ? 0 : 5000,
    retryAction
  });
}
function handleNetworkError(response, context, retryAction) {
  const message = `HTTP ${response.status}: ${response.statusText}`;
  console.error(`[${context}] ${message}`, response);
  toast.show({
    message: `${context}: ${message}`,
    type: "error",
    duration: retryAction ? 0 : 5000,
    retryAction
  });
}

// frontend/src/modules/edit.ts
function initInlineEdit() {
  document.addEventListener("click", (e) => {
    const target = e.target;
    if (target.classList.contains("issue-title") && target.closest(".issue-item")) {
      handleTitleEdit(target);
    }
  });
  function handleTitleEdit(titleEl) {
    const currentTitle = titleEl.innerText;
    const input = document.createElement("input");
    input.type = "text";
    input.value = currentTitle;
    input.classList.add("edit-input");
    input.addEventListener("click", (e) => e.stopPropagation());
    titleEl.replaceWith(input);
    input.focus();
    let isSaving = false;
    const save = async () => {
      if (isSaving)
        return;
      isSaving = true;
      const newTitle = input.value.trim();
      const issueItem = input.closest(".issue-item");
      const id = issueItem ? issueItem.getAttribute("data-id") : null;
      if (newTitle && newTitle !== currentTitle && id) {
        const updateTitle = async () => {
          const res = await fetch(`/api/issues/${id}`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ title: newTitle })
          });
          if (!res.ok)
            throw new Error("Update failed");
        };
        try {
          await updateTitle();
          const newTitleEl = document.createElement("div");
          newTitleEl.classList.add("issue-title");
          newTitleEl.innerText = newTitle;
          input.replaceWith(newTitleEl);
        } catch (err) {
          if (err instanceof Error && err.message === "Update failed") {
            handleNetworkError(new Response(null, { status: 500, statusText: "Update failed" }), "Failed to update title", updateTitle);
          } else {
            handleError(err, "Failed to update title", updateTitle);
          }
        }
      } else {
        replaceWithOriginal();
      }
    };
    const replaceWithOriginal = () => {
      const originalTitleEl = document.createElement("div");
      originalTitleEl.classList.add("issue-title");
      originalTitleEl.innerText = currentTitle;
      input.replaceWith(originalTitleEl);
    };
    input.addEventListener("blur", save);
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        input.blur();
      } else if (e.key === "Escape") {
        replaceWithOriginal();
        isSaving = true;
      }
    });
  }
}

// frontend/src/constants.ts
var STATUS = {
  OPEN: "open",
  IN_PROGRESS: "in_progress",
  BLOCKED: "blocked",
  CLOSED: "closed",
  DEFERRED: "deferred"
};
var STATUS_ORDER = {
  [STATUS.OPEN]: 0,
  [STATUS.IN_PROGRESS]: 1,
  [STATUS.BLOCKED]: 2,
  [STATUS.CLOSED]: 3,
  [STATUS.DEFERRED]: 4
};
var ISSUE_TYPE = {
  EPIC: "epic",
  FEATURE: "feature",
  BUG: "bug",
  TASK: "task",
  CHORE: "chore"
};
var TYPE_ORDER = {
  [ISSUE_TYPE.EPIC]: 0,
  [ISSUE_TYPE.FEATURE]: 1,
  [ISSUE_TYPE.BUG]: 2,
  [ISSUE_TYPE.TASK]: 3,
  [ISSUE_TYPE.CHORE]: 4
};

// frontend/src/modules/board.ts
var VALID_STATUSES = Object.values(STATUS);
function isValidStatus(value) {
  return VALID_STATUSES.includes(value);
}
function initBoardFeatures() {
  const columnsToggle = document.getElementById("columns-toggle");
  const columnsDropdown = document.getElementById("columns-dropdown");
  if (columnsToggle && columnsDropdown) {
    let updateColumnVisibility = function(status, isVisible) {
      const column = document.querySelector(`.board-column[data-status="${status}"]`);
      if (column) {
        column.style.display = isVisible ? "" : "none";
      }
    }, saveVisibilityState = function() {
      const newState = {};
      getColumnCheckboxes().forEach((checkbox) => {
        const status = checkbox.getAttribute("data-status");
        if (status && isValidStatus(status)) {
          newState[status] = checkbox.checked;
        }
      });
      localStorage.setItem("board-column-visibility", JSON.stringify(newState));
    };
    const savedVisibility = localStorage.getItem("board-column-visibility");
    const visibilityState = savedVisibility ? JSON.parse(savedVisibility) : null;
    const getColumnCheckboxes = () => columnsDropdown.querySelectorAll('input[type="checkbox"]');
    getColumnCheckboxes().forEach((checkbox) => {
      const status = checkbox.getAttribute("data-status");
      if (!status || !isValidStatus(status))
        return;
      if (visibilityState === null) {
        checkbox.checked = status !== STATUS.DEFERRED;
      } else {
        checkbox.checked = visibilityState[status] !== false;
      }
      updateColumnVisibility(status, checkbox.checked);
    });
    columnsDropdown.addEventListener("change", (e) => {
      const target = e.target;
      if (target.type !== "checkbox")
        return;
      const status = target.getAttribute("data-status");
      if (!status)
        return;
      updateColumnVisibility(status, target.checked);
      saveVisibilityState();
    });
    columnsToggle.addEventListener("click", (e) => {
      e.stopPropagation();
      columnsDropdown.classList.toggle("show");
    });
    document.addEventListener("click", (e) => {
      if (!columnsDropdown.contains(e.target) && e.target !== columnsToggle) {
        columnsDropdown.classList.remove("show");
      }
    });
    columnsDropdown.addEventListener("click", (e) => {
      e.stopPropagation();
    });
  }
  const updateCardVisibility = () => {
    const typeFilters = document.querySelectorAll(".type-filter");
    if (typeFilters.length === 0)
      return;
    const activeTypes = new Set(Array.from(typeFilters).filter((f) => f.checked).map((f) => f.value));
    const cards = document.querySelectorAll(".issue-card");
    cards.forEach((card) => {
      let visible = false;
      for (const type of activeTypes) {
        if (card.classList.contains(`issue-type-${type}`)) {
          visible = true;
          break;
        }
      }
      card.classList.toggle("hidden-by-type", !visible);
    });
  };
  document.addEventListener("change", (e) => {
    const target = e.target;
    if (target.classList.contains("type-filter")) {
      updateCardVisibility();
    }
  });
  updateCardVisibility();
}

// frontend/src/modules/dragdrop.ts
function initDragAndDrop() {
  const draggables = document.querySelectorAll('.issue-card[draggable="true"]');
  const droppables = document.querySelectorAll(".column-content");
  if (draggables.length > 0 && droppables.length > 0) {
    draggables.forEach((draggable) => {
      draggable.addEventListener("dragstart", () => {
        draggable.classList.add("dragging");
        draggable.style.opacity = "0.5";
      });
      draggable.addEventListener("dragend", () => {
        draggable.classList.remove("dragging");
        draggable.style.opacity = "1";
      });
    });
    droppables.forEach((droppable) => {
      droppable.addEventListener("dragover", (e) => {
        e.preventDefault();
        const dragging = document.querySelector(".dragging");
        if (dragging) {
          const afterElement = getDragAfterElement(droppable, e.clientY);
          if (afterElement == null) {
            droppable.appendChild(dragging);
          } else {
            droppable.insertBefore(dragging, afterElement);
          }
        }
      });
      droppable.addEventListener("drop", async () => {
        const draggable = document.querySelector(".dragging");
        if (!draggable)
          return;
        const apiStatus = droppable.getAttribute("data-status");
        const id = draggable.getAttribute("data-id");
        if (id && apiStatus) {
          const updateStatus = async () => {
            const res = await fetch(`/api/issues/${id}`, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ status: apiStatus })
            });
            if (!res.ok)
              throw new Error("Update failed");
          };
          try {
            await updateStatus();
          } catch (err) {
            if (err instanceof Error && err.message === "Update failed") {
              handleNetworkError(new Response(null, { status: 500, statusText: "Update failed" }), "Failed to update status", updateStatus);
            } else {
              handleError(err, "Failed to update status", updateStatus);
            }
          }
        }
      });
    });
  }
  function getDragAfterElement(container, y) {
    const draggableElements = [...container.querySelectorAll(".issue-card:not(.dragging)")];
    const result = draggableElements.reduce((closest, child) => {
      const box = child.getBoundingClientRect();
      const offset = y - box.top - box.height / 2;
      if (offset < 0 && offset > closest.offset) {
        return { offset, element: child };
      } else {
        return closest;
      }
    }, { offset: Number.NEGATIVE_INFINITY, element: null });
    return result.element;
  }
}

// frontend/src/modules/navigation.ts
function initNavigation() {
  let selectedIndex = -1;
  let selectedColumnIndex = 0;
  let selectedCardIndex = 0;
  const isBoard = document.querySelector(".board") !== null;
  const isList = document.querySelector(".issue-list") !== null;
  if (isBoard) {
    updateBoardSelection();
  }
  document.addEventListener("keydown", (e) => {
    const target = e.target;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA")
      return;
    if (e.key === "Backspace") {
      e.preventDefault();
      window.history.back();
      return;
    }
    if (isList) {
      handleListNavigation(e);
    } else if (isBoard) {
      handleBoardNavigation(e);
    }
  });
  function handleListNavigation(e) {
    const items = Array.from(document.querySelectorAll('.issue-item:not([style*="display: none"])'));
    if (items.length === 0)
      return;
    const current = document.querySelector(".issue-item.selected");
    if (current) {
      selectedIndex = items.indexOf(current);
    }
    if (e.key === "j" || e.key === "ArrowDown") {
      selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
      const item = items.at(selectedIndex);
      if (item)
        selectItem(item);
      e.preventDefault();
    } else if (e.key === "k" || e.key === "ArrowUp") {
      selectedIndex = Math.max(selectedIndex - 1, 0);
      const item = items.at(selectedIndex);
      if (item)
        selectItem(item);
      e.preventDefault();
    } else if (e.key === "Enter" || e.key === "o") {
      if (current) {
        const link = current.querySelector(".issue-meta a");
        if (link)
          link.click();
      }
    }
  }
  function handleBoardNavigation(e) {
    const columns = Array.from(document.querySelectorAll('.board-column:not([style*="display: none"])'));
    if (columns.length === 0)
      return;
    if (e.key === "j" || e.key === "ArrowDown") {
      const col = columns.at(selectedColumnIndex);
      if (!col)
        return;
      const cards = getVisibleCards(col);
      if (cards.length > 0) {
        selectedCardIndex = Math.min(selectedCardIndex + 1, cards.length - 1);
        updateBoardSelection();
        e.preventDefault();
      }
    } else if (e.key === "k" || e.key === "ArrowUp") {
      selectedCardIndex = Math.max(selectedCardIndex - 1, 0);
      updateBoardSelection();
      e.preventDefault();
    } else if (e.key === "h" || e.key === "ArrowLeft") {
      selectedColumnIndex = Math.max(selectedColumnIndex - 1, 0);
      const col = columns.at(selectedColumnIndex);
      if (!col)
        return;
      const cards = getVisibleCards(col);
      selectedCardIndex = Math.min(selectedCardIndex, Math.max(0, cards.length - 1));
      updateBoardSelection();
      e.preventDefault();
    } else if (e.key === "l" || e.key === "ArrowRight") {
      selectedColumnIndex = Math.min(selectedColumnIndex + 1, columns.length - 1);
      const col = columns.at(selectedColumnIndex);
      if (!col)
        return;
      const cards = getVisibleCards(col);
      selectedCardIndex = Math.min(selectedCardIndex, Math.max(0, cards.length - 1));
      updateBoardSelection();
      e.preventDefault();
    } else if (e.key === "Enter" || e.key === "o") {
      const selected = document.querySelector(".issue-card.selected");
      if (selected) {
        const link = selected.querySelector("a");
        if (link)
          link.click();
      }
    }
  }
  function getVisibleCards(column) {
    if (!column)
      return [];
    return Array.from(column.querySelectorAll('.issue-card:not([style*="display: none"])'));
  }
  function selectItem(item) {
    document.querySelectorAll(".issue-item.selected").forEach((el) => el.classList.remove("selected"));
    if (item) {
      item.classList.add("selected");
      item.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }
  function updateBoardSelection() {
    const columns = Array.from(document.querySelectorAll('.board-column:not([style*="display: none"])'));
    if (columns.length === 0)
      return;
    selectedColumnIndex = Math.max(0, Math.min(selectedColumnIndex, columns.length - 1));
    const col = columns.at(selectedColumnIndex);
    if (!col)
      return;
    const cards = getVisibleCards(col);
    document.querySelectorAll(".issue-card.selected").forEach((el) => el.classList.remove("selected"));
    if (cards.length > 0) {
      selectedCardIndex = Math.max(0, Math.min(selectedCardIndex, cards.length - 1));
      const card = cards.at(selectedCardIndex);
      if (card) {
        card.classList.add("selected");
        card.scrollIntoView({ behavior: "smooth", block: "nearest" });
      }
    }
  }
}

// frontend/src/modules/graph.ts
function initGraph() {
  const treeView = document.querySelector(".tree-view");
  if (!treeView)
    return;
  const treeList = treeView.querySelector(".tree-list");
  const controlsContainer = document.querySelector(".controls-grid") || document.querySelector(".child-expand-controls");
  const expandedNodes = new Set;
  const getNodes = () => treeView.querySelectorAll(".tree-node");
  const getTypeFilters = () => document.querySelectorAll(".type-filter");
  const issueType = treeView.getAttribute("data-issue-type");
  if (issueType === ISSUE_TYPE.TASK) {
    getNodes().forEach((node) => {
      const hasChildren = node.getAttribute("data-has-children") === "true";
      if (hasChildren) {
        const id = node.getAttribute("data-id");
        if (id) {
          expandedNodes.add(id);
          const toggleBtn = node.querySelector(".tree-toggle");
          if (toggleBtn) {
            toggleBtn.classList.add("expanded");
            const icon = toggleBtn.querySelector(".toggle-icon");
            if (icon)
              icon.textContent = "−";
          }
        }
      }
    });
  }
  function updateVisibility() {
    const typeFilters = getTypeFilters();
    const activeTypes = typeFilters.length > 0 ? new Set(Array.from(typeFilters).filter((f) => f.checked).map((f) => f.value)) : null;
    getNodes().forEach((node) => {
      const parentId = node.getAttribute("data-parent") || "";
      const type = node.getAttribute("data-type") || "";
      let visible = false;
      if (!parentId) {
        visible = true;
      } else if (expandedNodes.has(parentId)) {
        visible = true;
      }
      if (visible && activeTypes && !activeTypes.has(type)) {
        visible = false;
      }
      node.classList.toggle("hidden", !visible);
    });
  }
  if (treeList) {
    treeList.addEventListener("click", (e) => {
      const target = e.target;
      const toggleBtn = target.closest(".tree-toggle");
      if (!toggleBtn)
        return;
      e.preventDefault();
      e.stopPropagation();
      const node = toggleBtn.closest(".tree-node");
      if (!node)
        return;
      const id = node.getAttribute("data-id");
      if (!id)
        return;
      if (expandedNodes.has(id)) {
        expandedNodes.delete(id);
        toggleBtn.classList.remove("expanded");
        const icon = toggleBtn.querySelector(".toggle-icon");
        if (icon)
          icon.textContent = "+";
      } else {
        expandedNodes.add(id);
        toggleBtn.classList.add("expanded");
        const icon = toggleBtn.querySelector(".toggle-icon");
        if (icon)
          icon.textContent = "−";
      }
      updateVisibility();
    });
  }
  document.addEventListener("change", (e) => {
    const target = e.target;
    if (target.classList.contains("type-filter")) {
      updateVisibility();
    }
  });
  if (controlsContainer) {
    controlsContainer.addEventListener("click", (e) => {
      const target = e.target;
      const button = target.closest("button");
      if (!button)
        return;
      const id = button.id;
      if (id === "expand-all" || id === "detail-expand") {
        expandAll();
      } else if (id === "collapse-all" || id === "detail-collapse") {
        collapseAll();
      } else if (id === "expand-one-level") {
        expandOneLevel();
      } else if (id === "collapse-one-level") {
        collapseOneLevel();
      }
    });
  }
  function expandAll() {
    getNodes().forEach((node) => {
      const hasChildren = node.getAttribute("data-has-children") === "true";
      if (hasChildren) {
        const id = node.getAttribute("data-id");
        if (id) {
          expandedNodes.add(id);
          const toggleBtn = node.querySelector(".tree-toggle");
          if (toggleBtn) {
            toggleBtn.classList.add("expanded");
            const icon = toggleBtn.querySelector(".toggle-icon");
            if (icon)
              icon.textContent = "−";
          }
        }
      }
    });
    updateVisibility();
  }
  function collapseAll() {
    expandedNodes.clear();
    getNodes().forEach((node) => {
      const toggleBtn = node.querySelector(".tree-toggle");
      if (toggleBtn) {
        toggleBtn.classList.remove("expanded");
        const icon = toggleBtn.querySelector(".toggle-icon");
        if (icon)
          icon.textContent = "+";
      }
    });
    updateVisibility();
  }
  function expandOneLevel() {
    const nodes = getNodes();
    let currentMaxExpandedDepth = -1;
    nodes.forEach((node) => {
      const id = node.getAttribute("data-id");
      if (id && expandedNodes.has(id)) {
        const depth = parseInt(node.getAttribute("data-depth") || "0");
        if (depth > currentMaxExpandedDepth) {
          currentMaxExpandedDepth = depth;
        }
      }
    });
    const targetDepth = currentMaxExpandedDepth + 1;
    const nodesToExpand = [];
    nodes.forEach((node) => {
      const depth = parseInt(node.getAttribute("data-depth") || "0");
      const id = node.getAttribute("data-id");
      const hasChildren = node.getAttribute("data-has-children") === "true";
      const parentId = node.getAttribute("data-parent");
      if (id && hasChildren && !expandedNodes.has(id)) {
        if (depth <= targetDepth) {
          const parentExpanded = !parentId || expandedNodes.has(parentId);
          if (parentExpanded) {
            nodesToExpand.push({ id, element: node });
          }
        }
      }
    });
    nodesToExpand.forEach(({ id, element }) => {
      expandedNodes.add(id);
      const toggleBtn = element.querySelector(".tree-toggle");
      if (toggleBtn) {
        toggleBtn.classList.add("expanded");
        const icon = toggleBtn.querySelector(".toggle-icon");
        if (icon)
          icon.textContent = "−";
      }
    });
    updateVisibility();
  }
  function collapseOneLevel() {
    const nodes = getNodes();
    let maxExpandedDepth = 0;
    expandedNodes.forEach((id) => {
      nodes.forEach((node) => {
        if (node.getAttribute("data-id") === id) {
          const depth = parseInt(node.getAttribute("data-depth") || "0");
          maxExpandedDepth = Math.max(maxExpandedDepth, depth);
        }
      });
    });
    nodes.forEach((node) => {
      const depth = parseInt(node.getAttribute("data-depth") || "0");
      const id = node.getAttribute("data-id");
      const hasChildren = node.getAttribute("data-has-children") === "true";
      if (id && hasChildren && depth > maxExpandedDepth - 1) {
        expandedNodes.delete(id);
        const toggleBtn = node.querySelector(".tree-toggle");
        if (toggleBtn) {
          toggleBtn.classList.remove("expanded");
          const icon = toggleBtn.querySelector(".toggle-icon");
          if (icon)
            icon.textContent = "+";
        }
      }
    });
    updateVisibility();
  }
  updateVisibility();
}

// frontend/src/modules/dependency-graph.ts
function initDependencyGraph() {
  const container = document.querySelector(".graph-tree-container");
  const epicSelect = document.getElementById("epic-select");
  if (!container)
    return;
  if (epicSelect) {
    epicSelect.addEventListener("change", () => {
      const epicId = epicSelect.value;
      if (epicId) {
        window.location.href = `/graph/${epicId}`;
      } else {
        window.location.href = "/graph";
      }
    });
  }
  container.addEventListener("click", (e) => {
    const toggle = e.target.closest(".tree-toggle");
    if (!toggle)
      return;
    const node = toggle.closest(".tree-node");
    const nodeId = node?.dataset.id;
    if (!nodeId)
      return;
    const isExpanded = toggle.classList.toggle("expanded");
    toggleDirectChildren(nodeId, isExpanded);
  });
}
function toggleDirectChildren(parentId, show) {
  document.querySelectorAll(`[data-parent="${parentId}"]`).forEach((child) => {
    child.classList.toggle("hidden", !show);
    if (!show) {
      const childId = child.dataset.id;
      if (childId) {
        const toggle = child.querySelector(".tree-toggle");
        if (toggle?.classList.contains("expanded")) {
          toggle.classList.remove("expanded");
          toggleDirectChildren(childId, false);
        }
      }
    }
  });
}

// frontend/src/modules/sorting.ts
var VALID_SORT_KEYS = ["status", "type", "priority"];
function isValidSortKey(key) {
  return VALID_SORT_KEYS.includes(key);
}
function initSorting() {
  document.addEventListener("click", (e) => {
    const target = e.target;
    const button = target.closest(".sort-btn");
    if (!button)
      return;
    if (button.classList.contains("active")) {
      restoreTreeView();
      button.classList.remove("active");
      return;
    }
    const sortBy = button.getAttribute("data-sort");
    if (sortBy && isValidSortKey(sortBy)) {
      sortTreeNodes(sortBy);
      updateActiveSortButton(button);
    }
  });
}
function restoreTreeView() {
  window.location.reload();
}
function sortTreeNodes(sortBy) {
  const treeList = document.querySelector(".tree-list");
  if (!treeList)
    return;
  const nodes = Array.from(treeList.querySelectorAll(".tree-node"));
  const expandedStates = new Map;
  nodes.forEach((node) => {
    const toggleBtn = node.querySelector(".tree-toggle");
    if (toggleBtn && toggleBtn.classList.contains("expanded")) {
      expandedStates.set(node.dataset.id, true);
    }
  });
  nodes.sort((a, b) => compareNodes(a, b, sortBy));
  treeList.classList.add("sorting-active");
  const expandButtons = document.querySelectorAll("#expand-all, #collapse-all, #expand-one-level, #collapse-one-level");
  expandButtons.forEach((btn) => {
    btn.disabled = true;
  });
  treeList.innerHTML = "";
  nodes.forEach((node) => {
    node.classList.remove("hidden");
    treeList.appendChild(node);
    if (expandedStates.has(node.dataset.id)) {
      const toggleBtn = node.querySelector(".tree-toggle");
      if (toggleBtn) {
        toggleBtn.classList.add("expanded");
        const icon = toggleBtn.querySelector(".toggle-icon");
        if (icon) {
          icon.textContent = "−";
        }
      }
    }
  });
}
function compareNodes(a, b, sortBy) {
  switch (sortBy) {
    case "status":
      return compareStatus(a.dataset.status, b.dataset.status);
    case "type":
      return compareType(a.dataset.type, b.dataset.type);
    case "priority":
      return comparePriority(a.dataset.priority, b.dataset.priority);
    default:
      return 0;
  }
}
function compareStatus(statusA, statusB) {
  const orderA = STATUS_ORDER[statusA] ?? 999;
  const orderB = STATUS_ORDER[statusB] ?? 999;
  return orderA - orderB;
}
function compareType(typeA, typeB) {
  const orderA = TYPE_ORDER[typeA] ?? 999;
  const orderB = TYPE_ORDER[typeB] ?? 999;
  return orderA - orderB;
}
function comparePriority(priorityA, priorityB) {
  const prioA = parseInt(priorityA) || 999;
  const prioB = parseInt(priorityB) || 999;
  return prioA - prioB;
}
function updateActiveSortButton(activeButton) {
  const sortButtons = document.querySelectorAll(".sort-btn");
  sortButtons.forEach((button) => {
    button.classList.remove("active");
  });
  activeButton.classList.add("active");
}

// frontend/src/main.ts
document.addEventListener("DOMContentLoaded", () => {
  initTheme();
  initSearch();
  initListFeatures();
  initInlineEdit();
  initBoardFeatures();
  initDragAndDrop();
  initNavigation();
  initGraph();
  initDependencyGraph();
  initSorting();
});
