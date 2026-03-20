package dev.nucleus.ui.compose

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

/**
 * A circular avatar button that shows the current user's initials and
 * presents a dropdown menu with account actions.
 *
 * Must be placed inside a [NucleusProvider].
 */
@Composable
fun NucleusUserButton(
    modifier: Modifier = Modifier,
    onSignOut: (() -> Unit)? = null,
) {
    val auth = LocalNucleusAuth.current
    val user = LocalNucleusUser.current
    val scope = rememberCoroutineScope()
    var expanded by remember { mutableStateOf(false) }

    if (user == null) return

    val initials = buildString {
        user.firstName?.firstOrNull()?.let { append(it.uppercase()) }
        user.lastName?.firstOrNull()?.let { append(it.uppercase()) }
    }.ifEmpty { "?" }

    Box(modifier = modifier) {
        // Avatar circle
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape)
                .clickable { expanded = true },
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = initials,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
        }

        // Dropdown menu
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
        ) {
            user.fullName?.let { name ->
                DropdownMenuItem(
                    text = { Text(name, style = MaterialTheme.typography.labelLarge) },
                    onClick = { /* no-op — informational */ },
                    enabled = false,
                )
            }

            user.emailAddress?.let { email ->
                DropdownMenuItem(
                    text = { Text(email, style = MaterialTheme.typography.bodySmall) },
                    onClick = { /* no-op — informational */ },
                    enabled = false,
                )
            }

            DropdownMenuItem(
                text = { Text("Sign out") },
                onClick = {
                    expanded = false
                    scope.launch {
                        auth.signOut()
                        onSignOut?.invoke()
                    }
                },
            )
        }
    }
}
