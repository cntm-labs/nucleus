import { useParams, Link } from 'react-router-dom'

const mockStats = {
  total_users: 1234, mau: 890, sign_ins_today: 156, sign_ins_this_week: 1023,
}

export function ProjectOverviewPage() {
  const { slug } = useParams()

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold capitalize">{slug}</h1>
        <p className="text-gray-500 mt-1">Project overview and quick stats</p>
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        {[
          { label: 'Total Users', value: mockStats.total_users.toLocaleString() },
          { label: 'Monthly Active', value: mockStats.mau.toLocaleString() },
          { label: 'Sign-ins Today', value: mockStats.sign_ins_today.toLocaleString() },
          { label: 'Sign-ins This Week', value: mockStats.sign_ins_this_week.toLocaleString() },
        ].map((stat) => (
          <div key={stat.label} className="bg-white rounded-xl border border-gray-200 p-6">
            <p className="text-sm text-gray-500">{stat.label}</p>
            <p className="text-2xl font-bold mt-1">{stat.value}</p>
          </div>
        ))}
      </div>

      {/* Quick links */}
      <div className="bg-white rounded-xl border border-gray-200 p-6">
        <h2 className="text-lg font-semibold mb-4">Quick Actions</h2>
        <div className="grid grid-cols-2 lg:grid-cols-3 gap-3">
          {[
            { label: 'Users', path: 'users' },
            { label: 'Organizations', path: 'orgs' },
            { label: 'API Keys', path: 'api-keys' },
            { label: 'Auth Settings', path: 'auth-settings' },
            { label: 'Webhooks', path: 'webhooks' },
            { label: 'Audit Log', path: 'audit-log' },
          ].map((action) => (
            <Link key={action.path} to={`/projects/${slug}/${action.path}`}
              className="px-4 py-3 border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:border-nucleus-300 hover:text-nucleus-700 transition-colors">
              {action.label}
            </Link>
          ))}
        </div>
      </div>
    </div>
  )
}
