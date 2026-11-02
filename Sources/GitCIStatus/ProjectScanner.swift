import Foundation

/// 扫描本地目录里的 git 仓库，提取 GitHub remote（owner/repo）。
enum ProjectScanner {
    static func scan(root: String = "~/code", maxDepth: Int = 2, limit: Int = 30) -> [String] {
        let fm = FileManager.default
        let baseURL = URL(fileURLWithPath: (root as NSString).expandingTildeInPath)
        guard let enumerator = fm.enumerator(
            at: baseURL,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles, .skipsPackageDescendants]
        ) else { return [] }

        let regex = try? NSRegularExpression(
            pattern: #"github\.com[:/]+([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+?)(?:\.git)?/?\s*$"#
        )
        var found = Set<String>()

        for case let url as URL in enumerator {
            if enumerator.level >= maxDepth { enumerator.skipDescendents() }
            var isDirectory: ObjCBool = false
            guard fm.fileExists(atPath: url.path, isDirectory: &isDirectory), isDirectory.boolValue else { continue }
            guard let content = try? String(contentsOfFile: url.appendingPathComponent(".git/config").path, encoding: .utf8) else { continue }

            for rawLine in content.split(separator: "\n") {
                let line = String(rawLine)
                guard line.contains("github.com"),
                      let match = regex?.firstMatch(in: line, range: NSRange(line.startIndex..., in: line)),
                      let ownerRange = Range(match.range(at: 1), in: line),
                      let nameRange = Range(match.range(at: 2), in: line)
                else { continue }
                found.insert("\(line[ownerRange])/\(line[nameRange])")
                if found.count >= limit { return found.sorted() }
            }
        }
        return found.sorted()
    }

    /// 通过 gh CLI 导入当前登录账号的仓库（任何用户都可用，不依赖本地目录结构）
    static func scanGhRepos(limit: Int = 100) -> [String] {
        for path in ["/opt/homebrew/bin/gh", "/usr/local/bin/gh", "/usr/bin/gh"]
        where FileManager.default.fileExists(atPath: path) {
            guard let output = run(
                path,
                ["repo", "list", "--limit", "\(limit)", "--json", "nameWithOwner", "-q", "map(.nameWithOwner)"]
            ) else { continue }
            return output
                .split(separator: "\n")
                .map { String($0).trimmingCharacters(in: .whitespaces) }
                .filter { !$0.isEmpty }
        }
        return []
    }

    @discardableResult
    private static func run(_ path: String, _ args: [String]) -> String? {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: path)
        process.arguments = args
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = Pipe()
        do {
            try process.run()
        } catch {
            return nil
        }
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        guard process.terminationStatus == 0 else { return nil }
        return String(data: data, encoding: .utf8)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
