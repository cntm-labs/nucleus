import SwiftUI

/// A picker view that lets the user switch between their organizations.
///
/// ```swift
/// NucleusOrgSwitcher(auth: auth, organizations: orgs)
/// ```
public struct NucleusOrgSwitcher: View {
    @ObservedObject var auth: NucleusAuth

    /// All organizations the user belongs to.
    public let organizations: [NucleusOrganization]

    @State private var isLoading = false
    @State private var errorMessage: String?

    public init(auth: NucleusAuth, organizations: [NucleusOrganization]) {
        self.auth = auth
        self.organizations = organizations
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if organizations.isEmpty {
                Text("No organizations")
                    .foregroundStyle(.secondary)
            } else {
                ForEach(organizations) { org in
                    Button {
                        Task { await switchTo(org) }
                    } label: {
                        HStack {
                            orgAvatar(org)
                            Text(org.name)
                                .lineLimit(1)
                            Spacer()
                            if auth.organization?.id == org.id {
                                Image(systemName: "checkmark")
                                    .foregroundStyle(.accentColor)
                            }
                        }
                        .contentShape(Rectangle())
                        .padding(.vertical, 6)
                    }
                    .buttonStyle(.plain)
                    .disabled(isLoading)
                }
            }

            if let errorMessage {
                Text(errorMessage)
                    .foregroundStyle(.red)
                    .font(.footnote)
            }
        }
    }

    // MARK: - Private

    @ViewBuilder
    private func orgAvatar(_ org: NucleusOrganization) -> some View {
        if let urlString = org.logoURL, let url = URL(string: urlString) {
            AsyncImage(url: url) { image in
                image.resizable().scaledToFill()
            } placeholder: {
                orgInitials(org)
            }
            .frame(width: 28, height: 28)
            .clipShape(RoundedRectangle(cornerRadius: 6))
        } else {
            orgInitials(org)
        }
    }

    private func orgInitials(_ org: NucleusOrganization) -> some View {
        ZStack {
            RoundedRectangle(cornerRadius: 6)
                .fill(Color.accentColor.opacity(0.15))
            Text(String(org.name.prefix(1)).uppercased())
                .font(.caption.bold())
                .foregroundStyle(Color.accentColor)
        }
        .frame(width: 28, height: 28)
    }

    private func switchTo(_ org: NucleusOrganization) async {
        guard org.id != auth.organization?.id else { return }
        isLoading = true
        errorMessage = nil
        do {
            try await auth.switchOrganization(org)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
