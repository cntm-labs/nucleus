import { useParams } from 'react-router-dom'

const mockOrgs = [
  { id: '1', name: 'Acme Corp', slug: 'acme', members_count: 25, created_at: '2026-01-20' },
  { id: '2', name: 'Startup Inc', slug: 'startup-inc', members_count: 8, created_at: '2026-02-15' },
]

export function ProjectOrgsPage() {
  const { slug: _slug } = useParams()

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Organizations</h1>
        <span className="text-sm text-gray-500">{mockOrgs.length} total</span>
      </div>

      <div className="grid gap-4">
        {mockOrgs.map(org => (
          <div key={org.id} className="bg-white rounded-xl border border-gray-200 p-6 hover:border-nucleus-300 transition-colors cursor-pointer">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="font-semibold">{org.name}</h2>
                <p className="text-sm text-gray-500">{org.slug}</p>
              </div>
              <div className="text-right">
                <p className="text-sm font-medium">{org.members_count} members</p>
                <p className="text-xs text-gray-400 mt-1">Created {new Date(org.created_at).toLocaleDateString()}</p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
