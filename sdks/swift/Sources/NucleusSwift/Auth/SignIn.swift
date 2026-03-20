import Foundation

/// Handles email / password authentication.
actor SignInManager {

    static let shared = SignInManager()
    private let apiClient = APIClient.shared

    private init() {}

    /// Sign in with email and password, returning the full auth response.
    func signIn(email: String, password: String) async throws -> AuthResponse {
        let body: [String: String] = [
            "email": email,
            "password": password,
        ]

        let data: Data
        do {
            data = try await apiClient.post(
                path: "/auth/sign-in/email",
                body: body,
                token: nil
            )
        } catch let apiError as APIError {
            if case .httpError(let statusCode, _) = apiError, statusCode == 401 {
                throw NucleusAuthError.invalidCredentials
            }
            throw NucleusAuthError.networkError(apiError)
        }

        do {
            return try JSONDecoder.nucleus.decode(AuthResponse.self, from: data)
        } catch {
            throw NucleusAuthError.unknown("Failed to decode sign-in response.")
        }
    }
}
