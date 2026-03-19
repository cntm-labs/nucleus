import { useState } from 'react'
import { useNavigate } from 'react-router-dom'

export function ProjectCreatePage() {
  const navigate = useNavigate()
  const [name, setName] = useState('')
  const [slug, setSlug] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    // TODO: call API
    navigate('/projects')
  }

  return (
    <div>
      <h1 className="text-2xl font-bold mb-8">Create Project</h1>
      <form onSubmit={handleSubmit} className="max-w-lg space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Project Name</label>
          <input
            value={name} onChange={(e) => { setName(e.target.value); setSlug(e.target.value.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '')) }}
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 outline-none"
            placeholder="My App" required
          />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Slug</label>
          <input
            value={slug} onChange={(e) => setSlug(e.target.value)}
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 outline-none"
            placeholder="my-app" required
          />
        </div>
        <button type="submit" className="px-6 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 font-medium">
          Create
        </button>
      </form>
    </div>
  )
}
