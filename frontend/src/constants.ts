// Status values - must match backend beads::Status
export const STATUS = {
    OPEN: 'open',
    IN_PROGRESS: 'in_progress',
    BLOCKED: 'blocked',
    CLOSED: 'closed',
    DEFERRED: 'deferred',
} as const;

export type Status = (typeof STATUS)[keyof typeof STATUS];

// Status sort order (for sorting comparisons)
export const STATUS_ORDER: Record<Status, number> = {
    [STATUS.OPEN]: 0,
    [STATUS.IN_PROGRESS]: 1,
    [STATUS.BLOCKED]: 2,
    [STATUS.CLOSED]: 3,
    [STATUS.DEFERRED]: 4,
};

// Issue type values - must match backend beads::IssueType
export const ISSUE_TYPE = {
    EPIC: 'epic',
    FEATURE: 'feature',
    BUG: 'bug',
    TASK: 'task',
    CHORE: 'chore',
} as const;

export type IssueType = (typeof ISSUE_TYPE)[keyof typeof ISSUE_TYPE];

// Issue type sort order (for sorting comparisons)
export const TYPE_ORDER: Record<IssueType, number> = {
    [ISSUE_TYPE.EPIC]: 0,
    [ISSUE_TYPE.FEATURE]: 1,
    [ISSUE_TYPE.BUG]: 2,
    [ISSUE_TYPE.TASK]: 3,
    [ISSUE_TYPE.CHORE]: 4,
};
