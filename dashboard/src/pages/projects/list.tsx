import { Link } from 'react-router-dom'

// Mock data for now
const mockProjects = [
  { id: '1', name: 'Orbit', slug: 'orbit', environment: 'production', data_mode: 'federated', users_count: 1234, created_at: '2026-01-15' },
  { id: '2', name: 'Demo App', slug: 'demo-app', environment: 'development', data_mode: 'centralized', users_count: 56, created_at: '2026-03-01' },
]

export function ProjectListPage() {
  const projects = mockProjects // TODO: useQuery to fetch from API

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold">Projects</h1>
          <p className="text-gray-500 mt-1">Manage your authentication projects</p>
        </div>
        <Link to="/projects/new" className="px-4 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 text-sm font-medium">
          New Project
        </Link>
      </div>

      {projects.length === 0 ? (
        <div className="bg-white rounded-xl border border-gray-200 p-12 text-center">
          <p className="text-gray-500 mb-4">No projects yet</p>
          <Link to="/projects/new" className="text-nucleus-600 hover:underline">Create your first project</Link>
        </div>
      ) : (
        <div className="grid gap-4">
          {projects.map((project) => (
            <Link key={project.id} to={`/projects/${project.slug}`}
              className="bg-white rounded-xl border border-gray-200 p-6 hover:border-nucleus-300 transition-colors">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-lg font-semibold">{project.name}</h2>
                  <p className="text-gray-500 text-sm mt-1">{project.slug}</p>
                </div>
                <div className="text-right">
                  <div className="text-sm">
                    <span className={`inline-block px-2 py-0.5 rounded-full text-xs font-medium ${
                      project.environment === 'production' ? 'bg-green-100 text-green-700' : 'bg-yellow-100 text-yellow-700'
                    }`}>
                      {project.environment}
                    </span>
                    <span className="ml-2 inline-block px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
                      {project.data_mode}
                    </span>
                  </div>
                  <p className="text-sm text-gray-500 mt-2">{project.users_count.toLocaleString()} users</p>
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
