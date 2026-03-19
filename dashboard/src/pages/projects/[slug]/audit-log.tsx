export function AuditLogPage() {
  const mockLogs = [
    { id: '1', action: 'user.created', actor: 'system', target: 'john@example.com', ip: '1.2.3.4', created_at: '2026-03-19T10:30:00Z' },
    { id: '2', action: 'session.created', actor: 'john@example.com', target: 'session_abc', ip: '1.2.3.4', created_at: '2026-03-19T10:30:05Z' },
    { id: '3', action: 'org.member.added', actor: 'john@example.com', target: 'jane@example.com', ip: '1.2.3.4', created_at: '2026-03-19T11:00:00Z' },
  ]

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Audit Log</h1>
      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        <table className="w-full">
          <thead className="bg-gray-50 border-b">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Action</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Actor</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Target</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">IP</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Time</th>
            </tr>
          </thead>
          <tbody className="divide-y">
            {mockLogs.map(log => (
              <tr key={log.id} className="hover:bg-gray-50">
                <td className="px-6 py-4">
                  <span className="px-2 py-0.5 bg-gray-100 rounded text-xs font-mono">{log.action}</span>
                </td>
                <td className="px-6 py-4 text-sm">{log.actor}</td>
                <td className="px-6 py-4 text-sm text-gray-500">{log.target}</td>
                <td className="px-6 py-4 text-sm font-mono text-gray-400">{log.ip}</td>
                <td className="px-6 py-4 text-sm text-gray-500">{new Date(log.created_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
