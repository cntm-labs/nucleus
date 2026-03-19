import { useParams } from 'react-router-dom'
import { useState } from 'react'

const mockUsers = [
  { id: '1', email: 'john@example.com', first_name: 'John', last_name: 'Doe', email_verified: true, last_sign_in_at: '2026-03-19T10:00:00Z', created_at: '2026-01-15T08:00:00Z', banned_at: null },
  { id: '2', email: 'jane@example.com', first_name: 'Jane', last_name: 'Smith', email_verified: true, last_sign_in_at: '2026-03-18T15:00:00Z', created_at: '2026-02-20T12:00:00Z', banned_at: null },
  { id: '3', email: 'bob@example.com', first_name: 'Bob', last_name: 'Wilson', email_verified: false, last_sign_in_at: null, created_at: '2026-03-10T09:00:00Z', banned_at: '2026-03-15T00:00:00Z' },
]

export function ProjectUsersPage() {
  const { slug: _slug } = useParams()
  const [search, setSearch] = useState('')
  const filtered = mockUsers.filter(u =>
    u.email.includes(search) || `${u.first_name} ${u.last_name}`.toLowerCase().includes(search.toLowerCase())
  )

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Users</h1>
        <span className="text-sm text-gray-500">{mockUsers.length} total</span>
      </div>

      <div className="mb-4">
        <input type="text" placeholder="Search by email or name..." value={search} onChange={e => setSearch(e.target.value)}
          className="w-full max-w-md px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 outline-none" />
      </div>

      <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
        <table className="w-full">
          <thead className="bg-gray-50 border-b border-gray-200">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">User</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Status</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Last Sign In</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Joined</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {filtered.map(user => (
              <tr key={user.id} className="hover:bg-gray-50 cursor-pointer">
                <td className="px-6 py-4">
                  <div className="font-medium">{user.first_name} {user.last_name}</div>
                  <div className="text-sm text-gray-500">{user.email}</div>
                </td>
                <td className="px-6 py-4">
                  {user.banned_at ? (
                    <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-700">Banned</span>
                  ) : user.email_verified ? (
                    <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-700">Verified</span>
                  ) : (
                    <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-700">Unverified</span>
                  )}
                </td>
                <td className="px-6 py-4 text-sm text-gray-500">
                  {user.last_sign_in_at ? new Date(user.last_sign_in_at).toLocaleDateString() : 'Never'}
                </td>
                <td className="px-6 py-4 text-sm text-gray-500">
                  {new Date(user.created_at).toLocaleDateString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
