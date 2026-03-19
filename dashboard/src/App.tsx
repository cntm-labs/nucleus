import { Routes, Route, Navigate } from 'react-router-dom'
import { AuthProvider } from './lib/auth-context'
import { DashboardLayout } from './components/layout'
import { LoginPage } from './pages/login'
import { RegisterPage } from './pages/register'
import { ProjectListPage } from './pages/projects/list'
import { ProjectCreatePage } from './pages/projects/create'
import { ProjectOverviewPage } from './pages/projects/[slug]/overview'
import { ProjectUsersPage } from './pages/projects/[slug]/users'
import { ProjectOrgsPage } from './pages/projects/[slug]/orgs'
import { AuthSettingsPage } from './pages/projects/[slug]/auth-settings'
import { ProvidersPage } from './pages/projects/[slug]/providers'
import { ApiKeysPage } from './pages/projects/[slug]/api-keys'
import { WebhooksPage } from './pages/projects/[slug]/webhooks'
import { AuditLogPage } from './pages/projects/[slug]/audit-log'
import { AnalyticsPage } from './pages/projects/[slug]/analytics'
import { BillingPage } from './pages/projects/[slug]/billing'

export default function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/" element={<DashboardLayout />}>
          <Route index element={<Navigate to="/projects" replace />} />
          <Route path="projects" element={<ProjectListPage />} />
          <Route path="projects/new" element={<ProjectCreatePage />} />
          <Route path="projects/:slug" element={<ProjectOverviewPage />} />
          <Route path="projects/:slug/users" element={<ProjectUsersPage />} />
          <Route path="projects/:slug/orgs" element={<ProjectOrgsPage />} />
          <Route path="projects/:slug/auth-settings" element={<AuthSettingsPage />} />
          <Route path="projects/:slug/providers" element={<ProvidersPage />} />
          <Route path="projects/:slug/api-keys" element={<ApiKeysPage />} />
          <Route path="projects/:slug/webhooks" element={<WebhooksPage />} />
          <Route path="projects/:slug/audit-log" element={<AuditLogPage />} />
          <Route path="projects/:slug/analytics" element={<AnalyticsPage />} />
          <Route path="projects/:slug/billing" element={<BillingPage />} />
        </Route>
      </Routes>
    </AuthProvider>
  )
}
