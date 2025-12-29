# Execute Epic (Orchestrator)

## Objective

Manage the execution of a functional Epic by analyzing the dependency tree and spawning isolated, parallel agentic workers for individual tasks.

## Input

- **Epic ID**: The `bd` ticket ID for the Epic (e.g., `bd-e99.1`).
- **Instruction Source**: `docs/do/execute-task.md` (Content to be injected into worker containers).

## Process

1. **Initialization**

- Read Epic details: `bd show <epic_id>`
- Set Epic status to active: `bd update <epic_id> --status=in_progress`
- Verify `docs/do/execute-task.md` exists and is readable.

2. **Dependency Analysis & Scheduling**

- **Loop** (Repeat until all child tickets are resolved):

1. **Fetch State**: Retrieve the tree of child tickets for the Epic.

- `bd show <epic_id> --children`

2. **Filter Ready Tasks**: Identify tickets are ready:

- run `bd ready`.

3. **Check Completion**:

- If all children are `done` or `blocked` (waiting for review) -> **Proceed to Step 4**.
- If no tasks are ready but some are `in_progress` -> **Wait** (poll every 60s).
- If no tasks are ready and nothing is running -> **Error** (Deadlock detected).

3. **Worker Execution (Parallel)**

- For each **Ready Task** identified in Step 2:
- **Launch Container**:
  Start a new agent process in Docker. Pass the content of `execute-task.md` as the prompt and the Ticket ID as the target.

```bash
# Conceptual Docker Command
docker run \
  --rm \
  --name "agent-<ticket_id>" \
  -v $(pwd):/app \
  -w /app \
  -e BEADS_API_KEY=$BEADS_API_KEY \
  -e TARGET_TICKET_ID="<ticket_id>" \
  -e AGENT_SYSTEM_PROMPT="$(cat execute-task.md)" \
  agent-image:latest \
  run-agent

```

- **Monitor**:
- Track container exit codes.
- If exit code `0`: Assume task completed (Agent should have updated ticket to `blocked`).
- If exit code `!= 0`: Log error, post comment on Epic, and leave ticket in `todo` (do not retry infinite loops).

4. **Epic Finalization**

- Verify all child tickets are in `blocked` (ready for review) or `done` state.
- Summarize the work completed in the Epic description.
- Update Epic status: `bd update <epic_id> --status=blocked` (Ready for QA/Review).

## Constraints

- **Concurrency**: Limit to **X** concurrent Docker containers (e.g., 3) to avoid resource exhaustion or API rate limiting.
- **Context Isolation**: Workers must strictly use the `execute-task.md` protocol. They do not share memory; they only coordinate via `bd` ticket statuses.
- **Safe Failover**: If the orchestrator crashes, it must be able to restart and resume from the current state of the `bd` tickets (stateless orchestration).

## Error Handling

- **Orphaned Tasks**: If a container dies without updating the ticket, the Orchestrator must detect the timeout, kill the container, and reset the ticket status from `in_progress` back to `todo` with a comment.
- **Scope Creep**: If a worker requests to create _new_ tickets, the Orchestrator must approve them and add them to the Epic's child list dynamically.
