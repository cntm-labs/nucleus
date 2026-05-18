package dev.nucleus.ui.compose

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import dev.nucleus.auth.OAuth
import dev.nucleus.auth.SignIn
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/**
 * A pre-built Compose sign-in screen with email/password, passkey, and
 * OAuth provider buttons.
 *
 * @param oauthProviders  List of OAuth provider identifiers to display (e.g. `listOf("google", "github")`).
 * @param enablePasskey   Whether to show the "Sign in with Passkey" button.
 * @param onSignInSuccess Called after successful authentication.
 * @param onError         Called with an error message when sign-in fails.
 */
@Composable
fun NucleusSignInScreen(
    oauthProviders: List<String> = emptyList(),
    enablePasskey: Boolean = true,
    onSignInSuccess: () -> Unit = {},
    onError: (String) -> Unit = {},
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp, vertical = 48.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = "Sign In",
            style = MaterialTheme.typography.headlineMedium,
        )

        Spacer(modifier = Modifier.height(32.dp))

        EmailPasswordSection(
            email = email,
            password = password,
            isLoading = isLoading,
            onEmailChange = { email = it },
            onPasswordChange = { password = it },
            onSubmit = {
                scope.runAuthFlow(
                    setLoading = { isLoading = it },
                    onSuccess = onSignInSuccess,
                    onError = onError,
                    defaultError = "Sign-in failed.",
                ) { SignIn.withEmailPassword(email.trim(), password) }
            },
        )

        if (enablePasskey) {
            Spacer(modifier = Modifier.height(12.dp))
            PasskeyButton(
                isLoading = isLoading,
                onClick = {
                    scope.runAuthFlow(
                        setLoading = { isLoading = it },
                        onSuccess = onSignInSuccess,
                        onError = onError,
                        defaultError = "Passkey sign-in failed.",
                    ) { SignIn.withPasskey(context) }
                },
            )
        }

        if (oauthProviders.isNotEmpty()) {
            Spacer(modifier = Modifier.height(20.dp))
            oauthProviders.forEach { provider ->
                OAuthProviderButton(
                    provider = provider,
                    isLoading = isLoading,
                    onClick = {
                        scope.runAuthFlow(
                            setLoading = { isLoading = it },
                            onSuccess = onSignInSuccess,
                            onError = onError,
                            defaultError = "OAuth sign-in failed.",
                        ) { OAuth.startFlow(context, provider) }
                    },
                )
            }
        }
    }
}

@Composable
private fun EmailPasswordSection(
    email: String,
    password: String,
    isLoading: Boolean,
    onEmailChange: (String) -> Unit,
    onPasswordChange: (String) -> Unit,
    onSubmit: () -> Unit,
) {
    OutlinedTextField(
        value = email,
        onValueChange = onEmailChange,
        label = { Text("Email") },
        singleLine = true,
        keyboardOptions = KeyboardOptions(
            keyboardType = KeyboardType.Email,
            imeAction = ImeAction.Next,
        ),
        modifier = Modifier.fillMaxWidth(),
    )

    Spacer(modifier = Modifier.height(12.dp))

    OutlinedTextField(
        value = password,
        onValueChange = onPasswordChange,
        label = { Text("Password") },
        singleLine = true,
        visualTransformation = PasswordVisualTransformation(),
        keyboardOptions = KeyboardOptions(
            keyboardType = KeyboardType.Password,
            imeAction = ImeAction.Done,
        ),
        modifier = Modifier.fillMaxWidth(),
    )

    Spacer(modifier = Modifier.height(20.dp))

    Button(
        onClick = onSubmit,
        enabled = !isLoading && email.isNotBlank() && password.isNotBlank(),
        modifier = Modifier.fillMaxWidth(),
    ) {
        if (isLoading) {
            CircularProgressIndicator(
                color = MaterialTheme.colorScheme.onPrimary,
                strokeWidth = 2.dp,
            )
        } else {
            Text("Continue")
        }
    }
}

@Composable
private fun PasskeyButton(
    isLoading: Boolean,
    onClick: () -> Unit,
) {
    OutlinedButton(
        onClick = onClick,
        enabled = !isLoading,
        modifier = Modifier.fillMaxWidth(),
    ) {
        Text("Sign in with Passkey")
    }
}

@Composable
private fun OAuthProviderButton(
    provider: String,
    isLoading: Boolean,
    onClick: () -> Unit,
) {
    OutlinedButton(
        onClick = onClick,
        enabled = !isLoading,
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
    ) {
        Text("Continue with ${provider.replaceFirstChar { it.uppercase() }}")
    }
}

/**
 * Wrap a suspending auth action with the common loading + success + error +
 * cleanup pattern shared by all sign-in buttons. Extracted to keep the parent
 * Composable's cognitive complexity below Sonar's threshold.
 */
private fun CoroutineScope.runAuthFlow(
    setLoading: (Boolean) -> Unit,
    onSuccess: () -> Unit,
    onError: (String) -> Unit,
    defaultError: String,
    action: suspend () -> Unit,
) = launch {
    setLoading(true)
    try {
        action()
        onSuccess()
    } catch (e: Exception) {
        onError(e.message ?: defaultError)
    } finally {
        setLoading(false)
    }
}
