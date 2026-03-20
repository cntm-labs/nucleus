import SwiftUI

/// A ready-made sign-in view with email/password fields and OAuth buttons.
///
/// ```swift
/// NucleusSignInView(auth: auth, oauthProviders: ["google", "github"])
/// ```
public struct NucleusSignInView: View {
    @ObservedObject var auth: NucleusAuth

    @State private var email = ""
    @State private var password = ""
    @State private var isLoading = false
    @State private var errorMessage: String?

    /// OAuth provider identifiers to show as buttons (e.g. `["google", "github"]`).
    public var oauthProviders: [String]

    public init(auth: NucleusAuth, oauthProviders: [String] = []) {
        self.auth = auth
        self.oauthProviders = oauthProviders
    }

    public var body: some View {
        VStack(spacing: 24) {
            Text("Sign In")
                .font(.largeTitle.bold())

            VStack(spacing: 12) {
                TextField("Email", text: $email)
                    .textContentType(.emailAddress)
                    .keyboardType(.emailAddress)
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)
                    .padding()
                    .background(Color(.secondarySystemBackground))
                    .cornerRadius(10)

                SecureField("Password", text: $password)
                    .textContentType(.password)
                    .padding()
                    .background(Color(.secondarySystemBackground))
                    .cornerRadius(10)
            }

            if let errorMessage {
                Text(errorMessage)
                    .foregroundStyle(.red)
                    .font(.footnote)
                    .multilineTextAlignment(.center)
            }

            Button {
                Task { await signInWithEmail() }
            } label: {
                Group {
                    if isLoading {
                        ProgressView()
                    } else {
                        Text("Sign In")
                    }
                }
                .frame(maxWidth: .infinity)
                .padding()
            }
            .buttonStyle(.borderedProminent)
            .disabled(email.isEmpty || password.isEmpty || isLoading)

            if !oauthProviders.isEmpty {
                dividerRow

                ForEach(oauthProviders, id: \.self) { provider in
                    Button {
                        Task { await signInWithOAuth(provider: provider) }
                    } label: {
                        Text("Continue with \(provider.capitalized)")
                            .frame(maxWidth: .infinity)
                            .padding()
                    }
                    .buttonStyle(.bordered)
                    .disabled(isLoading)
                }
            }
        }
        .padding(24)
    }

    // MARK: - Private Views

    private var dividerRow: some View {
        HStack {
            Rectangle().frame(height: 1).foregroundStyle(.quaternary)
            Text("or").foregroundStyle(.secondary).font(.footnote)
            Rectangle().frame(height: 1).foregroundStyle(.quaternary)
        }
    }

    // MARK: - Actions

    private func signInWithEmail() async {
        isLoading = true
        errorMessage = nil
        do {
            try await auth.signIn(email: email, password: password)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    private func signInWithOAuth(provider: String) async {
        isLoading = true
        errorMessage = nil
        do {
            try await auth.signInWithOAuth(provider: provider)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
