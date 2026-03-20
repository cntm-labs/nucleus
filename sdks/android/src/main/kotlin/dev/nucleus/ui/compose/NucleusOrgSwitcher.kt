package dev.nucleus.ui.compose

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * Dropdown component that lets the user switch between organizations.
 *
 * Automatically fetches organizations on first composition when [autoLoad]
 * is `true`. Must be placed inside a [NucleusProvider].
 */
@Composable
fun NucleusOrgSwitcher(
    modifier: Modifier = Modifier,
    autoLoad: Boolean = true,
    onOrganizationSelected: ((dev.nucleus.models.NucleusOrganization) -> Unit)? = null,
) {
    val auth = LocalNucleusAuth.current
    val activeOrg = LocalNucleusOrganization.current
    val organizations by auth.organizations.collectAsState()
    var expanded by remember { mutableStateOf(false) }

    // Auto-load organizations on first composition.
    if (autoLoad) {
        LaunchedEffect(Unit) {
            if (organizations.isEmpty()) {
                auth.loadOrganizations()
            }
        }
    }

    if (organizations.isEmpty()) return

    Column(modifier = modifier) {
        // Trigger
        Row(
            modifier = Modifier
                .clickable { expanded = true }
                .padding(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = activeOrg?.name ?: "Select organization",
                style = MaterialTheme.typography.titleSmall,
            )
            Spacer(modifier = Modifier.width(4.dp))
            Icon(
                imageVector = Icons.Default.ArrowDropDown,
                contentDescription = "Switch organization",
                modifier = Modifier.size(20.dp),
            )
        }

        // Dropdown
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
        ) {
            organizations.forEach { org ->
                DropdownMenuItem(
                    text = {
                        Text(
                            text = org.name,
                            style = if (org.id == activeOrg?.id)
                                MaterialTheme.typography.labelLarge
                            else
                                MaterialTheme.typography.bodyMedium,
                        )
                    },
                    onClick = {
                        expanded = false
                        auth.setActiveOrganization(org)
                        onOrganizationSelected?.invoke(org)
                    },
                )
            }
        }
    }
}
