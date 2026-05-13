import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { projectsApi } from '../../lib/api'

export function ProjectCreatePage() {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [name, setName] = useState('')
  const [slug, setSlug] = useState('')
  const [error, setError] = useState('')

  const mutation = useMutation({
    mutationFn: () => projectsApi.create({ name, slug }),
    onSuccess: (project) => {
      qc.invalidateQueries({ queryKey: ['projects'] })
      navigate(`/projects/${project.slug}`)
    },
    onError: (err) => {
      setError(err instanceof Error ? err.message : 'Create project failed')
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    mutation.mutate()
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
            placeholder="my-app" required pattern="[a-z0-9-]+"
          />
        </div>
        {error && <div className="bg-red-50 text-red-600 text-sm p-3 rounded-lg">{error}</div>}
        <button type="submit" disabled={mutation.isPending}
          className="px-6 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 font-medium disabled:opacity-50">
          {mutation.isPending ? 'Creating...' : 'Create'}
        </button>
      </form>
    </div>
  )
}
