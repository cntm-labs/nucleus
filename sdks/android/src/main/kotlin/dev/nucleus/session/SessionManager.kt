package dev.nucleus.session

import dev.nucleus.models.NucleusSession
import dev.nucleus.models.NucleusUser
import dev.nucleus.network.ApiClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * Manages the current session lifecycle, including automatic token refresh.
 *
 * Exposes reactive [StateFlow]s so Compose (and other observers) can
 * recompose automatically when authentication state changes.
 */
class SessionManager(
    private val apiClient: ApiClient,
    private val tokenStorage: TokenStorage,
) {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var refreshJob: Job? = null

    private val _session = MutableStateFlow<NucleusSession?>(null)
    val session: StateFlow<NucleusSession?> = _session.asStateFlow()

    private val _user = MutableStateFlow<NucleusUser?>(null)
    val user: StateFlow<NucleusUser?> = _user.asStateFlow()

    private val _isSignedIn = MutableStateFlow(false)
    val isSignedIn: StateFlow<Boolean> = _isSignedIn.asStateFlow()

    // ------------------------------------------------------------------
    // Public API
    // ------------------------------------------------------------------

    /**
     * Persist the session tokens and start the auto-refresh loop.
     */
    fun setSession(session: NucleusSession, user: NucleusUser?) {
        tokenStorage.accessToken = session.accessToken
        tokenStorage.refreshToken = session.refreshToken
        tokenStorage.sessionId = session.id
        tokenStorage.expiresAt = session.expiresAt

        _session.value = session
        _user.value = user
        _isSignedIn.value = true

        scheduleTokenRefresh(session)
    }

    /**
     * Called during [dev.nucleus.Nucleus.configure] to silently restore
     * a persisted session.
     */
    fun restoreSessionIfAvailable() {
        if (!tokenStorage.hasValidTokens) return

        scope.launch {
            try {
                val response = apiClient.getSession(
                    tokenStorage.sessionId ?: return@launch
                )
                if (response != null) {
                    _session.value = response.session
                    _user.value = response.user
                    _isSignedIn.value = true
                    scheduleTokenRefresh(response.session)
                }
            } catch (_: Exception) {
                // Token may have been revoked server-side — fail silently.
                clearSession()
            }
        }
    }

    /** Clear all local state and cancel the refresh loop. */
    fun clearSession() {
        refreshJob?.cancel()
        refreshJob = null
        tokenStorage.clear()
        _session.value = null
        _user.value = null
        _isSignedIn.value = false
    }

    /** Cancel the background refresh coroutine. */
    fun stop() {
        refreshJob?.cancel()
        refreshJob = null
    }

    fun updateUser(user: NucleusUser) {
        _user.value = user
    }

    // ------------------------------------------------------------------
    // Token refresh
    // ------------------------------------------------------------------

    private fun scheduleTokenRefresh(session: NucleusSession) {
        refreshJob?.cancel()
        refreshJob = scope.launch {
            val now = System.currentTimeMillis() / 1000
            val delaySeconds = (session.expiresAt - now - 60).coerceAtLeast(0)
            delay(delaySeconds * 1000)
            refreshToken()
        }
    }

    private suspend fun refreshToken() {
        val rt = tokenStorage.refreshToken ?: return
        try {
            val newSession = apiClient.refreshSession(rt)
            if (newSession != null) {
                setSession(newSession, _user.value)
            } else {
                clearSession()
            }
        } catch (_: Exception) {
            clearSession()
        }
    }
}

/** Bundle returned when restoring a session from the API. */
data class SessionRestoreResponse(
    val session: NucleusSession,
    val user: NucleusUser?,
)
