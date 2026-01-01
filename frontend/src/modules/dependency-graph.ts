import cytoscape, { Core, NodeSingular } from 'cytoscape';
// @ts-expect-error - cytoscape-dagre has no types
import dagre from 'cytoscape-dagre';

// Register dagre layout only once
if (!Object.prototype.hasOwnProperty.call(cytoscape.prototype, 'dagreRegistered')) {
    cytoscape.use(dagre);
    // @ts-expect-error - marking as registered
    cytoscape.prototype.dagreRegistered = true;
}

interface GraphNode {
    id: string;
    title: string;
    type: string;
    status: string;
    priority: number;
    parent: string | null;
}

interface GraphEdge {
    from: string;
    to: string;
    type: string;
}

interface GraphData {
    nodes: GraphNode[];
    edges: GraphEdge[];
}

// Color mappings matching CSS variables
const TYPE_COLORS: Record<string, string> = {
    epic: '#9b59b6',
    feature: '#3498db',
    task: '#e07a4a',
    bug: '#e74c3c',
    chore: '#95a5a6',
};

const STATUS_COLORS: Record<string, string> = {
    open: '#e07a4a',
    in_progress: '#7eb8da',
    blocked: '#d48a8a',
    closed: '#6a6560',
    deferred: '#b8a9c9',
};

export function initDependencyGraph() {
    const container = document.getElementById('cy');
    if (!container) return;

    const epicSelect = document.getElementById('epic-select') as HTMLSelectElement | null;
    let cy: Core | null = null;
    let allData: GraphData | null = null;

    // Fetch graph data and initialize
    async function init() {
        try {
            const response = await fetch('/api/graph');
            if (!response.ok) throw new Error('Failed to fetch graph data');
            allData = await response.json();

            // Populate epic selector
            if (epicSelect && allData) {
                populateEpicSelector(allData.nodes);
            }

            // Start with empty graph - user must select an epic
            // (rendering all 100+ nodes at once causes stack overflow in dagre)
            showEmptyState('Select an epic to view its dependency graph');
        } catch {
            showEmptyState('Failed to load graph data');
        }
    }

    function showEmptyState(message: string) {
        container.innerHTML = `<div class="graph-empty-state">${message}</div>`;
    }

    function populateEpicSelector(nodes: GraphNode[]) {
        if (!epicSelect) return;

        // Find all epics (type === 'epic')
        const epics = nodes.filter(n => n.type === 'epic');

        // Sort by id
        epics.sort((a, b) => a.id.localeCompare(b.id));

        // Add options
        epics.forEach(epic => {
            const option = document.createElement('option');
            option.value = epic.id;
            option.textContent = `${epic.id}: ${epic.title}`;
            epicSelect.appendChild(option);
        });
    }

    function filterByEpic(epicId: string, data: GraphData): GraphData {
        if (!epicId) return data;

        const prefix = `${epicId}.`;

        // Find all descendants (epic itself + all dot-notation children)
        const descendantIds = new Set(
            data.nodes
                .filter(n => n.id === epicId || n.id.startsWith(prefix))
                .map(n => n.id)
        );

        return {
            nodes: data.nodes.filter(n => descendantIds.has(n.id)),
            edges: data.edges.filter(e => descendantIds.has(e.from) && descendantIds.has(e.to)),
        };
    }

    function renderGraph(data: GraphData) {
        // Destroy existing instance
        if (cy) {
            cy.destroy();
        }

        // Convert to cytoscape format
        const elements = [
            ...data.nodes.map(n => ({
                data: {
                    id: n.id,
                    label: n.title,
                    type: n.type,
                    status: n.status,
                    priority: n.priority,
                },
            })),
            ...data.edges.map(e => ({
                data: {
                    id: `${e.from}-${e.to}`,
                    source: e.from,
                    target: e.to,
                    edgeType: e.type,
                },
            })),
        ];

        cy = cytoscape({
            container,
            elements,
            layout: {
                name: 'dagre',
                rankDir: 'TB', // Top to bottom
                nodeSep: 80,
                rankSep: 100,
                padding: 30,
            } as cytoscape.LayoutOptions,
            style: [
                // Base node style
                {
                    selector: 'node',
                    style: {
                        'background-color': '#666',
                        'border-width': 3,
                        'border-color': '#999',
                        'label': 'data(label)',
                        'text-valign': 'center',
                        'text-halign': 'center',
                        'text-wrap': 'wrap',
                        'text-max-width': '120px',
                        'font-size': '11px',
                        'color': '#fff',
                        'text-outline-color': '#333',
                        'text-outline-width': 1,
                        'width': 140,
                        'height': 50,
                        'shape': 'roundrectangle',
                    },
                },
                // Type-based colors
                {
                    selector: 'node[type="epic"]',
                    style: {
                        'background-color': TYPE_COLORS.epic,
                        'width': 160,
                        'height': 60,
                        'font-size': '12px',
                        'font-weight': 'bold',
                    },
                },
                {
                    selector: 'node[type="feature"]',
                    style: {
                        'background-color': TYPE_COLORS.feature,
                    },
                },
                {
                    selector: 'node[type="task"]',
                    style: {
                        'background-color': TYPE_COLORS.task,
                    },
                },
                {
                    selector: 'node[type="bug"]',
                    style: {
                        'background-color': TYPE_COLORS.bug,
                    },
                },
                {
                    selector: 'node[type="chore"]',
                    style: {
                        'background-color': TYPE_COLORS.chore,
                    },
                },
                // Status-based border colors
                {
                    selector: 'node[status="open"]',
                    style: {
                        'border-color': STATUS_COLORS.open,
                    },
                },
                {
                    selector: 'node[status="in_progress"]',
                    style: {
                        'border-color': STATUS_COLORS.in_progress,
                        'border-width': 4,
                    },
                },
                {
                    selector: 'node[status="blocked"]',
                    style: {
                        'border-color': STATUS_COLORS.blocked,
                        'border-width': 4,
                        'border-style': 'dashed',
                    },
                },
                {
                    selector: 'node[status="closed"]',
                    style: {
                        'border-color': STATUS_COLORS.closed,
                        'opacity': 0.6,
                    },
                },
                // Edge styles
                {
                    selector: 'edge',
                    style: {
                        'width': 2,
                        'line-color': '#888',
                        'target-arrow-color': '#888',
                        'target-arrow-shape': 'triangle',
                        'curve-style': 'bezier',
                    },
                },
                {
                    selector: 'edge[edgeType="parent-child"]',
                    style: {
                        'line-color': '#666',
                        'target-arrow-color': '#666',
                    },
                },
                {
                    selector: 'edge[edgeType="blocks"]',
                    style: {
                        'line-style': 'dashed',
                        'line-color': '#d48a8a',
                        'target-arrow-color': '#d48a8a',
                        'width': 3,
                    },
                },
                // Hover state
                {
                    selector: 'node:active',
                    style: {
                        'overlay-opacity': 0.2,
                    },
                },
            ],
            minZoom: 0.3,
            maxZoom: 2,
        });

        // Click to navigate
        cy.on('tap', 'node', (e) => {
            const node = e.target as NodeSingular;
            const id = node.id();
            window.location.href = `/tasks/${id}`;
        });

        // Hover tooltip
        cy.on('mouseover', 'node', (e) => {
            const node = e.target as NodeSingular;
            const data = node.data();
            container.title = `${data.id}\n${data.label}\nStatus: ${data.status}\nPriority: P${data.priority}`;
        });

        cy.on('mouseout', 'node', () => {
            container.title = '';
        });
    }

    // Epic selector change handler
    if (epicSelect) {
        epicSelect.addEventListener('change', () => {
            if (!allData) return;

            if (!epicSelect.value) {
                // No epic selected - show empty state
                if (cy) {
                    cy.destroy();
                    cy = null;
                }
                showEmptyState('Select an epic to view its dependency graph');
                return;
            }

            const filtered = filterByEpic(epicSelect.value, allData);
            if (filtered.nodes.length === 0) {
                showEmptyState('No issues found for this epic');
                return;
            }

            renderGraph(filtered);
        });
    }

    // Initialize
    init();
}
