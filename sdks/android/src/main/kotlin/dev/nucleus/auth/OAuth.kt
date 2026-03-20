package dev.nucleus.auth

import android.content.Context
import android.net.Uri
import androidx.browser.customtabs.CustomTabsIntent
import dev.nucleus.Nucleus
import dev.nucleus.models.NucleusUser
import kotlinx.coroutines.CompletableDeferred

/**
 * Handles OAuth social sign-in flows using Chrome Custom Tabs and deep-link
 * callbacks.
 *
 * Usage:
 * ```kotlin
 * val user = OAuth.startFlow(context, provider = "google")
 * ```
 *
 * Your `AndroidManifest.xml` must include an intent-filter for the redirect
 * URI scheme (e.g. `nucleus-<pk_hash>://oauth/callback`).
 */
object OAuth {

    /**
     * In-flight deferred that is completed when the deep-link callback
     * delivers the authorization code.
     */
    @Volatile
    private var pendingResult: CompletableDeferred<OAuthCallbackParams>? = null

    /**
     * Launch an OAuth sign-in flow for the given provider.
     *
     * @param context  Activity context.
     * @param provider OAuth provider identifier (e.g. "google", "github", "apple").
     * @return The authenticated [NucleusUser].
     * @throws NucleusAuthException on failure or cancellation.
     */
    suspend fun startFlow(context: Context, provider: String): NucleusUser {
        val config = Nucleus.config
        val redirectUri = buildRedirectUri(config.publishableKey)

        val authorizeUrl = Uri.parse(config.baseUrl)
            .buildUpon()
            .appendPath("v1")
            .appendPath("oauth")
            .appendPath(provider)
            .appendPath("authorize")
            .appendQueryParameter("redirect_uri", redirectUri)
            .appendQueryParameter("publishable_key", config.publishableKey)
            .build()

        val deferred = CompletableDeferred<OAuthCallbackParams>()
        pendingResult = deferred

        // Open the authorize URL in a Custom Tab.
        val customTabsIntent = CustomTabsIntent.Builder()
            .setShowTitle(true)
            .build()
        customTabsIntent.launchUrl(context, authorizeUrl)

        // Suspend until the deep-link callback arrives.
        val callbackParams = try {
            deferred.await()
        } catch (e: Exception) {
            pendingResult = null
            throw NucleusAuthException("OAuth flow was cancelled or failed.", e)
        }

        pendingResult = null

        // Exchange the authorization code for a session.
        val result = Nucleus.apiClient.exchangeOAuthCode(
            code = callbackParams.code,
            redirectUri = redirectUri,
        ) ?: throw NucleusAuthException("Failed to exchange OAuth code for session.")

        Nucleus.auth.setSessionFromOAuth(result.session, result.user)
        return result.user
    }

    /**
     * Call this from your deep-link handling Activity/Fragment when the
     * OAuth redirect arrives.
     *
     * @param uri The full callback URI containing `code` and optional `state` params.
     */
    fun handleCallback(uri: Uri) {
        val code = uri.getQueryParameter("code")
        val error = uri.getQueryParameter("error")

        if (error != null || code == null) {
            pendingResult?.completeExceptionally(
                NucleusAuthException("OAuth callback error: ${error ?: "missing code"}")
            )
            return
        }

        pendingResult?.complete(
            OAuthCallbackParams(
                code = code,
                state = uri.getQueryParameter("state"),
            )
        )
    }

    // ------------------------------------------------------------------

    private fun buildRedirectUri(publishableKey: String): String {
        val hash = publishableKey.hashCode().toUInt().toString(16)
        return "nucleus-$hash://oauth/callback"
    }
}

internal data class OAuthCallbackParams(
    val code: String,
    val state: String?,
)
