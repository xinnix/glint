import Foundation

struct GHError: LocalizedError {
    let message: String
    var errorDescription: String? { message }
}

/// ETag 条件请求缓存：GitHub 对 304 响应不计入 API 限流额度。
final class ETagCache {
    static let shared = ETagCache()
    private let lock = NSLock()
    private var store: [String: (etag: String, data: Data)] = [:]

    func get(_ key: String) -> (etag: String, data: Data)? {
        lock.lock()
        defer { lock.unlock() }
        return store[key]
    }

    func put(_ key: String, etag: String, data: Data) {
        lock.lock()
        defer { lock.unlock() }
        store[key] = (etag, data)
    }
}

struct GitHubClient {
    static let baseURL = "https://api.github.com"

    private static let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        d.dateDecodingStrategy = .iso8601
        return d
    }()

    func get<T: Decodable>(_ path: String, query: [String: String] = [:], token: String?) async throws -> T {
        var components = URLComponents(string: Self.baseURL + path)!
        if !query.isEmpty {
            components.queryItems = query.map { URLQueryItem(name: $0.key, value: $0.value) }
        }
        let url = components.url!

        var request = URLRequest(url: url)
        request.timeoutInterval = 15
        request.setValue("application/vnd.github+json", forHTTPHeaderField: "Accept")
        request.setValue("2022-11-28", forHTTPHeaderField: "X-GitHub-Api-Version")
        request.setValue("GitCIStatus/1.0", forHTTPHeaderField: "User-Agent")
        if let token, !token.isEmpty {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        if let cached = ETagCache.shared.get(url.absoluteString) {
            request.setValue(cached.etag, forHTTPHeaderField: "If-None-Match")
        }

        let (data, response) = try await URLSession.shared.data(for: request)
        guard let http = response as? HTTPURLResponse else {
            throw GHError(message: "无效的网络响应")
        }

        switch http.statusCode {
        case 200:
            if let etag = http.value(forHTTPHeaderField: "ETag") {
                ETagCache.shared.put(url.absoluteString, etag: etag, data: data)
            }
            return try Self.decoder.decode(T.self, from: data)
        case 304:
            if let cached = ETagCache.shared.get(url.absoluteString) {
                return try Self.decoder.decode(T.self, from: cached.data)
            }
            throw GHError(message: "本地缓存失效，请重试")
        case 401:
            throw GHError(message: "Token 无效 (401)，请在管理窗口检查")
        case 403:
            if http.value(forHTTPHeaderField: "X-RateLimit-Remaining") == "0" {
                throw GHError(message: "API 限流 (403)，配置 Token 可提升至 5000 次/小时")
            }
            throw GHError(message: "无访问权限 (403)")
        case 404:
            throw GHError(message: "仓库不存在或无权限 (404)")
        default:
            throw GHError(message: "GitHub API 错误 (\(http.statusCode))")
        }
    }
}
