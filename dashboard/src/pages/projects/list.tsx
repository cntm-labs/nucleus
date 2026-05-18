import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { projectsApi, type Project } from '../../lib/api'

function LoadingState() {
  return (
    <div className="bg-white rounded-xl border border-gray-200 p-12 text-center text-gray-500">
      Loading projects...
    </div>
  )
}

function ErrorState({ error }: { error: unknown }) {
  const msg = error instanceof Error ? error.message : 'unknown error'
  return (
    <div className="bg-red-50 border border-red-200 text-red-700 rounded-xl p-6 text-sm">
      Failed to load projects: {msg}
    </div>
  )
}

function EmptyState() {
  return (
    <div className="bg-white rounded-xl border border-gray-200 p-12 text-center">
      <p className="text-gray-500 mb-4">No projects yet</p>
      <Link to="/projects/new" className="text-nucleus-600 hover:underline">Create your first project</Link>
    </div>
  )
}

function ProjectGrid({ projects }: { projects: Project[] }) {
  return (
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
            </div>
          </div>
        </Link>
      ))}
    </div>
  )
}

export function ProjectListPage() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['projects'],
    queryFn: projectsApi.list,
  })

  const projects = data?.data ?? []

  let body: React.ReactNode
  if (isLoading) {
    body = <LoadingState />
  } else if (error) {
    body = <ErrorState error={error} />
  } else if (projects.length === 0) {
    body = <EmptyState />
  } else {
    body = <ProjectGrid projects={projects} />
  }

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
      {body}
    </div>
  )
}
