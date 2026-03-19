// Provider configuration page

const providers = [
  { id: 'google', name: 'Google', enabled: true, client_id: 'xxxx...xxxx' },
  { id: 'github', name: 'GitHub', enabled: true, client_id: 'xxxx...xxxx' },
  { id: 'apple', name: 'Apple', enabled: false, client_id: '' },
  { id: 'microsoft', name: 'Microsoft', enabled: false, client_id: '' },
]

export function ProvidersPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">OAuth Providers</h1>
      <div className="space-y-4">
        {providers.map(p => (
          <div key={p.id} className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={`w-3 h-3 rounded-full ${p.enabled ? 'bg-green-500' : 'bg-gray-300'}`} />
                <div>
                  <h3 className="font-semibold">{p.name}</h3>
                  {p.client_id && <p className="text-xs text-gray-400 mt-0.5">Client ID: {p.client_id}</p>}
                </div>
              </div>
              <button className="text-sm text-nucleus-600 hover:underline">
                {p.enabled ? 'Configure' : 'Enable'}
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
