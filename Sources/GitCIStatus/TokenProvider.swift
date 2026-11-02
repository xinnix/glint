import Foundation

/// 按优先级解析 GitHub Token：环境变量 → 钥匙串 → gh CLI → 配置文件。
enum TokenProvider {
    private static let lock = NSLock()
    private static var cached: (value: String?, source: String, expires: Date)?

    static func token(configToken: String) -> String? {
        lock.lock()
        defer { lock.unlock() }
        if let c = cached, c.expires > Date() { return c.value }
        let resolved = resolve(configToken: configToken)
        cached = (resolved.value, resolved.source, Date().addingTimeInterval(300))
        return resolved.value
    }

    static func invalidate() {
        lock.lock()
        defer { lock.unlock() }
        cached = nil
    }

    static func describe(configToken: String) async -> String {
        await Task.detached(priority: .utility) {
            describeSync(configToken: configToken)
        }.value
    }

    static func describeSync(configToken: String) -> String {
        let resolved = resolve(configToken: configToken)
        if let value = resolved.value, !value.isEmpty {
            return "Token：\(resolved.source) ✓"
        }
        return "未配置 Token（仅支持公共仓库，易限流）"
    }

    static func saveToKeychain(_ rawToken: String) throws {
        let token = rawToken.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !token.isEmpty else { throw GHError(message: "Token 为空") }
        invalidate()
        guard run("/usr/bin/security", ["add-generic-password", "-U", "-s", "gitcistatus", "-a", "token", "-w", token]) != nil else {
            throw GHError(message: "写入钥匙串失败")
        }
    }

    private static func resolve(configToken: String) -> (value: String?, source: String) {
        let env = ProcessInfo.processInfo.environment
        if let t = env["GITHUB_TOKEN"] ?? env["GH_TOKEN"], !t.isEmpty {
            return (t, "环境变量")
        }
        if let t = run("/usr/bin/security", ["find-generic-password", "-s", "gitcistatus", "-a", "token", "-w"]), !t.isEmpty {
            return (t, "钥匙串")
        }
        for path in ["/opt/homebrew/bin/gh", "/usr/local/bin/gh"] where FileManager.default.fileExists(atPath: path) {
            if let t = run(path, ["auth", "token"]), !t.isEmpty {
                return (t, "gh CLI")
            }
        }
        if !configToken.isEmpty {
            return (configToken, "配置文件（建议改用钥匙串）")
        }
        return (nil, "")
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
