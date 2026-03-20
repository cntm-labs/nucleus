package dev.nucleus.ui.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import dev.nucleus.Nucleus
import dev.nucleus.auth.NucleusAuth
import dev.nucleus.models.NucleusOrganization
import dev.nucleus.models.NucleusSession
import dev.nucleus.models.NucleusUser

/**
 * Composition-local references so any descendant composable can access
 * Nucleus authentication state without explicit parameter passing.
 *
 * Usage:
 * ```kotlin
 * NucleusProvider {
 *     val user = LocalNucleusUser.current
 *     // ...
 * }
 * ```
 */
val LocalNucleusAuth = compositionLocalOf<NucleusAuth> {
    error("NucleusProvider not found. Wrap your composable tree with NucleusProvider {}.")
}

val LocalNucleusUser = compositionLocalOf<NucleusUser?> { null }
val LocalNucleusSession = compositionLocalOf<NucleusSession?> { null }
val LocalNucleusOrganization = compositionLocalOf<NucleusOrganization?> { null }
val LocalNucleusSignedIn = compositionLocalOf { false }

/**
 * Provides Nucleus authentication state to the composable tree.
 *
 * Place this near the root of your Compose hierarchy (e.g. inside `setContent {}`):
 * ```kotlin
 * setContent {
 *     NucleusProvider {
 *         AppNavGraph()
 *     }
 * }
 * ```
 */
@Composable
fun NucleusProvider(
    content: @Composable () -> Unit,
) {
    require(Nucleus.isConfigured) {
        "Nucleus.configure() must be called before using NucleusProvider."
    }

    val auth = Nucleus.auth
    val user by auth.user.collectAsState()
    val session by auth.session.collectAsState()
    val isSignedIn by auth.isSignedIn.collectAsState()
    val activeOrg by auth.activeOrganization.collectAsState()

    CompositionLocalProvider(
        LocalNucleusAuth provides auth,
        LocalNucleusUser provides user,
        LocalNucleusSession provides session,
        LocalNucleusSignedIn provides isSignedIn,
        LocalNucleusOrganization provides activeOrg,
    ) {
        content()
    }
}
