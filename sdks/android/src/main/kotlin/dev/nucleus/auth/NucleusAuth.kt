package dev.nucleus.auth

import dev.nucleus.models.NucleusOrganization
import dev.nucleus.models.NucleusSession
import dev.nucleus.models.NucleusUser
import dev.nucleus.network.ApiClient
import dev.nucleus.session.SessionManager
import dev.nucleus.session.TokenStorage
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Central authentication state holder for the Nucleus SDK.
 *
 * Exposes [StateFlow] properties that Compose UIs can collect to
 * reactively respond to authentication changes.
 */
class NucleusAuth internal constructor(
    private val apiClient: ApiClient,
    private val tokenStorage: TokenStorage,
    private val sessionManager: SessionManager,
) {
    // ---- Reactive state (delegates to SessionManager) -------------------

    /** The currently authenticated user, or `null` if signed out. */
    val user: StateFlow<NucleusUser?> get() = sessionManager.user

    /** The active session, or `null` if signed out. */
    val session: StateFlow<NucleusSession?> get() = sessionManager.session

    /** Whether a user is currently signed in with a valid session. */
    val isSignedIn: StateFlow<Boolean> get() = sessionManager.isSignedIn

    // ---- Organization state ---------------------------------------------

    private val _organizations = MutableStateFlow<List<NucleusOrganization>>(emptyList())
    val organizations: StateFlow<List<NucleusOrganization>> = _organizations.asStateFlow()

    private val _activeOrganization = MutableStateFlow<NucleusOrganization?>(null)
    val activeOrganization: StateFlow<NucleusOrganization?> = _activeOrganization.asStateFlow()

    // ---- Actions --------------------------------------------------------

    /**
     * Sign in with email and password.
     *
     * @return the authenticated [NucleusUser] on success.
     * @throws NucleusAuthException on failure.
     */
    suspend fun signInWithEmailPassword(email: String, password: String): NucleusUser {
        val result = apiClient.signIn(email, password)
            ?: throw NucleusAuthException("Sign-in failed. Check credentials and try again.")
        sessionManager.setSession(result.session, result.user)
        return result.user
    }

    /** Sign out the current user and clear all local state. */
    suspend fun signOut() {
        try {
            apiClient.signOut()
        } catch (_: Exception) {
            // Best-effort server-side sign-out.
        }
        sessionManager.clearSession()
        _organizations.value = emptyList()
        _activeOrganization.value = null
    }

    /** Fetch the organizations that the current user belongs to. */
    suspend fun loadOrganizations(): List<NucleusOrganization> {
        val orgs = apiClient.getOrganizations()
        _organizations.value = orgs
        if (_activeOrganization.value == null && orgs.isNotEmpty()) {
            _activeOrganization.value = orgs.first()
        }
        return orgs
    }

    /** Switch the active organization context. */
    fun setActiveOrganization(organization: NucleusOrganization) {
        _activeOrganization.value = organization
    }

    // ---- Internal helpers -----------------------------------------------

    internal fun setSessionFromOAuth(session: NucleusSession, user: NucleusUser) {
        sessionManager.setSession(session, user)
    }
}

class NucleusAuthException(message: String, cause: Throwable? = null) :
    Exception(message, cause)
