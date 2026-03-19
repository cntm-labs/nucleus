import { Routes, Route, Navigate } from 'react-router-dom'
import { AuthProvider } from './lib/auth-context'
import { DashboardLayout } from './components/layout'
import { LoginPage } from './pages/login'
import { ProjectListPage } from './pages/projects/list'
import { ProjectCreatePage } from './pages/projects/create'

export default function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/" element={<DashboardLayout />}>
          <Route index element={<Navigate to="/projects" replace />} />
          <Route path="projects" element={<ProjectListPage />} />
          <Route path="projects/new" element={<ProjectCreatePage />} />
          {/* More routes will be added by other tasks */}
        </Route>
      </Routes>
    </AuthProvider>
  )
}
