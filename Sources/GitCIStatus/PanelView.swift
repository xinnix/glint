import AppKit
import SwiftUI

func relativeTime(_ date: Date?) -> String {
    guard let date else { return "" }
    if abs(Date().timeIntervalSince(date)) < 10 { return "刚刚" }
    let formatter = RelativeDateTimeFormatter()
    formatter.locale = Locale(identifier: "zh_CN")
    formatter.unitsStyle = .abbreviated
    return formatter.localizedString(for: date, relativeTo: Date())
}

func openURLString(_ string: String?) {
    guard let string, let url = URL(string: string) else { return }
    NSWorkspace.shared.open(url)
}

func formatDuration(_ seconds: TimeInterval?) -> String {
    guard let seconds, seconds >= 1 else { return "" }
    let total = Int(seconds.rounded())
    if total < 60 { return "\(total)s" }
    let minutes = total / 60
    if minutes < 60 { return "\(minutes)m\(total % 60)s" }
    return "\(minutes / 60)h\(minutes % 60)m"
}

struct FailureEntry: Identifiable {
    let id: Int64
    let name: String
    let step: String?
    let duration: String
    let url: String?
}

struct PanelView: View {
    @EnvironmentObject var app: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                StatusPill(status: app.overall)
                Text("GitCIStatus").font(.headline)
                Spacer()
                if app.isRefreshing {
                    ProgressView().controlSize(.small)
                }
                Text(relativeTime(app.lastRefresh))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Divider()

            if app.repoStates.isEmpty {
                VStack(spacing: 10) {
                    if app.isRefreshing {
                        ProgressView().controlSize(.regular)
                        Text("正在加载仓库状态…")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                    } else {
                        Text("还没有监控的仓库")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                        Button("添加仓库") {
                            WindowManager.shared.openManager(app: app)
                        }
                    }
                }
                .frame(maxWidth: .infinity, minHeight: 380)
            } else {
                // MenuBarExtra 窗口高度只在创建时计算一次，必须用固定高度，
                // 否则初始空内容会把窗口压扁，数据加载后也不会再撑开
                ScrollView {
                    VStack(spacing: 8) {
                        ForEach(app.repoStates) { state in
                            RepoCard(state: state)
                        }
                    }
                }
                .frame(height: 400)
            }

            Divider()

            HStack(spacing: 12) {
                Button {
                    WindowManager.shared.openManager(app: app)
                } label: {
                    Label("管理", systemImage: "slider.horizontal.3")
                }
                .buttonStyle(.borderless)
                .controlSize(.small)

                Button {
                    app.refreshNow()
                } label: {
                    Label("刷新", systemImage: "arrow.clockwise")
                }
                .buttonStyle(.borderless)
                .controlSize(.small)
                .disabled(app.isRefreshing)

                Spacer()

                Text(app.tokenInfo)
                    .font(.caption2)
                    .foregroundStyle(.secondary)

                Button {
                    NSApp.terminate(nil)
                } label: {
                    Label("退出", systemImage: "power")
                }
                .buttonStyle(.borderless)
                .controlSize(.small)
            }
        }
        .padding(12)
        .frame(width: 380)
    }
}

struct StatusPill: View {
    let status: RepoStatus

    var body: some View {
        HStack(spacing: 4) {
            Circle().fill(color).frame(width: 7, height: 7)
            Text(label).font(.caption2).fontWeight(.semibold)
        }
        .padding(.horizontal, 6)
        .padding(.vertical, 2)
        .background(Capsule().fill(color.opacity(0.15)))
        .foregroundStyle(color)
    }

    private var color: Color {
        switch status {
        case .success: return .green
        case .failed: return .red
        case .running: return .orange
        case .loading: return .gray
        case .noChecks: return .gray
        case .error: return .purple
        }
    }

    private var label: String {
        switch status {
        case .success: return "通过"
        case .failed: return "失败"
        case .running: return "运行中"
        case .loading: return "加载中"
        case .noChecks: return "无 CI"
        case .error: return "错误"
        }
    }
}

struct RunDot: View {
    let state: RunState

    var body: some View {
        Circle().fill(color)
    }

    private var color: Color {
        switch state {
        case .success: return .green
        case .failed: return .red
        case .running: return .orange
        case .cancelled: return .gray
        case .skipped: return .gray.opacity(0.5)
        case .other: return .purple
        }
    }
}

struct RepoCard: View {
    let state: RepoState
    @State private var isExpanded = false

    private var failureEntries: [FailureEntry] {
        if !state.failedJobs.isEmpty {
            return state.failedJobs.map {
                FailureEntry(id: $0.id, name: $0.name, step: $0.failedStep, duration: formatDuration($0.duration), url: $0.url)
            }
        }
        return state.failedChecks.map {
            FailureEntry(id: $0.id, name: $0.name, step: nil, duration: formatDuration($0.duration), url: $0.url)
        }
    }

    private var compactStatus: (text: String, color: Color) {
        switch state.status {
        case .running: return ("运行中", .orange)
        case .failed:
            if state.failedCount == 1, let name = state.failedChecks.first?.name {
                return ("\(name) 失败", .red)
            }
            return ("\(state.failedCount) 项失败", .red)
        case .success: return ("通过", .green)
        case .noChecks: return ("无 CI", .gray)
        case .error: return ("错误", .purple)
        case .loading: return ("加载中…", .gray)
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            compactRow
            if isExpanded {
                detailSection
            }
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 5)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 7).fill(Color.primary.opacity(0.05)))
    }

    /// 折叠态：一行展示「当前正在运行的 / 最近一次」CI 状态
    private var compactRow: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(compactStatus.color)
                .frame(width: 8, height: 8)
            Text(state.repo)
                .font(.system(size: 12.5, weight: .medium))
                .lineLimit(1)
            Spacer(minLength: 8)
            Text(compactStatus.text)
                .font(.caption)
                .foregroundStyle(compactStatus.color)
                .lineLimit(1)
            Text(relativeTime(state.rows.first?.date ?? state.pushedAt))
                .font(.caption2)
                .foregroundStyle(.tertiary)
            Image(systemName: "chevron.right")
                .font(.caption2)
                .foregroundStyle(.tertiary)
                .rotationEffect(.degrees(isExpanded ? 90 : 0))
        }
        .contentShape(Rectangle())
        .onTapGesture {
            withAnimation(.easeInOut(duration: 0.15)) {
                isExpanded.toggle()
            }
        }
    }

    /// 展开态：提交、检查统计、失败详情、运行历史、链接
    @ViewBuilder
    private var detailSection: some View {
        if let error = state.errorText {
            Text(error)
                .font(.caption)
                .foregroundStyle(.red)
        } else {
            HStack(spacing: 6) {
                if let subject = state.commitSubject {
                    Text(subject)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                if let sha = state.headSha {
                    Text(String(sha.prefix(7)))
                        .font(.caption.monospaced())
                        .foregroundStyle(.tertiary)
                }
            }

            if state.totalChecks > 0 {
                HStack(spacing: 6) {
                    Text("检查 \(state.totalChecks) 项")
                        .foregroundStyle(.secondary)
                    if state.failedCount > 0 {
                        Text("✗ \(state.failedCount)").foregroundStyle(.red)
                    }
                    Text("✓ \(state.passedChecks)").foregroundStyle(.green)
                    if state.skippedChecks > 0 {
                        Text("⊘ \(state.skippedChecks)").foregroundStyle(.secondary)
                    }
                }
                .font(.caption2)
            }

            if state.status == .failed {
                failureSection
            }

            if !state.rows.isEmpty {
                VStack(alignment: .leading, spacing: 3) {
                    ForEach(state.rows.prefix(4)) { row in
                        HStack(spacing: 5) {
                            RunDot(state: row.state)
                                .frame(width: 8, height: 8)
                            Text(row.title)
                                .font(.caption)
                                .lineLimit(1)
                            Spacer()
                            Text(relativeTime(row.date))
                                .font(.caption2)
                                .foregroundStyle(.tertiary)
                        }
                        .contentShape(Rectangle())
                        .onTapGesture { openURLString(row.url) }
                    }
                }
            }

            if state.openPRs >= 0 {
                HStack(spacing: 10) {
                    if let url = URL(string: "https://github.com/\(state.repo)/pulls") {
                        Link("PR \(state.openPRs)", destination: url)
                    }
                    if let url = URL(string: "https://github.com/\(state.repo)/issues") {
                        Link("Issues \(state.openIssues)", destination: url)
                    }
                    if let url = URL(string: "https://github.com/\(state.repo)/actions") {
                        Link("Actions ↗", destination: url)
                    }
                    Spacer()
                    Text("★ \(state.stars)")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                .font(.caption)
            }
        }
    }

    @ViewBuilder
    private var failureSection: some View {
        let entries = failureEntries
        if !entries.isEmpty {
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 4) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.caption2)
                    Text("失败详情 · \(state.failedCount) 项")
                        .font(.caption2.bold())
                    Spacer()
                    if let url = state.topFailURL.flatMap(URL.init(string:)) {
                        Link("查看日志 ↗", destination: url)
                            .font(.caption2)
                    }
                }
                .foregroundStyle(.red)

                ForEach(entries.prefix(6)) { entry in
                    HStack(spacing: 5) {
                        Image(systemName: "xmark.circle.fill")
                            .font(.caption2)
                            .foregroundStyle(.red)
                        Text(entry.name)
                            .font(.caption)
                            .lineLimit(1)
                        if let step = entry.step {
                            Text("▸ \(step)")
                                .font(.caption2)
                                .foregroundStyle(.orange)
                                .lineLimit(1)
                        }
                        Spacer()
                        Text(entry.duration)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                    .contentShape(Rectangle())
                    .onTapGesture { openURLString(entry.url) }
                }

                if entries.count > 6 {
                    Text("还有 \(entries.count - 6) 项…")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
            }
            .padding(6)
            .background(RoundedRectangle(cornerRadius: 6).fill(Color.red.opacity(0.07)))
        }
    }
}
