import Foundation

// MARK: - GitHub API DTOs

struct CheckRunsResponse: Decodable {
    let totalCount: Int
    let checkRuns: [CheckRun]
}

struct CheckRun: Decodable {
    let id: Int64
    let name: String
    let headSha: String
    let status: String
    let conclusion: String?
    let htmlUrl: String?
    let startedAt: Date?
    let completedAt: Date?
}

struct CombinedStatus: Decodable {
    let state: String
    let statuses: [StatusItem]

    struct StatusItem: Decodable {
        let state: String
        let context: String
        let targetUrl: String?
    }
}

struct WorkflowRunsResponse: Decodable {
    let totalCount: Int
    let workflowRuns: [WorkflowRun]
}

struct WorkflowRun: Decodable {
    let id: Int64
    let name: String?
    let displayTitle: String?
    let status: String
    let conclusion: String?
    let htmlUrl: String?
    let createdAt: Date?
    let runNumber: Int
}

struct JobsResponse: Decodable {
    let totalCount: Int
    let jobs: [Job]
}

struct Job: Decodable {
    let id: Int64
    let name: String
    let status: String
    let conclusion: String?
    let htmlUrl: String?
    let startedAt: Date?
    let completedAt: Date?
    let steps: [Step]?

    struct Step: Decodable {
        let name: String
        let number: Int
        let status: String
        let conclusion: String?
    }
}

struct RepoMeta: Decodable {
    let fullName: String
    let defaultBranch: String
    let stargazersCount: Int
    let openIssuesCount: Int
    let pushedAt: Date?
    let htmlUrl: String
}

struct Pull: Decodable {
    let number: Int
    let title: String
    let htmlUrl: String
    let draft: Bool?
}

struct Issue: Decodable {
    let number: Int
    let title: String
    let htmlUrl: String
    let pullRequest: PullMarker?

    struct PullMarker: Decodable {}
}

// MARK: - UI models

enum RepoStatus: Equatable {
    case loading
    case running
    case success
    case failed
    case noChecks
    case error
}

enum RunState {
    case success
    case failed
    case running
    case cancelled
    case skipped
    case other
}

struct RunRow: Identifiable {
    let id: Int64
    let title: String
    let state: RunState
    let url: String?
    let date: Date?
}

/// 失败的检查项（来自 check-runs，覆盖所有 CI 系统）
struct FailedCheck: Identifiable {
    let id: Int64
    let name: String
    let url: String?
    let duration: TimeInterval?
}

/// 失败的 Actions Job（带具体失败的 step）
struct FailedJob: Identifiable {
    let id: Int64
    let name: String
    let failedStep: String?
    let url: String?
    let duration: TimeInterval?
}

struct RepoState: Identifiable {
    var id: String { repo }
    let repo: String
    var status: RepoStatus = .loading
    var branch: String?
    var headSha: String?
    var commitSubject: String?
    var repoURL: String?
    var rows: [RunRow] = []
    var openPRs = -1
    var openIssues = -1
    var stars = 0
    var pushedAt: Date?
    var failedCount = 0
    var topFailURL: String?
    var errorText: String?
    var failedChecks: [FailedCheck] = []
    var failedJobs: [FailedJob] = []
    var totalChecks = 0
    var passedChecks = 0
    var skippedChecks = 0
}

func mapRunState(status: String?, conclusion: String?) -> RunState {
    switch conclusion {
    case "success": return .success
    case "failure", "timed_out", "action_required", "startup_failure": return .failed
    case "cancelled": return .cancelled
    case "skipped", "neutral": return .skipped
    case .some: return .other
    case .none: break
    }
    switch status {
    case "queued", "in_progress", "waiting", "pending", "requested": return .running
    default: return .other
    }
}

func isFailedConclusion(_ conclusion: String?) -> Bool {
    switch conclusion {
    case "failure", "timed_out", "action_required", "startup_failure": return true
    case .none: return true
    default: return false
    }
}

/// 严格版：nil（运行中/无结论）不算失败，用于筛选已结束的 run/job
func isFailureConclusion(_ conclusion: String?) -> Bool {
    switch conclusion {
    case "failure", "timed_out", "action_required", "startup_failure": return true
    default: return false
    }
}

func checkDuration(_ start: Date?, _ end: Date?) -> TimeInterval? {
    guard let start else { return nil }
    return (end ?? Date()).timeIntervalSince(start)
}

func aggregateCheckRuns(_ response: CheckRunsResponse) -> RepoStatus {
    guard !response.checkRuns.isEmpty else { return .noChecks }
    if response.checkRuns.contains(where: { $0.status != "completed" }) { return .running }
    if response.checkRuns.contains(where: { isFailedConclusion($0.conclusion) }) { return .failed }
    return .success
}
