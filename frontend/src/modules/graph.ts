import { gsap } from 'gsap';
import { ISSUE_TYPE } from '../constants';

function initOrgTreeConnectors() {
    const orgTree = document.querySelector('.org-tree');
    if (!orgTree) return;

    // Create SVG overlay for Bezier curves
    let svg = orgTree.querySelector('.org-tree-svg') as SVGSVGElement;
    if (!svg) {
        svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.classList.add('org-tree-svg');
        svg.style.position = 'absolute';
        svg.style.top = '0';
        svg.style.left = '0';
        svg.style.width = '100%';
        svg.style.height = '100%';
        svg.style.pointerEvents = 'none';
        svg.style.overflow = 'visible';

        // Make org-tree position relative for absolute SVG positioning
        (orgTree as HTMLElement).style.position = 'relative';
        orgTree.insertBefore(svg, orgTree.firstChild);
    }

    // Maps to track paths for hover effects (parent→children and child→parent)
    const pathsByParent = new Map<HTMLElement, { path: SVGPathElement; childNode: HTMLElement; hitArea: SVGPathElement }[]>();
    const pathsByChild = new Map<HTMLElement, { path: SVGPathElement; parentNode: HTMLElement }>();
    let isInitialDraw = true;

    // Track node event listeners for cleanup on redraw
    let hoverCleanups: (() => void)[] = [];

    function drawConnectors() {
        // Kill any in-flight GSAP animations on existing paths
        svg.querySelectorAll('path').forEach(p => gsap.killTweensOf(p));

        // Remove previous node event listeners
        hoverCleanups.forEach(fn => fn());
        hoverCleanups = [];

        // Clear existing paths and maps
        svg.innerHTML = '';
        pathsByParent.clear();
        pathsByChild.clear();

        // Get computed style for colors
        const style = getComputedStyle(document.documentElement);
        const strokeColor = style.getPropertyValue('--connector-color').trim()
            || style.getPropertyValue('--border-color').trim()
            || '#585b70';

        // Collect all paths with their tree depth for staggered animation
        const allPaths: { path: SVGPathElement; depth: number }[] = [];

        // Find all nodes with children
        const nodes = orgTree.querySelectorAll('li');

        nodes.forEach(li => {
            const parentNode = li.querySelector(':scope > a.org-node, :scope > .org-node') as HTMLElement;
            const childrenUl = li.querySelector(':scope > ul');

            if (!parentNode || !childrenUl) return;

            const children = childrenUl.querySelectorAll(':scope > li');
            if (children.length === 0) return;

            // Calculate tree depth by counting ancestor <ul> elements
            let depth = 0;
            let el: Element | null = li;
            while (el && el !== orgTree) {
                if (el.tagName === 'UL') depth++;
                el = el.parentElement;
            }

            // Get parent node position (bottom center)
            const parentRect = parentNode.getBoundingClientRect();
            const treeRect = orgTree.getBoundingClientRect();

            const parentX = parentRect.left + parentRect.width / 2 - treeRect.left;
            const parentY = parentRect.bottom - treeRect.top;

            // Initialize array for this parent's paths
            const parentPaths: { path: SVGPathElement; childNode: HTMLElement; hitArea: SVGPathElement }[] = [];

            children.forEach(childLi => {
                const childNode = childLi.querySelector(':scope > a.org-node, :scope > .org-node') as HTMLElement;
                if (!childNode) return;

                // Get child node position (top center)
                const childRect = childNode.getBoundingClientRect();
                const childX = childRect.left + childRect.width / 2 - treeRect.left;
                const childY = childRect.top - treeRect.top;

                // Calculate control points for smooth S-curve
                const midY = (parentY + childY) / 2;

                // Create cubic Bezier path
                const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
                const d = `M ${parentX} ${parentY} C ${parentX} ${midY}, ${childX} ${midY}, ${childX} ${childY}`;

                // Invisible wider hit-area path for easier mouse targeting
                const hitArea = document.createElementNS('http://www.w3.org/2000/svg', 'path');
                hitArea.setAttribute('d', d);
                hitArea.setAttribute('fill', 'none');
                hitArea.setAttribute('stroke', 'transparent');
                hitArea.setAttribute('stroke-width', '14');
                hitArea.setAttribute('stroke-linecap', 'round');
                hitArea.classList.add('connector-hit-area');
                svg.appendChild(hitArea);

                // Visible connector path
                path.setAttribute('d', d);
                path.setAttribute('fill', 'none');
                path.setAttribute('stroke', strokeColor);
                path.setAttribute('stroke-width', '2');
                path.setAttribute('stroke-linecap', 'round');
                path.style.pointerEvents = 'none';

                svg.appendChild(path);
                parentPaths.push({ path, childNode, hitArea });
                pathsByChild.set(childNode, { path, parentNode });
                allPaths.push({ path, depth });
            });

            pathsByParent.set(parentNode, parentPaths);
        });

        // Animate draw-in on initial load, instant on resize/scroll redraws
        if (isInitialDraw) {
            isInitialDraw = false;
            if (allPaths.length > 0) {
                animateDrawIn(allPaths);
            }
            animateNodeEntrance(orgTree as HTMLElement);
        }

        // Setup hover effects
        setupHoverEffects();
    }

    function animateDrawIn(allPaths: { path: SVGPathElement; depth: number }[]) {
        // Sort by depth so parent connectors draw before child connectors
        allPaths.sort((a, b) => a.depth - b.depth);

        allPaths.forEach(({ path }, index) => {
            const length = path.getTotalLength();
            // Set initial state: fully hidden via dash offset
            path.setAttribute('stroke-dasharray', String(length));
            path.setAttribute('stroke-dashoffset', String(length));

            gsap.to(path, {
                strokeDashoffset: 0,
                duration: 0.4,
                delay: index * 0.06,
                ease: 'power2.out',
                onComplete: () => {
                    // Clean up dash attributes after animation
                    path.removeAttribute('stroke-dasharray');
                    path.removeAttribute('stroke-dashoffset');
                },
            });
        });
    }

    function animateNodeEntrance(tree: HTMLElement) {
        // Collect all org-nodes grouped by depth for staggered entrance
        const nodesByDepth: HTMLElement[][] = [];

        tree.querySelectorAll('li').forEach(li => {
            const node = li.querySelector(':scope > a.org-node, :scope > .org-node') as HTMLElement;
            if (!node) return;

            // Calculate depth by counting ancestor <ul> elements
            let depth = 0;
            let el: Element | null = li;
            while (el && el !== tree) {
                if (el.tagName === 'UL') depth++;
                el = el.parentElement;
            }

            if (!nodesByDepth[depth]) nodesByDepth[depth] = [];
            nodesByDepth[depth]!.push(node);
        });

        // Animate each depth level with stagger, cascading from root
        let cumulativeDelay = 0;
        nodesByDepth.forEach(nodes => {
            if (!nodes || nodes.length === 0) return;

            gsap.fromTo(nodes,
                { opacity: 0, y: 15, scale: 0.95 },
                {
                    opacity: 1, y: 0, scale: 1,
                    duration: 0.35,
                    stagger: 0.04,
                    delay: cumulativeDelay,
                    ease: 'power2.out',
                    clearProps: 'transform',
                },
            );

            // Next level starts after this level begins (overlap for fluidity)
            cumulativeDelay += 0.1 + nodes.length * 0.02;
        });
    }

    function setupHoverEffects() {
        const style = getComputedStyle(document.documentElement);
        const accentColor = style.getPropertyValue('--accent').trim() || '#fab387';
        const strokeColor = style.getPropertyValue('--connector-color').trim()
            || style.getPropertyValue('--border-color').trim()
            || '#585b70';

        // Helper to register a removable event listener
        function on(el: EventTarget, event: string, handler: EventListener) {
            el.addEventListener(event, handler);
            hoverCleanups.push(() => el.removeEventListener(event, handler));
        }

        // Parent node hover: highlight all child connectors + child nodes
        pathsByParent.forEach((paths, parentNode) => {
            on(parentNode, 'mouseenter', () => {
                paths.forEach(({ path, childNode }) => {
                    gsap.to(path, { stroke: accentColor, strokeWidth: 3, duration: 0.2, overwrite: true });
                    childNode.classList.add('org-node-highlight');
                });
            });

            on(parentNode, 'mouseleave', () => {
                paths.forEach(({ path, childNode }) => {
                    gsap.to(path, { stroke: strokeColor, strokeWidth: 2, duration: 0.2, overwrite: true });
                    childNode.classList.remove('org-node-highlight');
                });
            });
        });

        // Child node hover: highlight the connector going up to parent
        pathsByChild.forEach(({ path, parentNode }, childNode) => {
            on(childNode, 'mouseenter', () => {
                gsap.to(path, { stroke: accentColor, strokeWidth: 3, duration: 0.2, overwrite: true });
                parentNode.classList.add('org-node-highlight');
            });

            on(childNode, 'mouseleave', () => {
                gsap.to(path, { stroke: strokeColor, strokeWidth: 2, duration: 0.2, overwrite: true });
                parentNode.classList.remove('org-node-highlight');
            });
        });

        // Node micro-interactions: subtle scale + elevation on hover, click press
        const allNodes = (orgTree as HTMLElement).querySelectorAll('.org-node') as NodeListOf<HTMLElement>;
        allNodes.forEach(node => {
            on(node, 'mouseenter', () => {
                gsap.to(node, {
                    scale: 1.05,
                    boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
                    duration: 0.2,
                    ease: 'power2.out',
                    overwrite: true,
                });
            });

            on(node, 'mouseleave', () => {
                gsap.to(node, {
                    scale: 1,
                    boxShadow: 'var(--shadow-sm)',
                    duration: 0.2,
                    ease: 'power2.inOut',
                    overwrite: true,
                });
            });

            on(node, 'click', () => {
                gsap.timeline()
                    .to(node, { scale: 0.97, duration: 0.08, ease: 'power2.in' })
                    .to(node, { scale: 1.05, duration: 0.15, ease: 'power2.out' });
            });
        });

        // Direct connector hover + click via hit-area paths
        pathsByParent.forEach((paths, parentNode) => {
            paths.forEach(({ path, childNode, hitArea }) => {
                // Hover: thicken + color shift, highlight both connected nodes
                on(hitArea, 'mouseenter', () => {
                    gsap.to(path, { stroke: accentColor, strokeWidth: 3, duration: 0.2, overwrite: true });
                    parentNode.classList.add('org-node-highlight');
                    childNode.classList.add('org-node-highlight');
                });

                on(hitArea, 'mouseleave', () => {
                    gsap.to(path, { stroke: strokeColor, strokeWidth: 2, duration: 0.2, overwrite: true });
                    parentNode.classList.remove('org-node-highlight');
                    childNode.classList.remove('org-node-highlight');
                });

                // Click: pulse feedback (pop stroke-width then ease back)
                on(hitArea, 'click', () => {
                    gsap.timeline()
                        .to(path, { strokeWidth: 5, stroke: accentColor, duration: 0.1, ease: 'power2.out' })
                        .to(path, { strokeWidth: 2, stroke: strokeColor, duration: 0.4, ease: 'power2.inOut' });
                });
            });
        });
    }

    // Initial draw
    drawConnectors();

    // Redraw on resize (no animation, instant repositioning)
    let resizeTimeout: number;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimeout);
        resizeTimeout = window.setTimeout(drawConnectors, 100);
    });

    // Redraw on scroll (for scrollable containers)
    orgTree.addEventListener('scroll', drawConnectors);

    // Observe DOM changes (for dynamic content)
    const observer = new MutationObserver(() => {
        requestAnimationFrame(drawConnectors);
    });
    observer.observe(orgTree, { childList: true, subtree: true });
}

function initEpicSelector() {
    const wrapper = document.querySelector('.epic-selector-wrapper');
    if (!wrapper) return;

    const selector = wrapper.querySelector('.epic-selector') as HTMLElement;
    const leftBtn = wrapper.querySelector('#epic-nav-left') as HTMLButtonElement;
    const rightBtn = wrapper.querySelector('#epic-nav-right') as HTMLButtonElement;

    if (!selector || !leftBtn || !rightBtn) return;

    const scrollAmount = 200; // pixels to scroll per click

    const updateButtonStates = () => {
        const { scrollLeft, scrollWidth, clientWidth } = selector;
        leftBtn.disabled = scrollLeft <= 0;
        rightBtn.disabled = scrollLeft + clientWidth >= scrollWidth - 1;
    };

    leftBtn.addEventListener('click', () => {
        selector.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
    });

    rightBtn.addEventListener('click', () => {
        selector.scrollBy({ left: scrollAmount, behavior: 'smooth' });
    });

    selector.addEventListener('scroll', updateButtonStates);
    window.addEventListener('resize', updateButtonStates);

    // Initial state
    updateButtonStates();
}

export function initGraph() {
    // Initialize epic selector pagination
    initEpicSelector();

    // Initialize SVG Bezier connectors for org-tree
    initOrgTreeConnectors();

    const treeView = document.querySelector('.tree-view');
    if (!treeView) return;

    const treeList = treeView.querySelector('.tree-list');
    const controlsContainer = document.querySelector('.controls-grid') || document.querySelector('.child-expand-controls');
    const expandedNodes = new Set<string>();

    // Helper to get current nodes (supports dynamic DOM)
    const getNodes = () => treeView.querySelectorAll('.tree-node') as NodeListOf<HTMLElement>;
    const getTypeFilters = () => document.querySelectorAll('.type-filter') as NodeListOf<HTMLInputElement>;

    // Initial expansion for task view
    const issueType = treeView.getAttribute('data-issue-type');
    if (issueType === ISSUE_TYPE.TASK) {
        getNodes().forEach(node => {
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            if (hasChildren) {
                const id = node.getAttribute('data-id');
                if (id) {
                    expandedNodes.add(id);
                    const toggleBtn = node.querySelector('.tree-toggle');
                    if (toggleBtn) {
                        toggleBtn.classList.add('expanded');
                        const icon = toggleBtn.querySelector('.toggle-icon');
                        if (icon) icon.textContent = '−';
                    }
                }
            }
        });
    }

    function updateVisibility() {
        const typeFilters = getTypeFilters();
        const activeTypes = typeFilters.length > 0
            ? new Set(Array.from(typeFilters).filter(f => f.checked).map(f => f.value))
            : null;

        const appearing: HTMLElement[] = [];

        getNodes().forEach(node => {
            const parentId = node.getAttribute('data-parent') || '';
            const type = node.getAttribute('data-type') || '';

            let visible = false;

            // Top-level nodes (no parent) are always visible
            if (!parentId) {
                visible = true;
            } else if (expandedNodes.has(parentId)) {
                // Children are visible if parent is expanded
                visible = true;
            }

            // Must also match type filter if filters exist
            if (visible && activeTypes && !activeTypes.has(type)) {
                visible = false;
            }

            const wasHidden = node.classList.contains('hidden');
            node.classList.toggle('hidden', !visible);

            // Track nodes that just became visible for entrance animation
            if (visible && wasHidden) {
                appearing.push(node);
            }
        });

        // Animate newly revealed nodes
        if (appearing.length > 0) {
            gsap.fromTo(appearing,
                { opacity: 0, x: -8 },
                { opacity: 1, x: 0, duration: 0.25, stagger: 0.03, ease: 'power2.out', clearProps: 'transform,opacity' },
            );
        }
    }

    // Event delegation: single click handler on tree list for toggle buttons
    if (treeList) {
        treeList.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;
            const toggleBtn = target.closest('.tree-toggle');
            if (!toggleBtn) return;

            e.preventDefault();
            e.stopPropagation();

            const node = toggleBtn.closest('.tree-node');
            if (!node) return;

            const id = node.getAttribute('data-id');
            if (!id) return;

            if (expandedNodes.has(id)) {
                expandedNodes.delete(id);
                toggleBtn.classList.remove('expanded');
                const icon = toggleBtn.querySelector('.toggle-icon');
                if (icon) icon.textContent = '+';
            } else {
                expandedNodes.add(id);
                toggleBtn.classList.add('expanded');
                const icon = toggleBtn.querySelector('.toggle-icon');
                if (icon) icon.textContent = '−';
            }

            updateVisibility();
        });
    }

    // Event delegation: type filters (on document to catch dynamically added filters)
    document.addEventListener('change', (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('type-filter')) {
            updateVisibility();
        }
    });

    // Event delegation: control buttons
    if (controlsContainer) {
        controlsContainer.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;
            const button = target.closest('button');
            if (!button) return;

            const id = button.id;
            if (id === 'expand-all' || id === 'detail-expand') {
                expandAll();
            } else if (id === 'collapse-all' || id === 'detail-collapse') {
                collapseAll();
            } else if (id === 'expand-one-level') {
                expandOneLevel();
            } else if (id === 'collapse-one-level') {
                collapseOneLevel();
            }
        });
    }

    function expandAll() {
        getNodes().forEach(node => {
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            if (hasChildren) {
                const id = node.getAttribute('data-id');
                if (id) {
                    expandedNodes.add(id);
                    const toggleBtn = node.querySelector('.tree-toggle');
                    if (toggleBtn) {
                        toggleBtn.classList.add('expanded');
                        const icon = toggleBtn.querySelector('.toggle-icon');
                        if (icon) icon.textContent = '−';
                    }
                }
            }
        });
        updateVisibility();
    }

    function collapseAll() {
        expandedNodes.clear();
        getNodes().forEach(node => {
            const toggleBtn = node.querySelector('.tree-toggle');
            if (toggleBtn) {
                toggleBtn.classList.remove('expanded');
                const icon = toggleBtn.querySelector('.toggle-icon');
                if (icon) icon.textContent = '+';
            }
        });
        updateVisibility();
    }

    function expandOneLevel() {
        const nodes = getNodes();
        // 1. Calculate the current maximum depth of expanded nodes.
        let currentMaxExpandedDepth = -1;
        nodes.forEach(node => {
            const id = node.getAttribute('data-id');
            if (id && expandedNodes.has(id)) {
                const depth = parseInt(node.getAttribute('data-depth') || '0');
                if (depth > currentMaxExpandedDepth) {
                    currentMaxExpandedDepth = depth;
                }
            }
        });

        // 2. The target depth to expand is one level deeper.
        const targetDepth = currentMaxExpandedDepth + 1;
        const nodesToExpand: { id: string; element: Element }[] = [];

        // 3. Identify nodes to expand
        nodes.forEach(node => {
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            const id = node.getAttribute('data-id');
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            const parentId = node.getAttribute('data-parent');

            if (id && hasChildren && !expandedNodes.has(id)) {
                if (depth <= targetDepth) {
                    // Check if parent is visible (either root or expanded)
                    const parentExpanded = !parentId || expandedNodes.has(parentId);
                    if (parentExpanded) {
                        nodesToExpand.push({ id, element: node });
                    }
                }
            }
        });

        // 4. Batch update
        nodesToExpand.forEach(({ id, element }) => {
            expandedNodes.add(id);
            const toggleBtn = element.querySelector('.tree-toggle');
            if (toggleBtn) {
                toggleBtn.classList.add('expanded');
                const icon = toggleBtn.querySelector('.toggle-icon');
                if (icon) icon.textContent = '−';
            }
        });

        updateVisibility();
    }

    function collapseOneLevel() {
        const nodes = getNodes();
        // Find current maximum expanded depth
        let maxExpandedDepth = 0;
        expandedNodes.forEach(id => {
            nodes.forEach(node => {
                if (node.getAttribute('data-id') === id) {
                    const depth = parseInt(node.getAttribute('data-depth') || '0');
                    maxExpandedDepth = Math.max(maxExpandedDepth, depth);
                }
            });
        });

        nodes.forEach(node => {
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            const id = node.getAttribute('data-id');
            const hasChildren = node.getAttribute('data-has-children') === 'true';

            // Collapse nodes deeper than the new max depth
            if (id && hasChildren && depth > maxExpandedDepth - 1) {
                expandedNodes.delete(id);
                const toggleBtn = node.querySelector('.tree-toggle');
                if (toggleBtn) {
                    toggleBtn.classList.remove('expanded');
                    const icon = toggleBtn.querySelector('.toggle-icon');
                    if (icon) icon.textContent = '+';
                }
            }
        });
        updateVisibility();
    }

    // Initialize visibility
    updateVisibility();
}
