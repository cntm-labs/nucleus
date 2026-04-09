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
            orgList
            errorView
        }
    }

    // MARK: - Sub-views

    @ViewBuilder
    private var orgList: some View {
        if organizations.isEmpty {
            Text("No organizations")
                .foregroundStyle(.secondary)
        } else {
            ForEach(organizations, id: \.id) { org in
                orgRow(org)
            }
        }
    }

    @ViewBuilder
    private var errorView: some View {
        if let errorMessage {
            Text(errorMessage)
                .foregroundStyle(.red)
                .font(.footnote)
        }
    }

    private func orgRow(_ org: NucleusOrganization) -> some View {
        Button {
            Task { await switchTo(org) }
        } label: {
            orgRowLabel(org)
        }
        .buttonStyle(.plain)
        .disabled(isLoading)
    }

    private func orgRowLabel(_ org: NucleusOrganization) -> some View {
        HStack {
            orgAvatar(org)
            Text(org.name)
                .lineLimit(1)
            Spacer()
            if auth.organization?.id == org.id {
                Image(systemName: "checkmark")
                    .foregroundStyle(Color.accentColor)
            }
        }
        .contentShape(Rectangle())
        .padding(.vertical, 6)
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
