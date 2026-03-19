import { Outlet, Link, useLocation, Navigate } from 'react-router-dom'
import { useAuth } from '../lib/auth-context'

const navItems = [
  { path: '/projects', label: 'Projects', icon: '\u25A1' },
]

export function DashboardLayout() {
  const { isAuthenticated, account, logout } = useAuth()
  const location = useLocation()

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  return (
    <div className="flex h-screen">
      {/* Sidebar */}
      <aside className="w-64 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h1 className="text-xl font-bold text-nucleus-700">Nucleus</h1>
        </div>
        <nav className="flex-1 p-4 space-y-1">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium ${
                location.pathname.startsWith(item.path)
                  ? 'bg-nucleus-50 text-nucleus-700'
                  : 'text-gray-600 hover:bg-gray-50'
              }`}
            >
              <span>{item.icon}</span>
              {item.label}
            </Link>
          ))}
        </nav>
        <div className="p-4 border-t border-gray-200">
          <div className="text-sm text-gray-600">{account?.email}</div>
          <button onClick={logout} className="text-sm text-red-600 hover:underline mt-1">
            Sign out
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto bg-gray-50">
        <div className="max-w-6xl mx-auto p-8">
          <Outlet />
        </div>
      </main>
    </div>
  )
}
