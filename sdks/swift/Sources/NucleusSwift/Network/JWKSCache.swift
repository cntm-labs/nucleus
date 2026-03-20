import Foundation

/// Fetches and caches the JSON Web Key Set (JWKS) from the Nucleus API.
///
/// The cache refreshes automatically when it expires (default: 1 hour).
public actor JWKSCache {

    public static let shared = JWKSCache()

    // MARK: - Types

    /// A single JSON Web Key.
    public struct JWK: Codable, Sendable {
        public let kty: String
        public let use: String?
        public let kid: String?
        public let alg: String?
        public let n: String?
        public let e: String?
        public let crv: String?
        public let x: String?
        public let y: String?
    }

    /// The JWKS response containing an array of keys.
    public struct JWKS: Codable, Sendable {
        public let keys: [JWK]
    }

    // MARK: - State

    private var cachedJWKS: JWKS?
    private var fetchedAt: Date?
    private var activeFetch: Task<JWKS, Error>?
    private let cacheDuration: TimeInterval

    private let apiClient = APIClient.shared

    // MARK: - Init

    init(cacheDuration: TimeInterval = 3600) {
        self.cacheDuration = cacheDuration
    }

    // MARK: - Public

    /// Return the cached JWKS, fetching from the server if needed.
    public func getJWKS() async throws -> JWKS {
        if let cached = cachedJWKS, let fetchedAt, Date().timeIntervalSince(fetchedAt) < cacheDuration {
            return cached
        }

        // Coalesce concurrent fetches.
        if let activeFetch {
            return try await activeFetch.value
        }

        let task = Task<JWKS, Error> {
            let data = try await apiClient.get(path: "/.well-known/jwks.json", token: nil)
            let jwks = try JSONDecoder.nucleus.decode(JWKS.self, from: data)
            return jwks
        }

        activeFetch = task

        do {
            let jwks = try await task.value
            cachedJWKS = jwks
            fetchedAt = Date()
            activeFetch = nil
            return jwks
        } catch {
            activeFetch = nil
            throw error
        }
    }

    /// Look up a specific key by its `kid` (Key ID).
    public func getKey(kid: String) async throws -> JWK? {
        let jwks = try await getJWKS()
        return jwks.keys.first { $0.kid == kid }
    }

    /// Force-refresh the cache on next access.
    public func invalidate() {
        cachedJWKS = nil
        fetchedAt = nil
    }
}
