<!--file:create-prd.md-->

# Create PRD

## Objective

Generate a Product Requirements Document (PRD) in Markdown format from a user prompt. The PRD must be specific, actionable, and granular enough for direct task decomposition into development tickets.

## Target Audience

Junior developers (assume strong technical skills but limited domain knowledge; explain concepts clearly, avoid unnecessary jargon).

## Process

1. **Analyze Input** - Read the user feature request and provided context.
1. **Identify Critical Gaps** - Determine if essential information is missing. Critical gaps include:
   - Undefined user roles or personas
   - Missing core user actions or outcomes
   - Unclear data entities or attributes
   - Unknown integration points or system boundaries
   - Ambiguous success criteria or metrics
1. **Clarify (If Needed)** - If gaps exist, ask 3-5 clarifying questions.
   - Format: Numbered list. Open-ended questions preferred over multiple-choice to avoid missing context.
   - Example: "1. Which authentication provider should be used for this login flow?"
   - **STOP. Wait for user response before proceeding.**
1. **Make Reasonable Assumptions** - For non-critical missing information, make and document reasonable assumptions.
1. **Generate PRD** - If no critical gaps exist, generate the document using the structure below.
1. **Validate for Decomposition** - Ensure each functional requirement is granular enough to map to individual development tickets.

## PRD Structure

### Required Sections

1. **Overview** - Brief feature description, problem statement, and primary goal
1. **Goals** - 3-5 measurable objectives (SMART format preferred)
1. **Job Stories**:
   - As a [role], I can [functionality], so that [benefit].
1. **Assumptions** - List any assumptions made during PRD creation
1. **Functional Requirements** - Numbered list with acceptance criteria per item
1. **Non-functional Requirements** - Numbered list with acceptance criteria per item

```
   FR-1: [Requirement]
   - Acceptance: [Verifiable condition]

   NFR-1: [Requirement]
   - Acceptance: [Verifiable condition]
```

7. **Non-Goals** - Explicit exclusions (what this feature will not address)
8. **Success Metrics** - Quantitative metrics for post-launch validation (include both product and business metrics)

### Optional Sections

1. **Design Considerations** - UI/UX constraints, mockups
2. **Technical Constraints** - Dependencies, environment, execution context
3. **Open Questions** - Unresolved items
4. **Diagram** - Mermaid diagram illustrating the process if applicable

## Output

- **Format:** Markdown
- **File Naming Convention:** `prds/prd-[kebab-case-feature-name]-[version, v1, v2 etc.]-[YYYY-MM-DD].md`

## Constraints

- Respect the project's existing tech stack and architecture
- Planning only—do not write implementation code
- Write for junior developers: clear, explicit, and educational
- All requirements must be testable with clear acceptance criteria
