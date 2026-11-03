// Glint widget spike — minimal WidgetKit extension proving:
//   1. A hand-built .appex (swiftc, no Xcode project) can be embedded in the Tauri bundle
//   2. The host app stays unsandboxed while the extension is sandboxed (WidgetKit requirement)
//   3. "widgetContainer" transport works: host writes
//      ~/Library/Containers/io.github.glint.widget/Data/widget_data.json,
//      the sandboxed extension reads it (its HOME resolves inside its own container).

import Foundation
import WidgetKit
import SwiftUI

// MARK: - Model (mirrors what the Rust poller will export)

private struct Repo: Decodable {
    let name: String
    let status: String
}

private struct SpikeEntry: TimelineEntry {
    let date: Date
    let repos: [Repo]
    let detail: String
    let loadedAt: String?
}

// MARK: - Provider

private struct SpikeProvider: TimelineProvider {
    func placeholder(in context: Context) -> SpikeEntry {
        SpikeEntry(date: Date(), repos: [Repo(name: "owner/repo", status: "success")], detail: "CI status at a glance", loadedAt: nil)
    }

    func getSnapshot(in context: Context, completion: @escaping (SpikeEntry) -> Void) {
        completion(Self.makeEntry())
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<SpikeEntry>) -> Void) {
        let entry = Self.makeEntry()
        let next = Calendar.current.date(byAdding: .minute, value: 10, to: Date()) ?? Date().addingTimeInterval(600)
        completion(Timeline(entries: [entry], policy: .after(next)))
    }

    static func makeEntry() -> SpikeEntry {
        if let payload = readPayload() {
            return SpikeEntry(date: Date(), repos: payload.repos, detail: payload.detail, loadedAt: payload.updatedAt)
        }
        return SpikeEntry(date: Date(), repos: [], detail: "Waiting for Glint…", loadedAt: nil)
    }

    /// widgetContainer transport: the sandboxed extension's HOME is its own
    /// ~/Library/Containers/<bundle-id>/Data — the unsandboxed host writes there directly.
    static func readPayload() -> (repos: [Repo], detail: String, updatedAt: String?)? {
        let path = NSHomeDirectory() + "/widget_data.json"
        guard let data = FileManager.default.contents(atPath: path) else { return nil }
        guard let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else { return nil }
        let repos: [Repo]
        if let raw = obj["repos"] as? [[String: Any]] {
            repos = raw.compactMap { dict in
                guard let name = dict["name"] as? String else { return nil }
                return Repo(name: name, status: (dict["status"] as? String) ?? "no_checks")
            }
        } else {
            repos = []
        }
        return (repos, (obj["detail"] as? String) ?? "", obj["updated_at"] as? String)
    }
}

// MARK: - View

private func statusColor(_ status: String) -> Color {
    switch status {
    case "success": return Color(red: 0.114, green: 0.725, blue: 0.329) // glint green #1db954
    case "failed", "error": return .red
    case "running": return .yellow
    default: return .gray
    }
}

private struct SpikeView: View {
    let entry: SpikeEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 4) {
                Image(systemName: "arrow.triangle.branch")
                    .font(.caption2)
                Text("Glint")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            if entry.repos.isEmpty {
                Text(entry.detail)
                    .font(.headline)
            } else {
                ForEach(entry.repos.prefix(6), id: \.name) { repo in
                    HStack(spacing: 5) {
                        Circle()
                            .fill(statusColor(repo.status))
                            .frame(width: 7, height: 7)
                        Text(repo.name)
                            .font(.caption)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                }
            }
            Spacer(minLength: 0)
            if let at = entry.loadedAt {
                Text(at)
                    .font(.system(size: 9))
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .containerBackground(.fill.tertiary, for: .widget)
    }
}

// MARK: - Widget

private struct GlintSpikeWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "GlintSpikeWidget", provider: SpikeProvider()) { entry in
            SpikeView(entry: entry)
        }
        .configurationDisplayName("Glint")
        .description("GitHub CI status at a glance (spike)")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

@main
struct GlintSpikeWidgetBundle: WidgetBundle {
    var body: some Widget {
        GlintSpikeWidget()
    }
}
