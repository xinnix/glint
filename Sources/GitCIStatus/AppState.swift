import AppKit
import Foundation

@MainActor
final class AppState: ObservableObject {
    static let shared = AppState()

    @Published var config = Config()
    @Published var repoStates: [RepoState] = []
    @Published var isRefreshing = false
    @Published var lastRefresh: Date?
    @Published var menubarImage = MenubarIcon.image(for: .idle)
    @Published var tokenInfo = "正在读取 Token…"

    private var started = false
    private var previousStatuses: [String: RepoStatus] = [:]

    func start() {
        guard !started else { return }
        started = true

        if let saved = ConfigStore.load() {
            config = saved
        } else {
            // 通用发布版：首次运行不假设本地目录结构，保持空配置，
            // 由用户在管理窗口手动添加 / gh 导入 / 批量粘贴
            config = Config()
            try? ConfigStore.save(config)
        }

        Notifier.shared.requestAuth()
        Task {
            tokenInfo = await TokenProvider.describe(configToken: config.token)
            await runLoop()
        }
    }

    private func runLoop() async {
        await refreshAll()
        while true {
            // 没有 Token 时公共 API 限额只有 60 次/小时，自动放慢节奏
            let configToken = config.token
            let hasToken = await Task.detached(priority: .utility) {
                TokenProvider.token(configToken: configToken) != nil
            }.value
            let interval = hasToken ? max(10, config.intervalSeconds) : max(config.intervalSeconds, 240)
            try? await Task.sleep(nanoseconds: UInt64(interval) * 1_000_000_000)
            await refreshAll()
        }
    }

    // MARK: - 刷新

    func refreshAll() async {
        guard !isRefreshing else { return }
        isRefreshing = true
        defer { isRefreshing = false }

        let token = TokenProvider.token(configToken: config.token)
        let light = (token?.isEmpty ?? true)
        let repos = config.repos
        let client = GitHubClient()

        var results: [RepoState] = []
        await withTaskGroup(of: RepoState.self) { group in
            for repo in repos {
                group.addTask {
                    await Self.fetchRepo(repo: repo, token: token, light: light, client: client)
                }
            }
            for await state in group {
                results.append(state)
            }
        }

        let order = Dictionary(uniqueKeysWithValues: repos.enumerated().map { ($0.element, $0.offset) })
        results.sort { (order[$0.repo] ?? .max) < (order[$1.repo] ?? .max) }
        repoStates = results
        lastRefresh = Date()
        updateMenubar()
        processNotifications()
    }

    func refreshNow() {
        Task { await refreshAll() }
    }

    nonisolated static func fetchRepo(repo: String, token: String?, light: Bool, client: GitHubClient) async -> RepoState {
        var state = RepoState(repo: repo)
        do {
            let meta: RepoMeta = try await client.get("/repos/\(repo)", token: token)
            state.branch = meta.defaultBranch
            state.repoURL = meta.htmlUrl
            state.stars = meta.stargazersCount
            state.pushedAt = meta.pushedAt

            let ref = meta.defaultBranch.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? meta.defaultBranch
            let checkRuns: CheckRunsResponse = try await client.get(
                "/repos/\(repo)/commits/\(ref)/check-runs",
                query: ["per_page": "100"],
                token: token
            )
            if checkRuns.totalCount > 0 {
                state.status = aggregateCheckRuns(checkRuns)
                state.headSha = checkRuns.checkRuns.first?.headSha
                let failed = checkRuns.checkRuns.filter { $0.status == "completed" && isFailedConclusion($0.conclusion) }
                state.failedCount = failed.count
                state.topFailURL = failed.first?.htmlUrl
                state.failedChecks = failed.prefix(8).map {
                    FailedCheck(
                        id: $0.id,
                        name: $0.name,
                        url: $0.htmlUrl,
                        duration: checkDuration($0.startedAt, $0.completedAt)
                    )
                }
                state.totalChecks = checkRuns.totalCount
                state.passedChecks = checkRuns.checkRuns.filter { $0.conclusion == "success" || $0.conclusion == "neutral" }.count
                state.skippedChecks = checkRuns.checkRuns.filter { $0.conclusion == "skipped" }.count
            } else {
                // 仓库未用 Check Runs（如纯 Status API 的第三方 CI）
                let combined: CombinedStatus = try await client.get("/repos/\(repo)/commits/\(ref)/status", token: token)
                switch combined.state {
                case "success": state.status = .success
                case "failure", "error": state.status = .failed
                case "pending": state.status = .running
                default: state.status = .noChecks
                }
            }

            // 最近几次 Workflow 运行，用于面板展示
            var workflowRuns: [WorkflowRun] = []
            if let workflows: WorkflowRunsResponse = try? await client.get(
                "/repos/\(repo)/actions/runs",
                query: ["branch": meta.defaultBranch, "per_page": "6"],
                token: token
            ) {
                workflowRuns = workflows.workflowRuns
                state.rows = workflowRuns.map {
                    RunRow(
                        id: $0.id,
                        title: $0.displayTitle ?? $0.name ?? "workflow",
                        state: mapRunState(status: $0.status, conclusion: $0.conclusion),
                        url: $0.htmlUrl,
                        date: $0.createdAt
                    )
                }
            }
            if state.rows.isEmpty, !checkRuns.checkRuns.isEmpty {
                state.rows = checkRuns.checkRuns.prefix(6).map {
                    RunRow(
                        id: $0.id,
                        title: $0.name,
                        state: mapRunState(status: $0.status, conclusion: $0.conclusion),
                        url: $0.htmlUrl,
                        date: $0.completedAt ?? $0.startedAt
                    )
                }
            }
            state.commitSubject = state.rows.first?.title

            // 失败时定位具体 job / step
            if state.status == .failed, state.failedJobs.isEmpty,
               let failedRun = workflowRuns.first(where: { isFailureConclusion($0.conclusion) }) {
                if let jobs: JobsResponse = try? await client.get(
                    "/repos/\(repo)/actions/runs/\(failedRun.id)/jobs",
                    query: ["per_page": "100"],
                    token: token
                ) {
                    state.failedJobs = jobs.jobs
                        .filter { isFailureConclusion($0.conclusion) }
                        .prefix(6)
                        .map { job in
                            FailedJob(
                                id: job.id,
                                name: job.name,
                                failedStep: job.steps?.first(where: { isFailureConclusion($0.conclusion) })?.name,
                                url: job.htmlUrl,
                                duration: checkDuration(job.startedAt, job.completedAt)
                            )
                        }
                }
            }

            if !light {
                if let pulls: [Pull] = try? await client.get(
                    "/repos/\(repo)/pulls",
                    query: ["state": "open", "per_page": "100"],
                    token: token
                ) {
                    state.openPRs = pulls.count
                }
                if let issues: [Issue] = try? await client.get(
                    "/repos/\(repo)/issues",
                    query: ["state": "open", "per_page": "100"],
                    token: token
                ) {
                    state.openIssues = issues.filter { $0.pullRequest == nil }.count
                }
            }
        } catch {
            state.status = .error
            state.errorText = (error as? GHError)?.message ?? error.localizedDescription
        }
        return state
    }

    // MARK: - 菜单栏聚合状态

    var overall: RepoStatus {
        switch Self.aggregate(repoStates) {
        case .success: return .success
        case .failed: return .failed
        case .running: return .running
        case .error: return .error
        case .idle: return .noChecks
        }
    }

    func updateMenubar() {
        menubarImage = MenubarIcon.image(for: Self.aggregate(repoStates))
    }

    static func aggregate(_ states: [RepoState]) -> Aggregate {
        var failed = 0
        var running = false
        var success = false
        var errored = false
        for state in states {
            switch state.status {
            case .failed: failed += 1
            case .running: running = true
            case .success: success = true
            case .error: errored = true
            case .loading, .noChecks: break
            }
        }
        if failed > 0 { return .failed(failed) }
        if running { return .running }
        if errored && !success { return .error }
        if success { return .success }
        return .idle
    }

    // MARK: - 通知

    private func processNotifications() {
        for state in repoStates {
            let previous = previousStatuses[state.repo]
            defer { previousStatuses[state.repo] = state.status }
            guard let previous, previous != state.status else { continue }

            switch state.status {
            case .failed where config.notifyOnFailure:
                let body = state.failedCount > 0
                    ? "\(state.failedCount) 项检查未通过"
                    : "分支 \(state.branch ?? "-") 检查失败"
                Notifier.shared.notify(
                    title: "❌ \(state.repo) CI 失败",
                    body: body,
                    url: state.topFailURL ?? state.repoURL.map { $0 + "/actions" }
                )
            case .success where config.notifyOnSuccess:
                if previous == .running || previous == .failed || previous == .error {
                    Notifier.shared.notify(
                        title: "✅ \(state.repo) CI 通过",
                        body: "分支 \(state.branch ?? "-") 全部检查通过",
                        url: state.repoURL.map { $0 + "/actions" }
                    )
                }
            default:
                break
            }
        }
    }

    // MARK: - 仓库管理

    func saveConfig() {
        try? ConfigStore.save(config)
    }

    /// 批量添加：支持空格 / 逗号 / 换行分隔多个 owner/repo，返回实际新增数
    @discardableResult
    func addRepos(_ raw: String) -> Int {
        let candidates = raw
            .split(whereSeparator: { $0 == "," || $0 == "\n" || $0 == " " || $0 == "\t" })
            .map { String($0) }
        return importRepos(candidates)
    }

    /// 合并导入仓库列表（去重），返回实际新增数
    @discardableResult
    func importRepos(_ repos: [String]) -> Int {
        var added = 0
        for repo in repos where isValidRepo(repo) && !config.repos.contains(repo) {
            config.repos.append(repo)
            added += 1
        }
        guard added > 0 else { return 0 }
        saveConfig()
        refreshNow()
        return added
    }

    @discardableResult
    func importFromCodeDir() -> Int {
        importRepos(ProjectScanner.scan())
    }

    func removeRepo(_ repo: String) {
        config.repos.removeAll { $0 == repo }
        repoStates.removeAll { $0.repo == repo }
        previousStatuses.removeValue(forKey: repo)
        saveConfig()
        updateMenubar()
    }

    func invalidateToken() {
        TokenProvider.invalidate()
        Task {
            tokenInfo = await TokenProvider.describe(configToken: config.token)
            await refreshAll()
        }
    }

    private func isValidRepo(_ string: String) -> Bool {
        guard let regex = try? NSRegularExpression(pattern: #"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$"#) else { return false }
        return regex.firstMatch(in: string, range: NSRange(string.startIndex..., in: string)) != nil
    }
}
