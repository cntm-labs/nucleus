export function AnalyticsPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Analytics</h1>
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        {[
          { label: 'Total Users', value: '1,234', change: '+12%' },
          { label: 'MAU', value: '890', change: '+8%' },
          { label: 'Sign-ins (7d)', value: '1,023', change: '+15%' },
          { label: 'MFA Adoption', value: '34%', change: '+5%' },
        ].map(stat => (
          <div key={stat.label} className="bg-white rounded-xl border border-gray-200 p-6">
            <p className="text-sm text-gray-500">{stat.label}</p>
            <div className="flex items-end gap-2 mt-1">
              <p className="text-2xl font-bold">{stat.value}</p>
              <span className="text-xs text-green-600 font-medium mb-1">{stat.change}</span>
            </div>
          </div>
        ))}
      </div>
      <div className="bg-white rounded-xl border border-gray-200 p-6">
        <h2 className="text-lg font-semibold mb-4">Sign-in Methods Distribution</h2>
        <div className="space-y-3">
          {[
            { method: 'Password', pct: 65 },
            { method: 'OAuth (Google)', pct: 20 },
            { method: 'Magic Link', pct: 10 },
            { method: 'Passkey', pct: 5 },
          ].map(m => (
            <div key={m.method}>
              <div className="flex justify-between text-sm mb-1">
                <span>{m.method}</span><span className="text-gray-500">{m.pct}%</span>
              </div>
              <div className="h-2 bg-gray-100 rounded-full">
                <div className="h-2 bg-nucleus-500 rounded-full" style={{ width: `${m.pct}%` }} />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
