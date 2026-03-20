import SwiftUI

/// A compact button that shows the current user's avatar and name, with a
/// menu for signing out.
///
/// ```swift
/// NucleusUserButton(auth: auth)
/// ```
public struct NucleusUserButton: View {
    @ObservedObject var auth: NucleusAuth

    public init(auth: NucleusAuth) {
        self.auth = auth
    }

    public var body: some View {
        if let user = auth.user {
            Menu {
                Section {
                    Text(user.email)
                }
                if let org = auth.organization {
                    Section("Organization") {
                        Label(org.name, systemImage: "building.2")
                    }
                }
                Section {
                    Button(role: .destructive) {
                        Task { await auth.signOut() }
                    } label: {
                        Label("Sign Out", systemImage: "rectangle.portrait.and.arrow.forward")
                    }
                }
            } label: {
                HStack(spacing: 8) {
                    avatarView(user: user)
                    Text(user.displayName)
                        .lineLimit(1)
                }
            }
        }
    }

    // MARK: - Private

    @ViewBuilder
    private func avatarView(user: NucleusUser) -> some View {
        if let urlString = user.avatarURL, let url = URL(string: urlString) {
            AsyncImage(url: url) { image in
                image
                    .resizable()
                    .scaledToFill()
            } placeholder: {
                initialsView(user: user)
            }
            .frame(width: 32, height: 32)
            .clipShape(Circle())
        } else {
            initialsView(user: user)
        }
    }

    private func initialsView(user: NucleusUser) -> some View {
        ZStack {
            Circle()
                .fill(Color.accentColor.opacity(0.2))
            Text(user.initials)
                .font(.caption.bold())
                .foregroundStyle(Color.accentColor)
        }
        .frame(width: 32, height: 32)
    }
}
