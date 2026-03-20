import Foundation

/// Manages session persistence and token refresh logic.
final class SessionManager: @unchecked Sendable {

    static let shared = SessionManager()

    private let storage = TokenStorage.shared
    private let apiClient = APIClient.shared

    /// Serialises concurrent refresh attempts so only one network call is made.
    private let refreshLock = NSLock()
    private var activeRefreshTask: Task<AuthResponse, Error>?

    private init() {}

    // MARK: - Persistence

    struct StoredAuth: Codable {
        let user: NucleusUser
        let session: NucleusSession
        let organization: NucleusOrganization?
    }

    func saveSession(_ response: AuthResponse) {
        let stored = StoredAuth(
            user: response.user,
            session: response.session,
            organization: response.organization
        )
        if let data = try? JSONEncoder.nucleus.encode(stored) {
            storage.save(data, forKey: Keys.session)
        }
    }

    func loadSession() -> StoredAuth? {
        guard let data = storage.load(forKey: Keys.session) else { return nil }
        return try? JSONDecoder.nucleus.decode(StoredAuth.self, from: data)
    }

    func clearSession() {
        storage.delete(forKey: Keys.session)
    }

    // MARK: - Token Refresh

    /// Refresh the session using the given refresh token. Coalesces concurrent
    /// calls so only a single network request is made.
    func refresh(refreshToken: String) async throws -> AuthResponse {
        refreshLock.lock()
        if let existing = activeRefreshTask {
            refreshLock.unlock()
            return try await existing.value
        }

        let task = Task<AuthResponse, Error> {
            defer {
                refreshLock.lock()
                activeRefreshTask = nil
                refreshLock.unlock()
            }

            let body = ["refreshToken": refreshToken]
            let data = try await apiClient.post(
                path: "/auth/token/refresh",
                body: body,
                token: nil
            )
            return try JSONDecoder.nucleus.decode(AuthResponse.self, from: data)
        }

        activeRefreshTask = task
        refreshLock.unlock()
        return try await task.value
    }

    // MARK: - Keys

    private enum Keys {
        static let session = "com.nucleus.session"
    }
}
