import Foundation
import Combine

/// Observable authentication state for the current user.
///
/// Use this object as an `@EnvironmentObject` or `@StateObject` in your SwiftUI
/// views to reactively update the UI when the user signs in or out.
///
/// ```swift
/// @StateObject private var auth = NucleusAuth()
/// ```
@MainActor
public final class NucleusAuth: ObservableObject {

    // MARK: - Published State

    /// The currently signed-in user, or `nil` if not authenticated.
    @Published public private(set) var user: NucleusUser?

    /// The active session, or `nil` if not authenticated.
    @Published public private(set) var session: NucleusSession?

    /// The user's active organization, or `nil`.
    @Published public private(set) var organization: NucleusOrganization?

    /// Whether the user is currently signed in with a valid session.
    public var isSignedIn: Bool {
        session != nil && user != nil
    }

    // MARK: - Private

    private let sessionManager = SessionManager.shared
    private let apiClient = APIClient.shared
    private var refreshTask: Task<Void, Never>?

    // MARK: - Init

    public init() {
        Task { await restoreSession() }
    }

    // MARK: - Public API

    /// Sign in with email and password.
    public func signIn(email: String, password: String) async throws {
        let response = try await SignInManager.shared.signIn(email: email, password: password)
        apply(response)
    }

    /// Start an OAuth flow for the given provider (e.g. `"google"`, `"github"`).
    public func signInWithOAuth(provider: String) async throws {
        let response = try await OAuthManager.shared.startOAuthFlow(provider: provider)
        apply(response)
    }

    /// Sign out the current user and clear stored tokens.
    public func signOut() async {
        do {
            if let token = session?.accessToken {
                try await apiClient.post(path: "/auth/sign-out", body: nil, token: token)
            }
        } catch {
            // Best-effort server sign-out; always clear local state.
        }

        sessionManager.clearSession()
        user = nil
        session = nil
        organization = nil
    }

    /// Returns the current access token, refreshing it if expired.
    public func getToken() async throws -> String {
        guard let currentSession = session else {
            throw NucleusAuthError.notAuthenticated
        }

        if currentSession.isExpired {
            try await refreshSession()
        }

        guard let token = session?.accessToken else {
            throw NucleusAuthError.notAuthenticated
        }

        return token
    }

    /// Switch to a different organization.
    public func switchOrganization(_ org: NucleusOrganization) async throws {
        let token = try await getToken()
        let body = ["organizationId": org.id]
        let data = try await apiClient.post(
            path: "/auth/switch-organization",
            body: body,
            token: token
        )
        let response = try JSONDecoder.nucleus.decode(AuthResponse.self, from: data)
        apply(response)
    }

    // MARK: - Internal

    private func restoreSession() async {
        guard let stored = sessionManager.loadSession() else { return }

        session = stored.session
        user = stored.user
        organization = stored.organization

        // Proactively refresh if the token is expired or about to expire.
        if stored.session.isExpired || stored.session.isExpiringSoon {
            try? await refreshSession()
        }
    }

    private func refreshSession() async throws {
        guard let refreshToken = session?.refreshToken else {
            throw NucleusAuthError.notAuthenticated
        }

        let response = try await sessionManager.refresh(refreshToken: refreshToken)
        apply(response)
    }

    private func apply(_ response: AuthResponse) {
        self.user = response.user
        self.session = response.session
        self.organization = response.organization
        sessionManager.saveSession(response)
    }
}

// MARK: - Errors

public enum NucleusAuthError: LocalizedError {
    case notAuthenticated
    case invalidCredentials
    case oauthFailed(String)
    case networkError(Error)
    case unknown(String)

    public var errorDescription: String? {
        switch self {
        case .notAuthenticated:
            return "User is not authenticated."
        case .invalidCredentials:
            return "Invalid email or password."
        case .oauthFailed(let reason):
            return "OAuth authentication failed: \(reason)"
        case .networkError(let error):
            return "Network error: \(error.localizedDescription)"
        case .unknown(let message):
            return message
        }
    }
}

// MARK: - Auth Response

struct AuthResponse: Codable {
    let user: NucleusUser
    let session: NucleusSession
    let organization: NucleusOrganization?
}
