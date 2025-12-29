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
  const toggleButtons = document.querySelectorAll(".toggle-children");
  toggleButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const epicItem = button.closest(".epic-item");
      if (!epicItem)
        return;
      const children = epicItem.querySelector(".epic-children");
      if (children) {
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
      }
    });
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
        try {
          const res = await fetch(`/api/issues/${id}`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ title: newTitle })
          });
          if (!res.ok)
            throw new Error("Update failed");
          const newTitleEl = document.createElement("div");
          newTitleEl.classList.add("issue-title");
          newTitleEl.innerText = newTitle;
          input.replaceWith(newTitleEl);
        } catch (err) {
          console.error(err);
          alert("Failed to update title");
          replaceWithOriginal();
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

// frontend/src/modules/board.ts
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
      columnCheckboxes.forEach((checkbox) => {
        const status = checkbox.getAttribute("data-status");
        if (status)
          newState[status] = checkbox.checked;
      });
      localStorage.setItem("board-column-visibility", JSON.stringify(newState));
    };
    const savedVisibility = localStorage.getItem("board-column-visibility");
    let visibilityState = savedVisibility ? JSON.parse(savedVisibility) : null;
    const columnCheckboxes = columnsDropdown.querySelectorAll('input[type="checkbox"]');
    columnCheckboxes.forEach((checkbox) => {
      const status = checkbox.getAttribute("data-status");
      if (!status)
        return;
      if (visibilityState === null) {
        checkbox.checked = status !== "deferred";
      } else {
        checkbox.checked = visibilityState[status] !== false;
      }
      updateColumnVisibility(status, checkbox.checked);
      checkbox.addEventListener("change", (e) => {
        const isVisible = e.target.checked;
        updateColumnVisibility(status, isVisible);
        saveVisibilityState();
      });
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
  const typeFilters = document.querySelectorAll(".type-filter");
  if (typeFilters.length > 0) {
    const updateCardVisibility = () => {
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
        if (visible) {
          card.classList.remove("hidden-by-type");
        } else {
          card.classList.add("hidden-by-type");
        }
      });
    };
    typeFilters.forEach((filter) => {
      filter.addEventListener("change", updateCardVisibility);
    });
    updateCardVisibility();
  }
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
          try {
            const res = await fetch(`/api/issues/${id}`, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ status: apiStatus })
            });
            if (!res.ok)
              throw new Error("Update failed");
          } catch (err) {
            console.error(err);
            alert("Failed to update status");
            window.location.reload();
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
      selectItem(items[selectedIndex]);
      e.preventDefault();
    } else if (e.key === "k" || e.key === "ArrowUp") {
      selectedIndex = Math.max(selectedIndex - 1, 0);
      selectItem(items[selectedIndex]);
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
      const col = columns[selectedColumnIndex];
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
      const col = columns[selectedColumnIndex];
      const cards = getVisibleCards(col);
      selectedCardIndex = Math.min(selectedCardIndex, Math.max(0, cards.length - 1));
      updateBoardSelection();
      e.preventDefault();
    } else if (e.key === "l" || e.key === "ArrowRight") {
      selectedColumnIndex = Math.min(selectedColumnIndex + 1, columns.length - 1);
      const col = columns[selectedColumnIndex];
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
    const col = columns[selectedColumnIndex];
    const cards = getVisibleCards(col);
    document.querySelectorAll(".issue-card.selected").forEach((el) => el.classList.remove("selected"));
    if (cards.length > 0) {
      selectedCardIndex = Math.max(0, Math.min(selectedCardIndex, cards.length - 1));
      const card = cards[selectedCardIndex];
      card.classList.add("selected");
      card.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }
}

// frontend/src/modules/graph.ts
function initGraph() {
  const treeView = document.querySelector(".tree-view");
  if (!treeView)
    return;
  const nodes = document.querySelectorAll(".tree-node");
  const typeFilters = document.querySelectorAll(".type-filter");
  const expandAllBtn = document.getElementById("expand-all") || document.getElementById("detail-expand");
  const collapseAllBtn = document.getElementById("collapse-all") || document.getElementById("detail-collapse");
  const expandOneLevelBtn = document.getElementById("expand-one-level");
  const collapseOneLevelBtn = document.getElementById("collapse-one-level");
  const expandedNodes = new Set;
  const issueType = treeView.getAttribute("data-issue-type");
  if (issueType === "task") {
    nodes.forEach((node) => {
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
    const activeTypes = typeFilters.length > 0 ? new Set(Array.from(typeFilters).filter((f) => f.checked).map((f) => f.value)) : null;
    nodes.forEach((node) => {
      const id = node.getAttribute("data-id") || "";
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
      if (visible) {
        node.classList.remove("hidden");
      } else {
        node.classList.add("hidden");
      }
    });
  }
  nodes.forEach((node) => {
    const toggleBtn = node.querySelector(".tree-toggle");
    if (toggleBtn) {
      toggleBtn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
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
  });
  typeFilters.forEach((filter) => {
    filter.addEventListener("change", updateVisibility);
  });
  if (expandAllBtn) {
    expandAllBtn.addEventListener("click", () => {
      nodes.forEach((node) => {
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
    });
  }
  if (collapseAllBtn) {
    collapseAllBtn.addEventListener("click", () => {
      expandedNodes.clear();
      nodes.forEach((node) => {
        const toggleBtn = node.querySelector(".tree-toggle");
        if (toggleBtn) {
          toggleBtn.classList.remove("expanded");
          const icon = toggleBtn.querySelector(".toggle-icon");
          if (icon)
            icon.textContent = "+";
        }
      });
      updateVisibility();
    });
  }
  if (expandOneLevelBtn) {
    expandOneLevelBtn.addEventListener("click", () => {
      expandOneLevel();
    });
  }
  if (collapseOneLevelBtn) {
    collapseOneLevelBtn.addEventListener("click", () => {
      collapseOneLevel();
    });
  }
  function expandOneLevel() {
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
  function getCurrentExpandedDepths() {
    const depths = [];
    nodes.forEach((node) => {
      const id = node.getAttribute("data-id");
      const depth = parseInt(node.getAttribute("data-depth") || "0");
      if (id && expandedNodes.has(id)) {
        depths.push(depth);
      }
    });
    return [...new Set(depths)];
  }
  function isParentVisible(node) {
    const parentId = node.getAttribute("data-parent");
    if (!parentId)
      return true;
    const parent = Array.from(nodes).find((n) => n.getAttribute("data-id") === parentId);
    if (!parent)
      return false;
    return !parent.classList.contains("hidden") && isParentVisible(parent);
  }
  updateVisibility();
}

// frontend/src/modules/sorting.ts
function initSorting() {
  const sortButtons = document.querySelectorAll(".sort-btn");
  if (sortButtons.length === 0)
    return;
  sortButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const sortBy = button.getAttribute("data-sort");
      if (sortBy) {
        sortTreeNodes(sortBy);
        updateActiveSortButton(button);
      }
    });
  });
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
  treeList.innerHTML = "";
  nodes.forEach((node) => {
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
  const statusOrder = { open: 0, in_progress: 1, blocked: 2, closed: 3, deferred: 4 };
  const orderA = statusOrder[statusA] ?? 999;
  const orderB = statusOrder[statusB] ?? 999;
  return orderA - orderB;
}
function compareType(typeA, typeB) {
  const typeOrder = { epic: 0, feature: 1, bug: 2, task: 3, chore: 4 };
  const orderA = typeOrder[typeA] ?? 999;
  const orderB = typeOrder[typeB] ?? 999;
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
    button.classList.remove("btn-primary");
    button.classList.add("btn-tertiary");
  });
  activeButton.classList.remove("btn-tertiary");
  activeButton.classList.add("btn-primary");
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
  initSorting();
  console.log("Nacre modular frontend initialized");
});
