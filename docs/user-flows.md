# Nacre User Flows

This document describes the key user flows for Nacre, a local-first web interface for monitoring and tracking agentic code tool progress.

## Target Users

1. **Project Managers (PMs)** - Monitor overall project health, track epic progress, identify blockers
2. **Developers** - Find work, track agent-generated tasks, update status efficiently

---

## Project Manager Flows

### 1. Daily Standup Review

**Goal:** Quickly understand what changed overnight and what needs attention.

**Flow:**
1. Open Nacre dashboard (`/`)
2. Review stats cards to see:
   - Issues in progress (active work)
   - Blocked issues (need attention)
   - Recent closures (completed work)
3. Check "Blocked Issues" section for items needing triage
4. Check "In Progress" section to verify expected work is happening
5. Navigate to Board (`/board`) for Kanban view of all work

**Features Used:**
- Dashboard stats cards
- Blocked/In-Progress lists on dashboard
- Board view for detailed status

### 2. Epic Progress Review

**Goal:** Check feature delivery progress and identify at-risk epics.

**Flow:**
1. From dashboard, click "Epics" nav link or epic card
2. View epics sorted by completion percentage
3. Expand epic children to see task breakdown
4. Click epic title to go to detail view
5. On epic detail, review:
   - Overall progress bar
   - Child issue status distribution
   - Individual task status

**Features Used:**
- Epic Progress section on dashboard
- Epics list view (`/epics`)
- Epic detail view (`/epics/:id`)
- Collapsible children on epics list

### 3. Blocker Triage

**Goal:** Identify and resolve blocked work items.

**Flow:**
1. Dashboard shows blocked items count prominently
2. Click blocked item to view issue detail
3. Review dependencies and blocking reasons
4. Navigate to blocking issues to understand chain
5. Use Board view's "Blocked" column for overview

**Features Used:**
- Dashboard blocked issues list
- Issue detail view
- Board view Blocked column

---

## Developer Flows

### 1. Pick Next Work Item

**Goal:** Find the highest priority ready item to work on.

**Flow:**
1. Open dashboard to see current state
2. Check "Ready" count in stats
3. Navigate to Board view (`/board`)
4. Look at "Ready" column sorted by priority
5. Filter by type if needed (features, bugs, tasks)
6. Click issue to view details
7. Drag card to "In Progress" to claim work

**Features Used:**
- Dashboard stats (Ready count)
- Board view Ready column
- Filter input
- Drag-and-drop status update

### 2. Track Agent Progress

**Goal:** Monitor tasks created by automated agents and ensure they're progressing.

**Flow:**
1. Open Issues view (`/issues`)
2. Use filter to search for agent-related keywords
3. Review issues grouped by epic to understand context
4. Check Board view for status distribution
5. Identify any agent-created work that's stuck or blocked

**Features Used:**
- Issues list with epic grouping
- Filter search
- Board view for status overview

### 3. Quick Status Update

**Goal:** Mark work as done with minimal friction.

**Flow:**
1. Open Board view (`/board`)
2. Find your in-progress card
3. Drag to "Closed" column to complete
4. Or click issue to view details, then update

**Features Used:**
- Board view drag-and-drop
- Issue card links
- Inline title editing (on Issues view)

### 4. Review PRD Context

**Goal:** Understand requirements before starting work.

**Flow:**
1. Navigate to PRDs (`/prds`)
2. Filter/search for relevant document
3. Click to view rendered PRD content
4. Use browser back to return

**Features Used:**
- PRDs list view
- PRD content rendering

---

## Navigation Patterns

### Primary Navigation

- **Dashboard** - Entry point, project overview
- **Issues** - Full list grouped by epic, inline editing
- **Epics** - Feature tracking with progress
- **Board** - Kanban workflow view
- **PRDs** - Documentation access
- **+ Create** - New issue creation

### Contextual Navigation

- Dashboard epic cards link to epic detail
- Issue cards/links go to issue detail
- Epic children link to individual issues
- Breadcrumb-style context (epic > issue)

### Search/Filter

- Available on all list views
- Real-time filtering by title, ID, status
- Case-insensitive matching

---

## Feature Requirements Derived from Flows

### Must Have (Implemented)
- [x] Dashboard with stats overview
- [x] Epic progress visualization
- [x] Board view with drag-and-drop
- [x] Issue detail pages
- [x] Universal search/filter
- [x] Dark/light theme
- [x] Collapsible epic children

### Should Have (Future)
- [ ] Task dependency graph visualization
- [ ] Assignment filtering on board
- [ ] Quick actions (close from list)
- [ ] Keyboard shortcuts for navigation

### Nice to Have (Future)
- [ ] Custom filters/saved views
- [ ] Bulk status updates
- [ ] Export to clipboard
- [ ] Timeline view for scheduling
