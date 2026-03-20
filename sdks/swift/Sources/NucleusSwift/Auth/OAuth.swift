import Foundation
import AuthenticationServices

/// Manages OAuth flows using `ASWebAuthenticationSession`.
@MainActor
public final class OAuthManager: NSObject, ASWebAuthenticationPresentationContextProviding {

    static let shared = OAuthManager()

    private var continuation: CheckedContinuation<AuthResponse, Error>?
    private let apiClient = APIClient.shared

    private override init() {
        super.init()
    }

    // MARK: - Public

    /// Launch the OAuth flow for the given provider.
    ///
    /// - Parameter provider: e.g. `"google"`, `"github"`, `"apple"`.
    /// - Returns: The authenticated user / session / org response.
    func startOAuthFlow(provider: String) async throws -> AuthResponse {
        guard Nucleus.isConfigured else {
            throw NucleusAuthError.oauthFailed("Nucleus.configure() must be called first.")
        }

        let callbackScheme = "nucleus-\(Nucleus.publishableKey.prefix(8))"
        let authURL = Nucleus.baseURL
            .appendingPathComponent("/auth/oauth/\(provider)")
            .appending(queryItems: [
                URLQueryItem(name: "publishable_key", value: Nucleus.publishableKey),
                URLQueryItem(name: "redirect_uri", value: "\(callbackScheme)://auth-callback"),
            ])

        return try await withCheckedThrowingContinuation { continuation in
            self.continuation = continuation

            let session = ASWebAuthenticationSession(
                url: authURL,
                callbackURLScheme: callbackScheme
            ) { [weak self] callbackURL, error in
                guard let self else { return }

                if let error {
                    self.continuation?.resume(throwing: NucleusAuthError.oauthFailed(error.localizedDescription))
                    self.continuation = nil
                    return
                }

                guard let callbackURL,
                      let components = URLComponents(url: callbackURL, resolvingAgainstBaseURL: false),
                      let code = components.queryItems?.first(where: { $0.name == "code" })?.value
                else {
                    self.continuation?.resume(throwing: NucleusAuthError.oauthFailed("No authorization code received."))
                    self.continuation = nil
                    return
                }

                Task {
                    await self.exchangeCode(code)
                }
            }

            session.presentationContextProvider = self
            session.prefersEphemeralWebBrowserSession = false
            session.start()
        }
    }

    // MARK: - Callback Handling (Deep Link Path)

    func handleCallback(code: String) async {
        await exchangeCode(code)
    }

    func handleError(_ error: String) async {
        continuation?.resume(throwing: NucleusAuthError.oauthFailed(error))
        continuation = nil
    }

    // MARK: - Private

    private func exchangeCode(_ code: String) async {
        do {
            let body = ["code": code]
            let data = try await apiClient.post(
                path: "/auth/oauth/token",
                body: body,
                token: nil
            )
            let response = try JSONDecoder.nucleus.decode(AuthResponse.self, from: data)
            continuation?.resume(returning: response)
        } catch {
            continuation?.resume(throwing: NucleusAuthError.oauthFailed(error.localizedDescription))
        }
        continuation = nil
    }

    // MARK: - ASWebAuthenticationPresentationContextProviding

    public func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        ASPresentationAnchor()
    }
}
