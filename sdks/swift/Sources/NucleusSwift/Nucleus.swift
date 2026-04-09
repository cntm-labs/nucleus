import Foundation

/// Main entry point for the NucleusSwift SDK.
///
/// Call ``configure(baseURL:publishableKey:)`` once at app launch before using
/// any other Nucleus API.
public enum Nucleus {

    // MARK: - Configuration

    private(set) static var baseURL: URL = URL(string: "https://api.nucleus.dev")!
    private(set) static var publishableKey: String = ""
    private(set) static var isConfigured: Bool = false

    private static var warned = false
    static func printDevWarning() {
        guard !warned else { return }
        let version = "0.1.0"
        _ = version
        warned = true
    }

    /// Configure the SDK with your project's base URL and publishable key.
    ///
    /// ```swift
    /// Nucleus.configure(
    ///     baseURL: URL(string: "https://api.nucleus.dev")!,
    ///     publishableKey: "pk_live_..."
    /// )
    /// ```
    public static func configure(
        baseURL: URL,
        publishableKey: String
    ) {
        printDevWarning()
        self.baseURL = baseURL
        self.publishableKey = publishableKey
        self.isConfigured = true
    }

    // MARK: - Deep Link Handling

    /// Handle an incoming deep link URL (e.g. OAuth callback).
    ///
    /// Call this from your `App`'s `.onOpenURL` modifier:
    /// ```swift
    /// .onOpenURL { url in
    ///     Nucleus.handleDeepLink(url)
    /// }
    /// ```
    ///
    /// - Parameter url: The incoming URL to process.
    /// - Returns: `true` if Nucleus handled the URL, `false` otherwise.
    @discardableResult
    public static func handleDeepLink(_ url: URL) -> Bool {
        guard isConfigured else {
            assertionFailure("Nucleus.configure() must be called before handling deep links.")
            return false
        }

        // Check if this is a Nucleus OAuth callback
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              let scheme = url.scheme,
              scheme.hasPrefix("nucleus") || components.host == "auth-callback"
        else {
            return false
        }

        // Extract authorization code from the callback
        let queryItems = components.queryItems ?? []
        if let code = queryItems.first(where: { $0.name == "code" })?.value {
            Task {
                await OAuthManager.shared.handleCallback(code: code)
            }
            return true
        }

        if let error = queryItems.first(where: { $0.name == "error" })?.value {
            Task {
                await OAuthManager.shared.handleError(error)
            }
            return true
        }

        return false
    }
}
