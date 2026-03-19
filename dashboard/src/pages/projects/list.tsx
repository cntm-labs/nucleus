import { Link } from 'react-router-dom'

export function ProjectListPage() {
  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold">Projects</h1>
        <Link to="/projects/new" className="px-4 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 text-sm font-medium">
          New Project
        </Link>
      </div>
      <div className="bg-white rounded-xl border border-gray-200 p-12 text-center text-gray-500">
        No projects yet. Create your first project to get started.
      </div>
    </div>
  )
}
