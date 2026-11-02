import Foundation

struct Config: Codable, Equatable {
    var repos: [String] = []
    var intervalSeconds = 30
    var token = ""
    var notifyOnFailure = true
    var notifyOnSuccess = true

    init() {}

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        repos = try c.decodeIfPresent([String].self, forKey: .repos) ?? []
        intervalSeconds = try c.decodeIfPresent(Int.self, forKey: .intervalSeconds) ?? 30
        token = try c.decodeIfPresent(String.self, forKey: .token) ?? ""
        notifyOnFailure = try c.decodeIfPresent(Bool.self, forKey: .notifyOnFailure) ?? true
        notifyOnSuccess = try c.decodeIfPresent(Bool.self, forKey: .notifyOnSuccess) ?? true
    }

    private enum CodingKeys: String, CodingKey {
        case repos, intervalSeconds, token, notifyOnFailure, notifyOnSuccess
    }
}

enum ConfigStore {
    static var fileURL: URL {
        let dir = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/gitcistatus", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("config.json")
    }

    static var exists: Bool {
        FileManager.default.fileExists(atPath: fileURL.path)
    }

    static func load() -> Config? {
        guard let data = try? Data(contentsOf: fileURL) else { return nil }
        return try? JSONDecoder().decode(Config.self, from: data)
    }

    static func save(_ config: Config) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try encoder.encode(config)
        try data.write(to: fileURL, options: [.atomic])
        try? FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: fileURL.path)
    }
}
