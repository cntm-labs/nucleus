export function ApiKeysPage() {
  const mockKeys = [
    { id: '1', prefix: 'pk_live_abc12', key_type: 'publishable', label: 'Frontend', last_used: '2026-03-19', created: '2026-01-15' },
    { id: '2', prefix: 'sk_live_xyz78', key_type: 'secret', label: 'Backend', last_used: '2026-03-19', created: '2026-01-15' },
  ]

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">API Keys</h1>
        <button className="px-4 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 text-sm font-medium">
          Create Key
        </button>
      </div>
      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        <table className="w-full">
          <thead className="bg-gray-50 border-b">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Key</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Type</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Label</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Last Used</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y">
            {mockKeys.map(key => (
              <tr key={key.id}>
                <td className="px-6 py-4 font-mono text-sm">{key.prefix}...</td>
                <td className="px-6 py-4">
                  <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${
                    key.key_type === 'secret' ? 'bg-red-100 text-red-700' : 'bg-blue-100 text-blue-700'
                  }`}>{key.key_type}</span>
                </td>
                <td className="px-6 py-4 text-sm">{key.label}</td>
                <td className="px-6 py-4 text-sm text-gray-500">{key.last_used}</td>
                <td className="px-6 py-4">
                  <button className="text-sm text-red-600 hover:underline">Revoke</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
